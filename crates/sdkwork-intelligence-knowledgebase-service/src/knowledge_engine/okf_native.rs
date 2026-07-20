use async_trait::async_trait;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_mode, KnowledgeEngineDescriptor, KnowledgeEngineDocument,
    KnowledgeEngineDocumentList, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineHealth, KnowledgeEngineHealthStatus, KnowledgeEngineId,
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchHit,
    KnowledgeEngineSearchRequest, KnowledgeEngineSearchResult,
};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleLintResult,
    OkfConceptSummary, OkfConceptUpsertRequest, PublishKnowledgeOkfConceptRequest,
};
use sdkwork_knowledgebase_contract::okf_bundle_file::KnowledgeOkfBundleFile;
use sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineExecutionContext;
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::OkfBundleFileKind;
use sdkwork_utils_rust::is_blank;
use std::sync::Arc;

use crate::okf::{
    load_import_bundle_from_drive, read_managed_markdown, rebuild_bundle_index_for_space,
    to_contract_lint_result, ExportOkfBundleRequest, ImportOkfBundleRequest,
    OkfBundleExporterService, OkfBundleImporterError, OkfBundleImporterService,
    OkfBundleLinterService, OkfConceptService, OkfConceptServiceError,
};
use crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStore;
use crate::ports::knowledge_drive_storage::{KnowledgeDriveStorage, KnowledgeStorageError};
use crate::ports::knowledge_drive_workspace::KnowledgeDriveWorkspace;
use crate::ports::knowledge_okf_bundle_file_store::{
    CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
};
use crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStore;
use crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStore;
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_source_store::KnowledgeSourceStore;
use crate::ports::knowledge_space_store::KnowledgeSpaceStore;
use crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStore;

use super::okf_search::{
    body_match_score, combine_metadata_and_body_score, expand_ranked_with_link_edges,
    normalize_query, rank_okf_concepts_with_tokens, snippet_for_concept,
};
use super::KnowledgeEngine;
use crate::ports::knowledge_engine::OkfBundleEngine;

const SPI_ACTOR: &str = "knowledge-engine-spi";
const OKF_DRIVE_READ_CONCURRENCY: usize = 8;

#[derive(Clone)]
pub struct OkfNativeKnowledgeEngineDeps {
    pub concepts: Arc<dyn KnowledgeOkfConceptStore>,
    pub drive: Arc<dyn KnowledgeDriveStorage>,
    pub revision_metadata: Arc<dyn OkfConceptRevisionMetadataStore>,
    pub object_refs: Arc<dyn KnowledgeDriveObjectRefStore>,
    pub link_store: Arc<dyn KnowledgeOkfConceptLinkStore>,
    pub candidate_store: Arc<dyn KnowledgeOkfCandidateStore>,
    pub bundle_file_store: Arc<dyn KnowledgeOkfBundleFileStore>,
    pub drive_workspace: Arc<dyn KnowledgeDriveWorkspace>,
    pub source_store: Arc<dyn KnowledgeSourceStore>,
    pub space_store: Arc<dyn KnowledgeSpaceStore>,
}

impl OkfNativeKnowledgeEngineDeps {
    pub fn minimal(
        concepts: Arc<dyn KnowledgeOkfConceptStore>,
        drive: Arc<dyn KnowledgeDriveStorage>,
    ) -> Self {
        Self {
            concepts,
            drive,
            revision_metadata: Arc::new(UnsupportedOkfConceptRevisionMetadataStore),
            object_refs: Arc::new(UnsupportedObjectRefStore),
            link_store: Arc::new(UnsupportedLinkStore),
            candidate_store: Arc::new(UnsupportedCandidateStore),
            bundle_file_store: Arc::new(UnsupportedBundleFileStore),
            drive_workspace: Arc::new(UnsupportedDriveWorkspace),
            source_store: Arc::new(UnsupportedSourceStore),
            space_store: Arc::new(UnsupportedSpaceStore),
        }
    }
}

pub struct OkfNativeKnowledgeEngine {
    deps: OkfNativeKnowledgeEngineDeps,
}

impl OkfNativeKnowledgeEngine {
    pub fn new(
        concepts: Arc<dyn KnowledgeOkfConceptStore>,
        drive: Arc<dyn KnowledgeDriveStorage>,
    ) -> Self {
        Self::from_deps(OkfNativeKnowledgeEngineDeps::minimal(concepts, drive))
    }

    pub fn from_deps(deps: OkfNativeKnowledgeEngineDeps) -> Self {
        Self { deps }
    }

    fn concept_service(&self) -> OkfConceptService<'_> {
        OkfConceptService::new(
            self.deps.drive.as_ref(),
            self.deps.revision_metadata.as_ref(),
            self.deps.concepts.as_ref(),
        )
        .with_link_store(self.deps.link_store.as_ref())
        .with_candidate_store(self.deps.candidate_store.as_ref())
        .with_file_entry_store(self.deps.bundle_file_store.as_ref())
        .with_drive_workspace(self.deps.drive_workspace.as_ref())
    }

    async fn drive_space_id(&self, space_id: u64) -> Result<Option<String>, KnowledgeEngineError> {
        let space = self
            .deps
            .space_store
            .get_space(space_id)
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;
        Ok(space.drive_space_id)
    }

    pub async fn import_bundle_for_actor(
        &self,
        request: OkfBundleImportRequest,
        actor: &str,
    ) -> Result<sdkwork_knowledgebase_contract::okf::OkfBundleImportResult, KnowledgeEngineError>
    {
        if request.space_id == 0 || is_blank(Some(request.import_type.as_str())) {
            return Err(KnowledgeEngineError::Validation(
                "space_id and import_type are required".to_string(),
            ));
        }
        let import_type = request.import_type.trim();
        if import_type != "okf_strict" && import_type != "okf_bundle" {
            return Err(KnowledgeEngineError::Validation(format!(
                "unsupported import_type: {import_type}"
            )));
        }
        let publish = import_type == "okf_strict";
        let drive_space_id = self.drive_space_id(request.space_id).await?;
        let files = load_import_bundle_from_drive(
            self.deps.drive.as_ref(),
            request.space_id,
            request.import_id.as_deref(),
            drive_space_id.as_deref(),
        )
        .await
        .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        self.import_bundle_files(
            ImportOkfBundleRequest {
                space_id: request.space_id,
                actor: actor.to_string(),
                publish,
                files,
            },
            drive_space_id.as_deref(),
        )
        .await
    }

    pub async fn import_bundle_files(
        &self,
        request: ImportOkfBundleRequest,
        drive_space_id: Option<&str>,
    ) -> Result<sdkwork_knowledgebase_contract::okf::OkfBundleImportResult, KnowledgeEngineError>
    {
        let result = OkfBundleImporterService::new(self.concept_service())
            .import_bundle(request, drive_space_id)
            .await
            .map_err(map_importer_error)?;

        Ok(sdkwork_knowledgebase_contract::okf::OkfBundleImportResult {
            imported_concept_count: result.imported_concept_count,
            skipped_files: result.skipped_files,
        })
    }

    pub async fn publish_existing_revision(
        &self,
        request: crate::okf::PublishExistingOkfConceptRevisionRequest,
        drive_space_id: Option<&str>,
    ) -> Result<
        sdkwork_knowledgebase_contract::okf::KnowledgeOkfConceptPublication,
        KnowledgeEngineError,
    > {
        self.concept_service()
            .publish_existing_revision(request, drive_space_id)
            .await
            .map_err(map_concept_service_error)
    }
}

#[async_trait]
impl KnowledgeEngine for OkfNativeKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        descriptor_for_mode(KnowledgeAgentKnowledgeMode::OkfBundle)
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        Ok(KnowledgeEngineHealth {
            implementation_id: KnowledgeEngineId::OKF_NATIVE.to_string(),
            status: KnowledgeEngineHealthStatus::Available,
            detail: Some("native OKF bundle engine".to_string()),
        })
    }

    async fn search(
        &self,
        _context: &KnowledgeEngineExecutionContext,
        request: KnowledgeEngineSearchRequest,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let pages = self
            .deps
            .concepts
            .list_concept_summaries(request.space_id, None)
            .await
            .map_err(map_okf_store_error)?;

        let summaries_by_id = pages
            .iter()
            .map(|page| (page.concept_id.clone(), page.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let tokens = normalize_query(&request.query);
        let candidate_limit = request.top_k.saturating_mul(4).max(8) as usize;
        let mut ranked = rank_okf_concepts_with_tokens(pages, &tokens);

        if let Ok(edges) = self
            .deps
            .link_store
            .list_active_link_edges(request.space_id)
            .await
        {
            ranked = expand_ranked_with_link_edges(
                ranked,
                &edges,
                &summaries_by_id,
                &tokens,
                candidate_limit,
            );
        }

        let drive_space_id = self.drive_space_id(request.space_id).await.ok().flatten();
        let candidates: Vec<_> = ranked.into_iter().take(candidate_limit).collect();
        let drive = Arc::clone(&self.deps.drive);
        let mut hits = Vec::with_capacity(candidates.len().min(request.top_k as usize));

        for chunk in candidates.chunks(OKF_DRIVE_READ_CONCURRENCY) {
            let mut handles = Vec::with_capacity(chunk.len());
            for (metadata_score, concept) in chunk {
                let drive = Arc::clone(&drive);
                let drive_space_id = drive_space_id.clone();
                let logical_path = concept.logical_path.clone();
                let metadata_score = *metadata_score;
                let concept = concept.clone();
                handles.push(tokio::spawn(async move {
                    let body = match read_managed_markdown(
                        drive.as_ref(),
                        &logical_path,
                        drive_space_id.as_deref(),
                    )
                    .await
                    {
                        Ok(content) => Some(content),
                        Err(error) => {
                            tracing::warn!(
                                logical_path = %logical_path,
                                ?error,
                                "okf native search skipped drive body read; using metadata-only ranking"
                            );
                            None
                        }
                    };
                    (metadata_score, concept, body)
                }));
            }

            for handle in handles {
                let (metadata_score, concept, body) = handle
                    .await
                    .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;
                let body_score = body
                    .as_deref()
                    .map(|content| body_match_score(content, &tokens))
                    .unwrap_or(0.0);
                let score = combine_metadata_and_body_score(metadata_score, body_score);
                if score <= 0.0 && !tokens.is_empty() {
                    continue;
                }
                hits.push(KnowledgeEngineSearchHit {
                    document: KnowledgeEngineDocumentRef {
                        document_id: concept.concept_id.clone(),
                        title: concept.title.clone(),
                        source_uri: Some(concept.logical_path.clone()),
                    },
                    snippet: snippet_for_concept(
                        &concept.description,
                        body.as_deref(),
                        &request.query,
                    ),
                    score: Some(score),
                });
            }
        }

        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.document.document_id.cmp(&right.document.document_id))
        });
        hits.truncate(request.top_k.max(1) as usize);

        Ok(KnowledgeEngineSearchResult {
            implementation_id: KnowledgeEngineId::OKF_NATIVE.to_string(),
            hits,
        })
    }

    async fn read_document(
        &self,
        _context: &KnowledgeEngineExecutionContext,
        request: KnowledgeEngineReadRequest,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let pages = self
            .deps
            .concepts
            .list_concept_summaries(request.space_id, None)
            .await
            .map_err(map_okf_store_error)?;

        let concept = pages
            .into_iter()
            .find(|page| {
                page.concept_id == request.document_id || page.logical_path == request.document_id
            })
            .ok_or_else(|| {
                KnowledgeEngineError::NotFound(format!(
                    "okf concept not found: {}",
                    request.document_id
                ))
            })?;

        let drive_space_id = self.drive_space_id(request.space_id).await.ok().flatten();
        let content = read_managed_markdown(
            self.deps.drive.as_ref(),
            &concept.logical_path,
            drive_space_id.as_deref(),
        )
        .await
        .map_err(map_drive_error)?;

        Ok(KnowledgeEngineDocument {
            document_id: concept.concept_id,
            title: concept.title,
            content,
            source_uri: Some(concept.logical_path),
        })
    }

    async fn list_documents(
        &self,
        _context: &KnowledgeEngineExecutionContext,
        request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        let pages = self
            .deps
            .concepts
            .list_concept_summaries(request.space_id, Some(request.limit.max(1)))
            .await
            .map_err(map_okf_store_error)?;

        let items = pages
            .into_iter()
            .map(|concept| KnowledgeEngineDocumentRef {
                document_id: concept.concept_id,
                title: concept.title,
                source_uri: Some(concept.logical_path),
            })
            .collect();

        Ok(KnowledgeEngineDocumentList { items })
    }
}

#[async_trait]
impl OkfBundleEngine for OkfNativeKnowledgeEngine {
    async fn list_concepts(
        &self,
        space_id: u64,
    ) -> Result<Vec<OkfConceptSummary>, KnowledgeEngineError> {
        self.deps
            .concepts
            .list_concept_summaries(space_id, None)
            .await
            .map_err(map_okf_store_error)
    }

    async fn upsert_concept(
        &self,
        request: OkfConceptUpsertRequest,
    ) -> Result<KnowledgeOkfConcept, KnowledgeEngineError> {
        let drive_space_id = self.drive_space_id(request.space_id).await?;
        let publication = self
            .concept_service()
            .upsert_concept_from_markdown(request, drive_space_id.as_deref())
            .await
            .map_err(map_concept_service_error)?;
        Ok(publication.concept)
    }

    async fn delete_concept(
        &self,
        space_id: u64,
        concept_row_id: u64,
        actor: &str,
    ) -> Result<(), KnowledgeEngineError> {
        let drive_space_id = self.drive_space_id(space_id).await?;
        self.concept_service()
            .delete_concept(space_id, concept_row_id, actor, drive_space_id.as_deref())
            .await
            .map_err(map_concept_service_error)
    }

    async fn publish_concept(
        &self,
        request: PublishKnowledgeOkfConceptRequest,
    ) -> Result<
        sdkwork_knowledgebase_contract::okf::KnowledgeOkfConceptPublication,
        KnowledgeEngineError,
    > {
        let drive_space_id = self.drive_space_id(request.space_id).await?;
        self.concept_service()
            .publish_concept(request, drive_space_id.as_deref())
            .await
            .map_err(map_concept_service_error)
    }

    async fn lint_bundle_report(
        &self,
        space_id: u64,
    ) -> Result<OkfBundleLintResult, KnowledgeEngineError> {
        let drive_space_id = self.drive_space_id(space_id).await?;
        let report =
            OkfBundleLinterService::new(self.deps.drive.as_ref(), self.deps.concepts.as_ref())
                .with_link_store(self.deps.link_store.as_ref())
                .with_source_store(self.deps.source_store.as_ref())
                .lint_space(space_id, drive_space_id.as_deref())
                .await
                .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;
        Ok(to_contract_lint_result(&report))
    }

    async fn import_bundle(
        &self,
        request: OkfBundleImportRequest,
    ) -> Result<sdkwork_knowledgebase_contract::okf::OkfBundleImportResult, KnowledgeEngineError>
    {
        self.import_bundle_for_actor(request, SPI_ACTOR).await
    }

    async fn export_bundle(
        &self,
        request: OkfBundleExportRequest,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeEngineError> {
        if request.space_id == 0 || is_blank(Some(request.export_type.as_str())) {
            return Err(KnowledgeEngineError::Validation(
                "space_id and export_type are required".to_string(),
            ));
        }
        if request.stage_for_import {
            return Err(KnowledgeEngineError::Unsupported(
                "okf export stage_for_import is owned by hosted export routes".to_string(),
            ));
        }

        let source_object_refs = if request.export_type.trim() == "okf_with_sources" {
            self.deps
                .object_refs
                .list_object_refs_by_logical_path_prefix(request.space_id, "sources/raw/")
                .await
                .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?
        } else {
            Vec::new()
        };

        let drive_space_id = self.drive_space_id(request.space_id).await?;
        let exported =
            OkfBundleExporterService::new(self.deps.drive.as_ref(), self.deps.concepts.as_ref())
                .with_source_object_refs(source_object_refs)
                .export_bundle(
                    ExportOkfBundleRequest {
                        space_id: request.space_id,
                        export_type: request.export_type,
                    },
                    drive_space_id.as_deref(),
                )
                .await
                .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        self.deps
            .bundle_file_store
            .create_file_entry(CreateKnowledgeOkfBundleFileRecord {
                space_id: request.space_id,
                logical_path: exported.manifest_path,
                file_kind: OkfBundleFileKind::OutputExport,
                artifact_role: exported.manifest_ref.object_role.clone(),
                drive_bucket: exported.manifest_ref.bucket.clone(),
                drive_object_key: exported.manifest_ref.object_key.clone(),
                checksum_sha256_hex: exported.manifest_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))
    }

    async fn rebuild_index(&self, space_id: u64) -> Result<(), KnowledgeEngineError> {
        rebuild_bundle_index_for_space(
            self.deps.drive.as_ref(),
            self.deps.concepts.as_ref(),
            self.deps.space_store.as_ref(),
            space_id,
        )
        .await
        .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))
    }
}

fn map_okf_store_error(error: KnowledgeOkfConceptStoreError) -> KnowledgeEngineError {
    KnowledgeEngineError::Internal(error.to_string())
}

fn map_drive_error(error: KnowledgeStorageError) -> KnowledgeEngineError {
    match error {
        KnowledgeStorageError::NotFound(detail) => KnowledgeEngineError::NotFound(detail),
        other => KnowledgeEngineError::Internal(other.to_string()),
    }
}

fn map_importer_error(error: OkfBundleImporterError) -> KnowledgeEngineError {
    match error {
        OkfBundleImporterError::InvalidRequest(message) => {
            KnowledgeEngineError::Validation(message)
        }
        OkfBundleImporterError::Conformance(message) => KnowledgeEngineError::Validation(message),
        OkfBundleImporterError::Storage(error) => map_drive_error(error),
        OkfBundleImporterError::ConceptService(error) => map_concept_service_error(error),
    }
}

fn map_concept_service_error(error: OkfConceptServiceError) -> KnowledgeEngineError {
    match error {
        OkfConceptServiceError::InvalidRequest(message) => {
            KnowledgeEngineError::Validation(message)
        }
        OkfConceptServiceError::Conformance(error) => {
            KnowledgeEngineError::Validation(error.to_string())
        }
        OkfConceptServiceError::Storage(error) => match error {
            KnowledgeStorageError::NotFound(detail) => KnowledgeEngineError::NotFound(detail),
            other => KnowledgeEngineError::Internal(other.to_string()),
        },
        other => KnowledgeEngineError::Internal(other.to_string()),
    }
}

struct UnsupportedOkfConceptRevisionMetadataStore;

#[async_trait::async_trait]
impl crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStore
    for UnsupportedOkfConceptRevisionMetadataStore
{
    async fn prepare_concept_revision_slot(
        &self,
        _concept: crate::ports::knowledge_okf_concept_store::UpsertKnowledgeOkfConceptRecord,
    ) -> Result<
        crate::ports::okf_concept_revision_metadata_store::PreparedOkfConceptRevisionSlot,
        crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError,
    > {
        Err(
            crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError::internal(
                "okf native engine missing revision metadata store wiring".to_string(),
            ),
        )
    }

    async fn stage_concept_revision_metadata(
        &self,
        _record: crate::ports::okf_concept_revision_metadata_store::StageOkfConceptRevisionMetadataRecord,
    ) -> Result<
        crate::ports::okf_concept_revision_metadata_store::StagedOkfConceptRevisionMetadata,
        crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError,
    > {
        Err(
            crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError::internal(
                "okf native engine missing revision metadata store wiring".to_string(),
            ),
        )
    }

    async fn publish_existing_revision_metadata(
        &self,
        _record: crate::ports::okf_concept_revision_metadata_store::PublishOkfConceptRevisionMetadataRecord,
    ) -> Result<
        crate::ports::okf_concept_revision_metadata_store::PublishedOkfConceptRevisionMetadata,
        crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError,
    > {
        Err(
            crate::ports::okf_concept_revision_metadata_store::OkfConceptRevisionMetadataStoreError::internal(
                "okf native engine missing revision metadata store wiring".to_string(),
            ),
        )
    }
}

struct UnsupportedObjectRefStore;

#[async_trait::async_trait]
impl KnowledgeDriveObjectRefStore for UnsupportedObjectRefStore {
    async fn create_object_ref(
        &self,
        _record: crate::ports::knowledge_drive_object_ref_store::CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef,
        crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError,
    > {
        Err(crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError::Internal(
            "okf native engine missing object ref store wiring".to_string(),
        ))
    }

    async fn list_object_refs_by_logical_path_prefix(
        &self,
        _space_id: u64,
        _prefix: &str,
    ) -> Result<
        Vec<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef>,
        crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError,
    > {
        Err(crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError::Internal(
            "okf native engine missing object ref store wiring".to_string(),
        ))
    }

    async fn get_object_ref_by_id(
        &self,
        _object_ref_id: u64,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef,
        crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError,
    > {
        Err(crate::ports::knowledge_drive_object_ref_store::KnowledgeDriveObjectRefStoreError::Internal(
            "okf native engine missing object ref store wiring".to_string(),
        ))
    }
}

struct UnsupportedLinkStore;

#[async_trait::async_trait]
impl KnowledgeOkfConceptLinkStore for UnsupportedLinkStore {
    async fn replace_outbound_links(
        &self,
        _record: crate::ports::knowledge_okf_concept_link_store::ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStoreError>
    {
        Err(crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStoreError::Internal(
            "okf native engine missing link store wiring".to_string(),
        ))
    }

    async fn list_inbound_concept_ids(
        &self,
        _space_id: u64,
        _to_concept_id: &str,
    ) -> Result<
        Vec<String>,
        crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStoreError,
    > {
        Ok(Vec::new())
    }

    async fn list_orphan_concept_ids(
        &self,
        _space_id: u64,
        _published_concept_ids: &[String],
    ) -> Result<
        Vec<String>,
        crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStoreError,
    > {
        Ok(Vec::new())
    }

    async fn list_active_link_edges(
        &self,
        _space_id: u64,
    ) -> Result<
        Vec<crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkEdge>,
        crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStoreError,
    > {
        Ok(Vec::new())
    }
}

struct UnsupportedCandidateStore;

#[async_trait::async_trait]
impl KnowledgeOkfCandidateStore for UnsupportedCandidateStore {
    async fn upsert_candidate(
        &self,
        _record: crate::ports::knowledge_okf_candidate_store::UpsertKnowledgeOkfCandidateRecord,
    ) -> Result<(), crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError>
    {
        Err(
            crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError::Internal(
                "okf native engine missing candidate store wiring".to_string(),
            ),
        )
    }

    async fn update_candidate_state_by_concept_row_id(
        &self,
        _concept_row_id: u64,
        _state: sdkwork_knowledgebase_contract::OkfConceptPublishState,
        _reviewer_id: Option<u64>,
        _review_note: Option<String>,
    ) -> Result<(), crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError>
    {
        Err(
            crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError::Internal(
                "okf native engine missing candidate store wiring".to_string(),
            ),
        )
    }

    async fn list_open_candidates(
        &self,
        _space_id: Option<u64>,
    ) -> Result<
        Vec<crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateListItem>,
        crate::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError,
    > {
        Ok(Vec::new())
    }
}

struct UnsupportedBundleFileStore;

#[async_trait::async_trait]
impl KnowledgeOkfBundleFileStore for UnsupportedBundleFileStore {
    async fn create_file_entry(
        &self,
        _record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<
        KnowledgeOkfBundleFile,
        crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStoreError,
    > {
        Err(crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStoreError::Internal(
            "okf native engine missing bundle file store wiring".to_string(),
        ))
    }
}

struct UnsupportedDriveWorkspace;

#[async_trait::async_trait]
impl KnowledgeDriveWorkspace for UnsupportedDriveWorkspace {
    async fn ensure_nodes(
        &self,
        _request: crate::ports::knowledge_drive_workspace::EnsureKnowledgeDriveNodesRequest,
    ) -> Result<(), crate::ports::knowledge_drive_workspace::KnowledgeDriveWorkspaceError> {
        Ok(())
    }
}

struct UnsupportedSourceStore;

#[async_trait::async_trait]
impl KnowledgeSourceStore for UnsupportedSourceStore {
    async fn create_source(
        &self,
        _record: crate::ports::knowledge_source_store::CreateKnowledgeSourceRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::source::KnowledgeSource,
        crate::ports::knowledge_source_store::KnowledgeSourceStoreError,
    > {
        Err(
            crate::ports::knowledge_source_store::KnowledgeSourceStoreError::Internal(
                "okf native engine missing source store wiring".to_string(),
            ),
        )
    }
}

struct UnsupportedSpaceStore;

#[async_trait::async_trait]
impl KnowledgeSpaceStore for UnsupportedSpaceStore {
    async fn create_space(
        &self,
        _record: crate::ports::knowledge_space_store::CreateKnowledgeSpaceRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::space::KnowledgeSpace,
        crate::ports::knowledge_space_store::KnowledgeSpaceStoreError,
    > {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }

    async fn get_space(
        &self,
        _space_id: u64,
    ) -> Result<
        sdkwork_knowledgebase_contract::space::KnowledgeSpace,
        crate::ports::knowledge_space_store::KnowledgeSpaceStoreError,
    > {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }

    async fn mark_drive_space_bound(
        &self,
        _space_id: u64,
        _drive_space_id: String,
    ) -> Result<
        sdkwork_knowledgebase_contract::space::KnowledgeSpace,
        crate::ports::knowledge_space_store::KnowledgeSpaceStoreError,
    > {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }

    async fn mark_okf_bundle_initialized(
        &self,
        _space_id: u64,
    ) -> Result<
        sdkwork_knowledgebase_contract::space::KnowledgeSpace,
        crate::ports::knowledge_space_store::KnowledgeSpaceStoreError,
    > {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }

    async fn update_space(
        &self,
        _space_id: u64,
        _record: crate::ports::knowledge_space_store::UpdateKnowledgeSpaceRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::space::KnowledgeSpace,
        crate::ports::knowledge_space_store::KnowledgeSpaceStoreError,
    > {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }

    async fn mark_space_deleted(
        &self,
        _space_id: u64,
    ) -> Result<(), crate::ports::knowledge_space_store::KnowledgeSpaceStoreError> {
        Err(
            crate::ports::knowledge_space_store::KnowledgeSpaceStoreError::Internal(
                "okf native engine missing space store wiring".to_string(),
            ),
        )
    }
}
