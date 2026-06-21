use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_node_tree::{
    GetKnowledgeDriveNodeRequest, KnowledgeDriveNodeTree,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::{
    browser::{KnowledgeBrowserAccessContext, KnowledgeBrowserService},
    imports::KnowledgeDriveImportService,
    ingest::KnowledgeApiPayloadIngestService,
    okf::OkfConceptServiceError,
    ports::{
        knowledge_document_store::{
            CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
        },
        knowledge_document_version_store::{
            CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
        },
        knowledge_ingestion_job_store::IngestionJobStore,
        knowledge_okf_bundle_file_store::{
            CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
        },
        knowledge_source_store::KnowledgeSourceStore,
        knowledge_space_store::KnowledgeSpaceStore,
    },
};
use sdkwork_utils_rust::is_blank;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, IngestionJob, KnowledgeBrowserPage, KnowledgeDocument,
    KnowledgeDocumentList, KnowledgeDocumentVersion, KnowledgeDocumentVersionList,
    KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeIngestRequest,
    KnowledgeOkfBundleFile, KnowledgeOkfConceptRevisionList, KnowledgeSpace,
    ListKnowledgeBrowserRequest, OkfBundleExportRequest, OkfBundleFileKind, OkfBundleImportRequest,
    OkfBundleImportResult, OkfConceptSummary, OkfConceptSummaryList, OkfConceptUpsertRequest,
    OkfContextPackRequest, OkfFileAnswerRequest, OkfIndexDocument, OkfLogDocument,
    OkfProfileDocument, OkfQualityRun, OkfQualityRunRequest, OkfQueryRequest, OkfQueryResult,
    PublishKnowledgeOkfConceptRequest, GrantKnowledgeSpaceMemberRequest, KnowledgeSpaceMemberList,
    KnowledgeSpaceMemberSubjectType, UpdateKnowledgeSpaceRequest,
};

use crate::{
    hosted_access::{
        create_space_with_context, delete_space_with_context, grant_space_member_with_context,
        list_space_members_with_context, require_document_access, require_ingest_access,
        require_okf_concept_space_access, require_space_access, revoke_space_member_with_context,
        update_space_with_context,
    },
    hosted_support::{
        build_okf_context_pack_from_engine, concept_to_summary, create_okf_bundle_export,
        create_okf_bundle_import, create_okf_lint_run, format_okf_engine_answer,
        okf_answer_concept_id, okf_bundle_not_initialized_detail, okf_paths, read_managed_okf_text,
        retrieve_okf_bundle_export, stable_u64_hash,
    },
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
    KnowledgeDocumentAppService, KnowledgeDriveImportAppService, KnowledgeIngestAppService,
    KnowledgeOkfAppService, KnowledgeSpaceAppService,
};

#[derive(Clone)]
pub(crate) struct HostedSpaceService {
    runtime: KnowledgebaseRuntime,
}

impl HostedSpaceService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeSpaceAppService for HostedSpaceService {
    async fn create_space(
        &self,
        context: KnowledgeAppRequestContext,
        mut request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        if request.owner_subject_id.is_none() {
            if let Some(actor_id) = context.actor_id {
                request.owner_subject_id = Some(actor_id.to_string());
            }
        }
        if request.owner_subject_type.is_none() {
            request.owner_subject_type = Some("user".to_string());
        }
        create_space_with_context(&self.runtime, &context, request).await
    }

    async fn retrieve_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpace> {
        require_space_access(&self.runtime, &context, space_id).await
    }

    async fn update_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: UpdateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        update_space_with_context(&self.runtime, &context, space_id, request).await
    }

    async fn delete_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<()> {
        delete_space_with_context(&self.runtime, &context, space_id).await
    }

    async fn list_space_members(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpaceMemberList> {
        list_space_members_with_context(&self.runtime, &context, space_id).await
    }

    async fn grant_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: GrantKnowledgeSpaceMemberRequest,
    ) -> ApiResult<()> {
        grant_space_member_with_context(&self.runtime, &context, space_id, request).await
    }

    async fn revoke_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        subject_type: KnowledgeSpaceMemberSubjectType,
        subject_id: String,
    ) -> ApiResult<()> {
        revoke_space_member_with_context(
            &self.runtime,
            &context,
            space_id,
            subject_type,
            &subject_id,
        )
        .await
    }
}

#[derive(Clone)]
pub(crate) struct HostedIngestService {
    runtime: KnowledgebaseRuntime,
}

impl HostedIngestService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeIngestAppService for HostedIngestService {
    async fn create_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeIngestRequest,
    ) -> ApiResult<IngestionJob> {
        use sdkwork_intelligence_knowledgebase_service::ingest::{
            KnowledgeApiMarkdownIndexService, KnowledgeIngestionService,
        };
        use sdkwork_knowledgebase_contract::ingest::IngestionJobState;

        let space_id = request.space_id;
        let title = request.title.clone();
        let payload_markdown = request.payload_markdown.clone();

        let space = require_space_access(&self.runtime, &context, space_id).await?;
        let service = KnowledgeApiPayloadIngestService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        );
        let result = service
            .ingest_markdown_payload(request, space.drive_space_id.as_deref())
            .await
            .map_err(ApiError::from)?;
        let mut job = result.job;
        if job.state == IngestionJobState::Queued {
            let ingestion = KnowledgeIngestionService::new(self.runtime.ingestion_job_store());
            job = ingestion
                .mark_running(job.id)
                .await
                .map_err(ApiError::from)?;

            let source = self
                .runtime
                .source_store()
                .create_or_get_source(
                    sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::CreateKnowledgeSourceRecord {
                        space_id,
                        source_type: sdkwork_knowledgebase_contract::source::KnowledgeSourceType::Api,
                        provider: Some("api-ingest".to_string()),
                        drive_bucket: None,
                        drive_prefix: Some(format!("inbox/api/{}", job.id)),
                        connector_metadata_json: None,
                    },
                )
                .await
                .map_err(ApiError::from)?;

            let indexer = KnowledgeApiMarkdownIndexService::new(
                self.runtime.document_store(),
                self.runtime.version_store(),
                self.runtime.object_ref_store(),
                self.runtime.chunk_store(),
            );
            let index_result = match indexer
                .index_payload_markdown(
                    space_id,
                    source.id,
                    &title,
                    &payload_markdown,
                    &result.payload_object_ref,
                )
                .await
            {
                Ok(index_result) => index_result,
                Err(error) => {
                    let _ = ingestion.mark_failed(job.id, format!("{error:?}")).await;
                    return Err(ApiError::from(error));
                }
            };

            let _ = self
                .runtime
                .try_embed_document_version(space_id, index_result.document_version_id)
                .await;

            job = ingestion
                .mark_succeeded(job.id)
                .await
                .map_err(ApiError::from)?;
            self.runtime.try_append_ingest_succeeded_outbox(&job).await;
        }
        Ok(job)
    }

    async fn retrieve_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        ingest_id: u64,
    ) -> ApiResult<IngestionJob> {
        require_ingest_access(&self.runtime, &context, ingest_id).await
    }
}

#[derive(Clone)]
pub(crate) struct HostedDriveImportService {
    runtime: KnowledgebaseRuntime,
}

impl HostedDriveImportService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeDriveImportAppService for HostedDriveImportService {
    async fn import_drive_object(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        let request = resolve_drive_import_request(self.runtime.drive_tree(), request).await?;
        let service = KnowledgeDriveImportService::new(
            self.runtime.drive_storage(),
            self.runtime.source_store(),
            self.runtime.document_store(),
            self.runtime.object_ref_store(),
            self.runtime.version_store(),
            self.runtime.ingestion_job_store(),
        );
        let result = service
            .import_drive_object(request)
            .await
            .map_err(ApiError::from)?;

        let pipeline = sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionJobWorkerService::new(
            self.runtime.ingestion_job_store(),
            self.runtime.drive_storage(),
            self.runtime.chunk_store(),
        );
        if let Ok(pipeline_result) = pipeline.process_drive_import_result(&result).await {
            if let Some(index_result) = pipeline_result.index_result {
                let _ = self
                    .runtime
                    .try_embed_document_version(
                        result.document.space_id,
                        index_result.document_version_id,
                    )
                    .await;
            }
            if pipeline_result.job.state
                == sdkwork_knowledgebase_contract::ingest::IngestionJobState::Succeeded
            {
                self.runtime
                    .try_append_ingest_succeeded_outbox(&pipeline_result.job)
                    .await;
            }
            return Ok(KnowledgeDriveImportResult {
                job: pipeline_result.job,
                ..result
            });
        }

        Ok(result)
    }
}

#[derive(Clone)]
pub(crate) struct HostedDocumentService {
    runtime: KnowledgebaseRuntime,
}

impl HostedDocumentService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeDocumentAppService for HostedDocumentService {
    async fn list_documents(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeDocumentList> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_list_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, space_id).await?;
        let items = self
            .runtime
            .document_store()
            .list_documents_for_space(space_id, 200)
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeDocumentList { items })
    }

    async fn create_document(
        &self,
        context: KnowledgeAppRequestContext,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "title is required",
            ));
        }
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, request.space_id).await?;

        self.runtime
            .document_store()
            .create_document(CreateKnowledgeDocumentRecord {
                space_id: request.space_id,
                collection_id: request.collection_id.unwrap_or(0),
                source_id: request.source_id,
                identity_scope: KnowledgeDocumentIdentityScope::SourceOnly,
                original_file_drive_node_id: None,
                title: request.title,
                mime_type: request.mime_type,
                language: request.language,
            })
            .await
            .map_err(Into::into)
    }

    async fn retrieve_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocument> {
        require_document_access(&self.runtime, &context, document_id).await
    }

    async fn update_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        let document = require_document_access(&self.runtime, &context, document_id).await?;
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "title is required",
            ));
        }
        if request.space_id != 0 && request.space_id != document.space_id {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_request",
                "space_id does not match the document space",
            ));
        }
        self.runtime
            .document_store()
            .update_document_metadata(
                document_id,
                request.title,
                request.mime_type,
                request.language,
            )
            .await
            .map_err(Into::into)
    }

    async fn delete_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<()> {
        require_document_access(&self.runtime, &context, document_id).await?;
        self.runtime
            .document_store()
            .soft_delete_document(document_id)
            .await
            .map_err(Into::into)
    }

    async fn list_document_versions(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        require_document_access(&self.runtime, &context, document_id).await?;
        let items = self
            .runtime
            .version_store()
            .list_versions_for_document(document_id)
            .await
            .map_err(map_document_version_error)?;
        Ok(KnowledgeDocumentVersionList { items })
    }

    async fn create_document_version(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        require_document_access(&self.runtime, &context, document_id).await?;
        if request.document_id != 0 && request.document_id != document_id {
            return Err(ApiError::invalid_request(
                "invalid_knowledge_document_version_request",
                "document_id in body must match path documentId when provided",
            ));
        }
        self.runtime
            .document_store()
            .get_document_by_id(document_id)
            .await
            .map_err(ApiError::from)?;

        let existing = self
            .runtime
            .version_store()
            .list_versions_for_document(document_id)
            .await
            .map_err(map_document_version_error)?;
        let version_no = existing
            .iter()
            .map(|version| version.version_no)
            .max()
            .unwrap_or(0)
            .saturating_add(1);

        self.runtime
            .version_store()
            .create_document_version(CreateKnowledgeDocumentVersionRecord {
                document_id,
                version_no,
                original_object_ref_id: request.original_object_ref_id,
                checksum_sha256_hex: request.checksum_sha256_hex,
                size_bytes: request.size_bytes,
                mime_type: request.mime_type,
            })
            .await
            .map_err(map_document_version_error)
    }
}

#[derive(Clone)]
pub(crate) struct HostedBrowserService {
    runtime: KnowledgebaseRuntime,
}

impl HostedBrowserService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }
}

#[async_trait]
impl KnowledgeBrowserApi for HostedBrowserService {
    async fn list_browser(
        &self,
        context: KnowledgeAppRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        let actor_id = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let service = KnowledgeBrowserService::new(
            self.runtime.space_store(),
            self.runtime.drive_tree(),
            self.runtime.browser_projection_store(),
        )
        .with_access_control(self.runtime.access_control());
        service
            .list(
                Some(KnowledgeBrowserAccessContext {
                    tenant_id: context.tenant_id,
                    actor_id,
                }),
                request,
            )
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub(crate) struct HostedOkfService {
    runtime: KnowledgebaseRuntime,
}

impl HostedOkfService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    async fn resolve_okf_space(&self) -> ApiResult<KnowledgeSpace> {
        self.runtime
            .space_store()
            .find_first_okf_bundle_initialized_space()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| {
                ApiError::not_found(
                    "okf_bundle_not_initialized",
                    okf_bundle_not_initialized_detail(),
                )
            })
    }
}

#[async_trait]
impl KnowledgeOkfAppService for HostedOkfService {
    async fn list_okf_concepts(&self, space_id: u64) -> ApiResult<OkfConceptSummaryList> {
        if space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_list_request",
                "space_id is required",
            ));
        }
        self.runtime
            .resolve_okf_bundle_engine_for_space(space_id)
            .await?;
        let items = self
            .runtime
            .knowledge_engines()
            .list_okf_concepts(space_id)
            .await
            .map_err(ApiError::from)?;
        Ok(OkfConceptSummaryList { items })
    }

    async fn retrieve_okf_concept(&self, concept_row_id: u64) -> ApiResult<OkfConceptSummary> {
        let page = self
            .runtime
            .okf_concept_store()
            .get_concept_by_row_id(concept_row_id)
            .await
            .map_err(map_okf_concept_store_error)?;
        Ok(concept_to_summary(page))
    }

    async fn list_okf_concept_revisions(
        &self,
        concept_row_id: u64,
    ) -> ApiResult<KnowledgeOkfConceptRevisionList> {
        let items = self
            .runtime
            .okf_concept_store()
            .list_concept_revisions(concept_row_id)
            .await
            .map_err(map_okf_concept_store_error)?;
        Ok(KnowledgeOkfConceptRevisionList { items })
    }

    async fn upsert_okf_concept(
        &self,
        request: OkfConceptUpsertRequest,
    ) -> ApiResult<OkfConceptSummary> {
        if is_blank(Some(request.concept_id.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "concept_id is required",
            ));
        }
        if is_blank(Some(request.markdown.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "markdown is required",
            ));
        }

        let space = self.resolve_okf_space().await?;
        if request.space_id != 0 && request.space_id != space.id {
            return Err(ApiError::invalid_request(
                "invalid_okf_concept_upsert_request",
                "space_id does not match the active OKF knowledge space",
            ));
        }

        let actor = if is_blank(Some(request.actor.as_str())) {
            self.runtime.operator_id().to_string()
        } else {
            request.actor
        };
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let concept = self
            .runtime
            .knowledge_engines()
            .upsert_okf_concept(OkfConceptUpsertRequest {
                space_id: space.id,
                concept_id: request.concept_id,
                markdown: request.markdown,
                actor,
                publish: request.publish,
            })
            .await
            .map_err(ApiError::from)?;
        Ok(concept_to_summary(concept))
    }

    async fn delete_okf_concept(
        &self,
        context: KnowledgeAppRequestContext,
        concept_row_id: u64,
    ) -> ApiResult<()> {
        let space = require_okf_concept_space_access(&self.runtime, &context, concept_row_id).await?;
        let actor = context
            .actor_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| self.runtime.operator_id().to_string());
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        self.runtime
            .knowledge_engines()
            .delete_okf_concept(space.id, concept_row_id, &actor)
            .await
            .map_err(ApiError::from)?;
        Ok(())
    }

    async fn retrieve_okf_index(&self) -> ApiResult<OkfIndexDocument> {
        let space = self.resolve_okf_space().await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.index_md,
            "bundle_index",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfIndexDocument { markdown })
    }

    async fn retrieve_okf_log(&self) -> ApiResult<OkfLogDocument> {
        let space = self.resolve_okf_space().await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.log_md,
            "bundle_log",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfLogDocument { markdown })
    }

    async fn retrieve_okf_schema(&self) -> ApiResult<OkfProfileDocument> {
        let space = self.resolve_okf_space().await?;
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let paths = okf_paths();
        let agents_markdown = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.agents_md,
            "bundle_profile",
            space.drive_space_id.as_deref(),
        )
        .await?;
        let profile_yaml = read_managed_okf_text(
            self.runtime.drive_storage(),
            paths.profile_yaml,
            "bundle_profile",
            space.drive_space_id.as_deref(),
        )
        .await?;
        Ok(OkfProfileDocument {
            agents_markdown,
            profile_yaml,
        })
    }

    async fn create_okf_query(&self, request: OkfQueryRequest) -> ApiResult<OkfQueryResult> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_query_request",
                "space_id is required",
            ));
        }
        if is_blank(Some(request.query.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_query_request",
                "query is required",
            ));
        }

        let search = self
            .runtime
            .search_knowledge_engine_for_space(request.space_id, &request.query, 8)
            .await
            .map_err(|error| ApiError::internal("okf_engine_search_failed", error))?;

        Ok(OkfQueryResult {
            answer_markdown: format_okf_engine_answer(&search.hits),
            trace_id: Some(format!(
                "{}:{}",
                search.implementation_id,
                stable_u64_hash(&request.query)
            )),
        })
    }

    async fn file_okf_query_answer(
        &self,
        query_id: u64,
        request: OkfFileAnswerRequest,
    ) -> ApiResult<OkfQueryResult> {
        if is_blank(Some(request.title.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_file_answer_request",
                "title is required",
            ));
        }
        if is_blank(Some(request.answer_markdown.as_str())) {
            return Err(ApiError::invalid_request(
                "invalid_okf_file_answer_request",
                "answer_markdown is required",
            ));
        }

        let space = self.resolve_okf_space().await?;
        let concept_id = okf_answer_concept_id(query_id);
        let description = request.title.clone();
        self.runtime
            .resolve_okf_bundle_engine_for_space(space.id)
            .await?;
        let publication = self
            .runtime
            .knowledge_engines()
            .publish_okf_concept(PublishKnowledgeOkfConceptRequest {
                space_id: space.id,
                concept_id,
                title: request.title,
                concept_type: "Answer".to_string(),
                description,
                markdown: request.answer_markdown.clone(),
                source_count: 1,
                tags: vec!["okf-query".to_string()],
                actor: self.runtime.operator_id().to_string(),
                resource: None,
                timestamp: None,
            })
            .await
            .map_err(ApiError::from)?;

        Ok(OkfQueryResult {
            answer_markdown: request.answer_markdown,
            trace_id: Some(publication.concept.id.to_string()),
        })
    }

    async fn create_okf_context_pack(
        &self,
        request: OkfContextPackRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_okf_context_pack_request",
                "space_id is required",
            ));
        }

        let query = request.query.unwrap_or_default();
        let context_pack =
            build_okf_context_pack_from_engine(&self.runtime, request.space_id, query, 4096)
                .await?;

        let body = serde_json::to_vec_pretty(&context_pack).map_err(|error| {
            ApiError::internal("okf_context_pack_serialization_failed", error.to_string())
        })?;
        let logical_path = format!("context_packs/cp-{}.json", context_pack.context_pack_id);
        let space = self.runtime.space_store().get_space(request.space_id).await?;
        let object_ref = self
            .runtime
            .drive_storage()
            .put_object(
                PutKnowledgeObjectRequest {
                    logical_path: logical_path.clone(),
                    object_role: "context_pack".to_string(),
                    content_type: "application/json; charset=utf-8".to_string(),
                    body,
                    checksum_sha256_hex: None,
                    space_uuid: None,
                }
                .with_drive_space_id(space.drive_space_id.as_deref()),
            )
            .await?;

        self.runtime
            .okf_bundle_file_store()
            .create_file_entry(CreateKnowledgeOkfBundleFileRecord {
                space_id: request.space_id,
                logical_path,
                file_kind: OkfBundleFileKind::ContextPack,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(map_okf_bundle_file_error)
    }

    async fn create_okf_export(
        &self,
        request: OkfBundleExportRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        create_okf_bundle_export(&self.runtime, request).await
    }

    async fn retrieve_okf_export(&self, export_id: u64) -> ApiResult<KnowledgeOkfBundleFile> {
        retrieve_okf_bundle_export(&self.runtime, export_id).await
    }

    async fn create_okf_import(
        &self,
        request: OkfBundleImportRequest,
    ) -> ApiResult<OkfBundleImportResult> {
        let actor = self.runtime.operator_id().to_string();
        create_okf_bundle_import(&self.runtime, request, &actor).await
    }

    async fn create_okf_lint_run(&self, request: OkfQualityRunRequest) -> ApiResult<OkfQualityRun> {
        create_okf_lint_run(&self.runtime, request).await
    }
}

fn map_document_version_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::KnowledgeDocumentVersionStoreError,
) -> ApiError {
    ApiError::internal("knowledge_document_version_store_failed", error.to_string())
}

async fn resolve_drive_import_request(
    drive_tree: &dyn KnowledgeDriveNodeTree,
    mut request: KnowledgeDriveImportRequest,
) -> ApiResult<KnowledgeDriveImportRequest> {
    let needs_locator = is_blank(Some(request.drive_object_key.as_str()))
        || is_blank(Some(request.drive_bucket.as_str()))
        || is_blank(Some(request.drive_storage_provider_id.as_str()));
    if !needs_locator {
        return Ok(request);
    }

    let Some(drive_node_id) = request
        .drive_node_id
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Err(ApiError::invalid_request(
            "invalid_drive_import_request",
            "drive_storage_provider_id, drive_bucket, and drive_object_key are required when drive_node_id is absent".to_string(),
        ));
    };
    let drive_space_id = request.drive_space_id.clone().ok_or_else(|| {
        ApiError::invalid_request(
            "invalid_drive_import_request",
            "drive_space_id is required when resolving drive object locator from drive_node_id"
                .to_string(),
        )
    })?;

    let node = drive_tree
        .get_node(GetKnowledgeDriveNodeRequest {
            drive_space_id,
            drive_node_id,
        })
        .await
        .map_err(|error| {
            ApiError::internal("knowledge_drive_node_tree_failed", error.to_string())
        })?
        .ok_or_else(|| {
            ApiError::not_found(
                "drive_node_not_found",
                "drive node was not found for import resolution".to_string(),
            )
        })?;
    let locator = node.object_locator.ok_or_else(|| {
        ApiError::invalid_request(
            "drive_object_locator_missing",
            "active drive object locator is not available for the requested node".to_string(),
        )
    })?;

    if is_blank(Some(request.drive_storage_provider_id.as_str())) {
        request.drive_storage_provider_id = locator.storage_provider_id;
    }
    if is_blank(Some(request.drive_bucket.as_str())) {
        request.drive_bucket = locator.bucket;
    }
    if is_blank(Some(request.drive_object_key.as_str())) {
        request.drive_object_key = locator.object_key;
    }
    Ok(request)
}

pub(crate) fn map_okf_concept_store_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> ApiError {
    let detail = error.to_string();
    if detail.contains("missing okf concept") {
        ApiError::not_found("okf_concept_not_found", detail)
    } else {
        ApiError::internal("knowledge_okf_concept_store_failed", detail)
    }
}

fn map_okf_bundle_file_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStoreError,
) -> ApiError {
    ApiError::internal("knowledge_okf_bundle_file_store_failed", error.to_string())
}
