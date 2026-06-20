use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::okf::{
    discover_bundle_files_from_directory, load_import_bundle_from_drive, stackoverflow_bundle_root,
    stage_export_bundle_for_drive_import, ExportOkfBundleRequest, ImportOkfBundleRequest,
    OkfBundleExporterService, OkfBundleImporterService, OkfBundleStandardFileService,
    OkfConceptService, PersistStandardFilesRequest, PublishExistingOkfConceptRevisionRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_bundle_file_store::{
    CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
    KnowledgeOkfBundleFileStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateListItem, KnowledgeOkfCandidateStore, KnowledgeOkfCandidateStoreError,
    UpsertKnowledgeOkfCandidateRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_link_store::{
    KnowledgeOkfConceptLinkRecord, KnowledgeOkfConceptLinkStore, KnowledgeOkfConceptLinkStoreError,
    ReplaceKnowledgeOkfConceptLinksRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
    KnowledgeOkfConceptProjection, KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
    MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptPublishState, OkfConceptSummary,
    OkfLogEntry, OkfLogEventType, OkfRevisionReviewState, PublishKnowledgeOkfConceptRequest,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;

#[tokio::test]
async fn okf_concept_service_publishes_concept_and_rebuilds_standard_files() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    let publication = service
        .publish_concept(
            PublishKnowledgeOkfConceptRequest {
                space_id: 7,
                concept_id: "entities/entity-name".to_string(),
                title: "Entity Name".to_string(),
                concept_type: "Entity".to_string(),
                description: "Entity summary.".to_string(),
                markdown: "# Entity Name\n\nA durable synthesis.".to_string(),
                source_count: 2,
                tags: vec!["entity".to_string()],
                actor: "system".to_string(),
                resource: None,
                timestamp: None,
            },
            Some("drv-kb-001"),
        )
        .await
        .unwrap();

    assert_eq!(
        publication.published_logical_path,
        "okf/entities/entity-name.md"
    );
    assert_eq!(
        publication.governance_revision_path,
        ".sdkwork/governance/revisions/entities/entity-name/r1.md"
    );
    assert_eq!(
        publication.concept.publish_state,
        OkfConceptPublishState::Published
    );
    assert_eq!(
        publication.revision.review_state,
        OkfRevisionReviewState::Approved
    );

    let published_body = drive.body_at("okf/entities/entity-name.md").unwrap();
    assert!(published_body.starts_with("---\n"));
    assert!(published_body.contains("type: Entity"));
    assert!(published_body.contains("title: \"Entity Name\""));
    assert!(published_body.contains("description: \"Entity summary.\""));
    assert!(published_body.contains("# Entity Name\n\nA durable synthesis."));
    assert!(object_refs
        .ref_by_path("okf/entities/entity-name.md")
        .is_some());
    assert!(object_refs
        .ref_by_path(".sdkwork/governance/revisions/entities/entity-name/r1.md")
        .is_some());

    assert!(file_entries.paths().contains(&"okf/index.md".to_string()));
    assert!(file_entries.paths().contains(&"okf/log.md".to_string()));
    assert!(workspace
        .paths()
        .contains(&"okf/entities/entity-name.md".to_string()));
    assert!(workspace.paths().contains(&"okf/index.md".to_string()));
    assert!(workspace.paths().contains(&"okf/log.md".to_string()));

    let index_ref = file_entries.object_key_for("okf/index.md").unwrap();
    let index_content = drive.body_at(&index_ref).unwrap();
    assert!(index_content.contains("okf_version: \"0.1\""));
    assert!(index_content.contains("* [Entity Name](entities/entity-name.md)"));

    let log_ref = file_entries.object_key_for("okf/log.md").unwrap();
    let log_content = drive.body_at(&log_ref).unwrap();
    assert!(log_content.contains("* **Publish**: Published Entity Name"));
}

#[tokio::test]
async fn stackoverflow_bundle_imports_as_candidates_by_default() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let candidates = MemoryCandidateStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_candidate_store(&candidates)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);
    let importer = OkfBundleImporterService::new(service);

    let bundle_root = stackoverflow_bundle_root();
    assert!(
        bundle_root.exists(),
        "stackoverflow bundle fixture must exist"
    );
    let files = discover_bundle_files_from_directory(&bundle_root).expect("bundle walk");

    let result = importer
        .import_bundle(
            ImportOkfBundleRequest {
                space_id: 42,
                actor: "compliance-test".to_string(),
                publish: false,
                files,
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("stackoverflow bundle import");

    assert!(result.imported_concept_count >= 1);
    assert!(drive.body_at("okf/tables/users.md").is_none());
    assert!(candidates.count() >= 1);
    assert_eq!(
        result.publications[0].concept.publish_state,
        OkfConceptPublishState::CandidateReady
    );
}

#[tokio::test]
async fn stackoverflow_bundle_imports_published_concepts_when_requested() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let links = MemoryLinkStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_link_store(&links)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);
    let importer = OkfBundleImporterService::new(service);

    let bundle_root = stackoverflow_bundle_root();
    let files = discover_bundle_files_from_directory(&bundle_root).expect("bundle walk");
    let expected_concept_count = files
        .iter()
        .filter(|file| {
            let path = file.bundle_relative_path.replace('\\', "/");
            path.ends_with(".md")
                && path != "index.md"
                && path != "log.md"
                && !path.ends_with("/index.md")
        })
        .count();

    let result = importer
        .import_bundle(
            ImportOkfBundleRequest {
                space_id: 42,
                actor: "compliance-test".to_string(),
                publish: true,
                files,
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("stackoverflow bundle import");

    assert!(result.imported_concept_count >= 1);
    assert!(drive.body_at("okf/tables/users.md").is_some());
    assert!(file_entries.paths().contains(&"okf/index.md".to_string()));

    let report = sdkwork_intelligence_knowledgebase_service::okf::OkfBundleLinterService::new(
        &drive, &concepts,
    )
    .with_link_store(&links)
    .lint_space(42)
    .await
    .expect("stackoverflow bundle lint");
    assert!(
        report.conformance_passed(),
        "expected OKF conformance pass, issues: {:?}",
        report.issues
    );
    let published = concepts
        .list_concept_summaries(42)
        .await
        .expect("list published concepts");
    assert_eq!(
        published.len(),
        expected_concept_count,
        "published concept count should match bundle concept files"
    );
    assert!(
        expected_concept_count >= 40,
        "stackoverflow fixture should include many concepts"
    );
}

#[tokio::test]
async fn stackoverflow_published_bundle_lints_without_stale_claims_when_sources_fresh() {
    use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
        CreateKnowledgeSourceRecord, KnowledgeSourceLineageSnapshot, KnowledgeSourceStore,
        KnowledgeSourceStoreError,
    };

    struct MemorySourceStore {
        lineage: Vec<KnowledgeSourceLineageSnapshot>,
    }

    #[async_trait::async_trait]
    impl KnowledgeSourceStore for MemorySourceStore {
        async fn create_source(
            &self,
            _record: CreateKnowledgeSourceRecord,
        ) -> Result<
            sdkwork_knowledgebase_contract::source::KnowledgeSource,
            KnowledgeSourceStoreError,
        > {
            Err(KnowledgeSourceStoreError::Internal(
                "not used in lint test".to_string(),
            ))
        }

        async fn list_space_source_lineage(
            &self,
            _space_id: u64,
        ) -> Result<Vec<KnowledgeSourceLineageSnapshot>, KnowledgeSourceStoreError> {
            Ok(self.lineage.clone())
        }
    }

    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let links = MemoryLinkStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let sources = MemorySourceStore {
        lineage: vec![KnowledgeSourceLineageSnapshot {
            source_id: 1,
            updated_at: "2020-01-01T00:00:00Z".to_string(),
            last_sync_at: None,
            provider: Some("stackoverflow".to_string()),
            drive_prefix: Some("sources/raw/stackoverflow".to_string()),
            drive_bucket: None,
        }],
    };
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_link_store(&links)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);
    let importer = OkfBundleImporterService::new(service);

    let bundle_root = stackoverflow_bundle_root();
    let files = discover_bundle_files_from_directory(&bundle_root).expect("bundle walk");
    importer
        .import_bundle(
            ImportOkfBundleRequest {
                space_id: 42,
                actor: "compliance-test".to_string(),
                publish: true,
                files,
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("stackoverflow bundle import");

    let report = sdkwork_intelligence_knowledgebase_service::okf::OkfBundleLinterService::new(
        &drive, &concepts,
    )
    .with_link_store(&links)
    .with_source_store(&sources)
    .lint_space(42)
    .await
    .expect("stackoverflow bundle lint with sources");
    assert!(
        report
            .issues
            .iter()
            .all(|issue| issue.check != "stale_claims"),
        "fresh kb_source lineage should not produce stale_claims warnings: {:?}",
        report.issues
    );
}

#[tokio::test]
async fn export_bundle_round_trips_through_drive_import_inbox() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let source_concepts = MemoryOkfConceptStore::default();
    let target_concepts = MemoryOkfConceptStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let source_service = OkfConceptService::new(&drive, &object_refs, &source_concepts)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    source_service
        .publish_concept(
            PublishKnowledgeOkfConceptRequest {
                space_id: 7,
                concept_id: "entities/widget".to_string(),
                title: "Widget".to_string(),
                concept_type: "Entity".to_string(),
                description: "Widget summary.".to_string(),
                markdown: "# Widget\n\nA durable widget.".to_string(),
                source_count: 0,
                tags: vec!["entity".to_string()],
                actor: "author".to_string(),
                resource: None,
                timestamp: None,
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("publish source concept");

    let summaries = source_concepts
        .list_concept_summaries(7)
        .await
        .expect("list source concepts");
    OkfBundleStandardFileService::new(&drive)
        .persist_standard_files(PersistStandardFilesRequest {
            space_name: "Space Seven".to_string(),
            concepts: summaries,
            log_entries: vec![],
        })
        .await
        .expect("persist standard bundle files");

    let exported = OkfBundleExporterService::new(&drive, &source_concepts)
        .export_bundle(ExportOkfBundleRequest {
            space_id: 7,
            export_type: "okf_strict".to_string(),
        })
        .await
        .expect("export okf bundle");

    stage_export_bundle_for_drive_import(&drive, &exported.export_root, 99, "roundtrip")
        .await
        .expect("stage export for import");

    let files = load_import_bundle_from_drive(&drive, 99, Some("roundtrip"))
        .await
        .expect("load staged import bundle");
    let target_service = OkfConceptService::new(&drive, &object_refs, &target_concepts)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);
    let importer = OkfBundleImporterService::new(target_service);
    let result = importer
        .import_bundle(
            ImportOkfBundleRequest {
                space_id: 99,
                actor: "roundtrip".to_string(),
                publish: true,
                files,
            },
            Some("drv-kb-002"),
        )
        .await
        .expect("import staged bundle");

    assert!(result.imported_concept_count >= 1);
    assert_eq!(target_concepts.concept_count(), 1);
    assert!(drive.body_at("okf/entities/widget.md").is_some());
}

#[tokio::test]
async fn publish_existing_revision_projects_governance_markdown_to_bundle() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    let staged = service
        .upsert_concept_from_markdown(
            sdkwork_knowledgebase_contract::okf::OkfConceptUpsertRequest {
                space_id: 7,
                concept_id: "entities/entity-name".to_string(),
                markdown: r#"---
type: Entity
title: Entity Name
description: Entity summary.
tags: [entity]
---
# Entity Name

A durable synthesis."#
                    .to_string(),
                actor: "author".to_string(),
                publish: false,
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("draft upsert");

    assert_eq!(
        staged.concept.publish_state,
        OkfConceptPublishState::CandidateReady
    );
    assert!(drive.body_at("okf/entities/entity-name.md").is_none());

    service
        .publish_existing_revision(
            PublishExistingOkfConceptRevisionRequest {
                space_id: 7,
                concept: staged.concept.clone(),
                revision: staged.revision,
                actor: "reviewer".to_string(),
            },
            Some("drv-kb-001"),
        )
        .await
        .expect("publish existing revision");

    assert!(drive.body_at("okf/entities/entity-name.md").is_some());
    assert!(file_entries.paths().contains(&"okf/index.md".to_string()));
}

#[tokio::test]
async fn okf_concept_service_requires_drive_space_when_workspace_enabled() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let concepts = MemoryOkfConceptStore::default();
    let file_entries = MemoryOkfBundleFileStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = OkfConceptService::new(&drive, &object_refs, &concepts)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    let error = service
        .publish_concept(
            PublishKnowledgeOkfConceptRequest {
                space_id: 7,
                concept_id: "entities/entity-name".to_string(),
                title: "Entity Name".to_string(),
                concept_type: "Entity".to_string(),
                description: "Entity summary.".to_string(),
                markdown: "# Entity Name\n\nA durable synthesis.".to_string(),
                source_count: 2,
                tags: vec!["entity".to_string()],
                actor: "system".to_string(),
                resource: None,
                timestamp: None,
            },
            None,
        )
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("drive_space_id is required when drive workspace synchronization is enabled"));
    assert_eq!(drive.object_count(), 0);
    assert_eq!(object_refs.ref_count(), 0);
    assert_eq!(concepts.concept_count(), 0);
    assert_eq!(concepts.revision_count(), 0);
    assert_eq!(concepts.log_count(), 0);
    assert!(file_entries.paths().is_empty());
    assert!(workspace.paths().is_empty());
}

#[derive(Default)]
struct MemoryDrive {
    objects: Mutex<HashMap<String, (KnowledgeObjectRef, Vec<u8>)>>,
}

impl MemoryDrive {
    fn body_at(&self, logical_path: &str) -> Option<String> {
        self.objects
            .lock()
            .unwrap()
            .get(logical_path)
            .map(|(_, body)| String::from_utf8_lossy(body).into_owned())
    }

    fn object_count(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDriveStorage for MemoryDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let digest = Sha256::digest(&request.body);
        let checksum = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let object_ref = KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "test".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path.clone(),
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: Some(checksum),
            etag: None,
            version_id: None,
        };
        self.objects
            .lock()
            .unwrap()
            .insert(request.logical_path, (object_ref.clone(), request.body));
        Ok(object_ref)
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let logical_path = request
            .logical_path
            .as_deref()
            .ok_or_else(|| KnowledgeStorageError::internal("missing logical_path"))?;
        self.objects
            .lock()
            .unwrap()
            .get(logical_path)
            .map(|(object_ref, _)| object_ref.clone())
            .ok_or_else(|| KnowledgeStorageError::internal("missing object"))
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.body_at(&object_ref.logical_path)
            .ok_or_else(|| KnowledgeStorageError::internal("missing object"))
    }
}

#[derive(Default)]
struct MemoryObjectRefStore {
    next_id: Mutex<u64>,
    refs: Mutex<Vec<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef>>,
}

impl MemoryObjectRefStore {
    fn ref_by_path(
        &self,
        logical_path: &str,
    ) -> Option<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef> {
        self.refs
            .lock()
            .unwrap()
            .iter()
            .find(|object_ref| object_ref.logical_path.as_deref() == Some(logical_path))
            .cloned()
    }

    fn ref_count(&self) -> usize {
        self.refs.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDriveObjectRefStore for MemoryObjectRefStore {
    async fn create_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef,
        KnowledgeDriveObjectRefStoreError,
    > {
        self.create_or_get_object_ref(record).await
    }

    async fn create_or_get_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef,
        KnowledgeDriveObjectRefStoreError,
    > {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let object_ref = sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef {
            id: *next_id,
            space_id: record.space_id,
            drive_space_id: record.drive_space_id,
            drive_node_id: record.drive_node_id,
            logical_path: record.logical_path,
            drive_provider_kind: record.drive_provider_kind,
            drive_storage_provider_id: record.drive_storage_provider_id,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            drive_object_version: record.drive_object_version,
            drive_etag: record.drive_etag,
            content_type: record.content_type,
            size_bytes: record.size_bytes,
            checksum_sha256_hex: record.checksum_sha256_hex,
            object_role: record.object_role,
            access_mode: record.access_mode,
        };
        self.refs.lock().unwrap().push(object_ref.clone());
        Ok(object_ref)
    }

    async fn list_object_refs_by_logical_path_prefix(
        &self,
        space_id: u64,
        prefix: &str,
    ) -> Result<
        Vec<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef>,
        KnowledgeDriveObjectRefStoreError,
    > {
        Ok(self
            .refs
            .lock()
            .unwrap()
            .iter()
            .filter(|object_ref| {
                object_ref.space_id == space_id
                    && object_ref
                        .logical_path
                        .as_deref()
                        .is_some_and(|path| path.starts_with(prefix))
            })
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct MemoryOkfConceptStore {
    next_concept_id: Mutex<u64>,
    next_revision_id: Mutex<u64>,
    concepts: Mutex<Vec<KnowledgeOkfConcept>>,
    revisions: Mutex<Vec<KnowledgeOkfConceptRevision>>,
    logs: Mutex<Vec<OkfLogEntry>>,
}

impl MemoryOkfConceptStore {
    fn concept_count(&self) -> usize {
        self.concepts.lock().unwrap().len()
    }

    fn revision_count(&self) -> usize {
        self.revisions.lock().unwrap().len()
    }

    fn log_count(&self) -> usize {
        self.logs.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeOkfConceptStore for MemoryOkfConceptStore {
    async fn upsert_concept(
        &self,
        record: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        let mut concepts = self.concepts.lock().unwrap();
        if let Some(concept) = concepts.iter_mut().find(|concept| {
            concept.space_id == record.space_id && concept.concept_id == record.concept_id
        }) {
            concept.title = record.title;
            concept.concept_type = record.concept_type;
            concept.logical_path = record.logical_path.clone();
            concept.bundle_relative_path = record
                .logical_path
                .strip_prefix("okf/")
                .unwrap_or(&record.logical_path)
                .to_string();
            concept.description = record.description;
            concept.source_count = record.source_count;
            concept.tags = record.tags;
            concept.publish_state = record.publish_state;
            return Ok(concept.clone());
        }
        let mut next_concept_id = self.next_concept_id.lock().unwrap();
        *next_concept_id += 1;
        let bundle_relative_path = record
            .logical_path
            .strip_prefix("okf/")
            .unwrap_or(&record.logical_path)
            .to_string();
        let concept = KnowledgeOkfConcept {
            id: *next_concept_id,
            space_id: record.space_id,
            concept_id: record.concept_id,
            title: record.title,
            concept_type: record.concept_type,
            logical_path: record.logical_path,
            bundle_relative_path,
            description: record.description,
            source_count: record.source_count,
            tags: record.tags,
            current_revision_id: None,
            publish_state: record.publish_state,
            updated_at: "2026-06-04T12:00:00Z".to_string(),
        };
        concepts.push(concept.clone());
        Ok(concept)
    }

    async fn create_revision(
        &self,
        record: CreateKnowledgeOkfConceptRevisionRecord,
    ) -> Result<KnowledgeOkfConceptRevision, KnowledgeOkfConceptStoreError> {
        let mut next_revision_id = self.next_revision_id.lock().unwrap();
        *next_revision_id += 1;
        let revision = KnowledgeOkfConceptRevision {
            id: *next_revision_id,
            concept_row_id: record.concept_row_id,
            revision_no: record.revision_no,
            markdown_object_ref_id: record.markdown_object_ref_id,
            content_hash: record.content_hash,
            review_state: record.review_state,
            created_at: "2026-06-04T12:00:00Z".to_string(),
        };
        self.revisions.lock().unwrap().push(revision.clone());
        Ok(revision)
    }

    async fn next_revision_no(
        &self,
        concept_row_id: u64,
    ) -> Result<u64, KnowledgeOkfConceptStoreError> {
        let revisions = self.revisions.lock().unwrap();
        let max_revision = revisions
            .iter()
            .filter(|revision| revision.concept_row_id == concept_row_id)
            .map(|revision| revision.revision_no)
            .max()
            .unwrap_or(0);
        Ok(max_revision + 1)
    }

    async fn mark_current_revision(
        &self,
        record: MarkKnowledgeOkfConceptCurrentRevisionRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        let mut concepts = self.concepts.lock().unwrap();
        let concept = concepts
            .iter_mut()
            .find(|concept| concept.id == record.concept_row_id)
            .ok_or_else(|| {
                KnowledgeOkfConceptStoreError::Internal("missing concept".to_string())
            })?;
        concept.current_revision_id = Some(record.revision_id);
        concept.publish_state = record.publish_state;
        Ok(concept.clone())
    }

    async fn list_concept_summaries(
        &self,
        space_id: u64,
    ) -> Result<Vec<OkfConceptSummary>, KnowledgeOkfConceptStoreError> {
        Ok(self
            .concepts
            .lock()
            .unwrap()
            .iter()
            .filter(|concept| {
                concept.space_id == space_id
                    && concept.publish_state == OkfConceptPublishState::Published
            })
            .map(|concept| OkfConceptSummary {
                title: concept.title.clone(),
                concept_id: concept.concept_id.clone(),
                concept_type: concept.concept_type.clone(),
                logical_path: concept.logical_path.clone(),
                bundle_relative_path: concept.bundle_relative_path.clone(),
                description: concept.description.clone(),
                source_count: concept.source_count,
                updated_at: concept.updated_at.clone(),
                tags: concept.tags.clone(),
            })
            .collect())
    }

    async fn append_log_entry(
        &self,
        record: AppendKnowledgeOkfLogEntryRecord,
    ) -> Result<OkfLogEntry, KnowledgeOkfConceptStoreError> {
        let entry = OkfLogEntry {
            occurred_at: record.event_time,
            event_type: OkfLogEventType::Publish,
            title: record.title,
            actor: record.actor,
            affected_concepts: record.affected_concepts,
            audit_event_id: record.audit_event_id,
            warnings: record.warnings,
        };
        self.logs.lock().unwrap().push(entry.clone());
        Ok(entry)
    }

    async fn list_log_entries(
        &self,
        _space_id: u64,
    ) -> Result<Vec<OkfLogEntry>, KnowledgeOkfConceptStoreError> {
        Ok(self.logs.lock().unwrap().clone())
    }

    async fn batch_concept_projections_by_paths(
        &self,
        _space_id: u64,
        _logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeOkfConceptProjection>, KnowledgeOkfConceptStoreError> {
        Ok(vec![])
    }
}

#[derive(Default)]
struct MemoryOkfBundleFileStore {
    entries: Mutex<Vec<CreateKnowledgeOkfBundleFileRecord>>,
}

impl MemoryOkfBundleFileStore {
    fn paths(&self) -> Vec<String> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|entry| entry.logical_path.clone())
            .collect()
    }

    fn object_key_for(&self, logical_path: &str) -> Option<String> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .find(|entry| entry.logical_path == logical_path)
            .map(|entry| entry.drive_object_key.clone())
    }
}

#[async_trait]
impl KnowledgeOkfBundleFileStore for MemoryOkfBundleFileStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeOkfBundleFile,
        KnowledgeOkfBundleFileStoreError,
    > {
        self.upsert_file_entry(record).await
    }

    async fn upsert_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeOkfBundleFile,
        KnowledgeOkfBundleFileStoreError,
    > {
        self.entries.lock().unwrap().push(record.clone());
        Ok(sdkwork_knowledgebase_contract::KnowledgeOkfBundleFile {
            id: self.entries.lock().unwrap().len() as u64,
            space_id: record.space_id,
            logical_path: record.logical_path,
            file_kind: record.file_kind,
            artifact_role: record.artifact_role,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            checksum_sha256_hex: record.checksum_sha256_hex,
            staged_import_root: None,
            import_id: None,
        })
    }
}

#[derive(Default)]
struct MemoryDriveWorkspace {
    paths: Mutex<Vec<String>>,
}

impl MemoryDriveWorkspace {
    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveWorkspace for MemoryDriveWorkspace {
    async fn ensure_nodes(
        &self,
        request: EnsureKnowledgeDriveNodesRequest,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        for node in request.nodes {
            self.paths.lock().unwrap().push(node.logical_path);
        }
        Ok(())
    }
}

#[derive(Default)]
struct MemoryCandidateStore {
    records: Mutex<Vec<UpsertKnowledgeOkfCandidateRecord>>,
}

impl MemoryCandidateStore {
    fn count(&self) -> usize {
        self.records.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeOkfCandidateStore for MemoryCandidateStore {
    async fn upsert_candidate(
        &self,
        record: UpsertKnowledgeOkfCandidateRecord,
    ) -> Result<(), KnowledgeOkfCandidateStoreError> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    async fn update_candidate_state_by_concept_row_id(
        &self,
        _concept_row_id: u64,
        _state: OkfConceptPublishState,
        _reviewer_id: Option<u64>,
        _review_note: Option<String>,
    ) -> Result<(), KnowledgeOkfCandidateStoreError> {
        Ok(())
    }

    async fn list_open_candidates(
        &self,
        _space_id: Option<u64>,
    ) -> Result<Vec<KnowledgeOkfCandidateListItem>, KnowledgeOkfCandidateStoreError> {
        Ok(vec![])
    }
}

#[derive(Default)]
struct MemoryLinkStore {
    outbound: Mutex<HashMap<(u64, String), Vec<KnowledgeOkfConceptLinkRecord>>>,
}

#[async_trait]
impl KnowledgeOkfConceptLinkStore for MemoryLinkStore {
    async fn replace_outbound_links(
        &self,
        record: ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), KnowledgeOkfConceptLinkStoreError> {
        self.outbound
            .lock()
            .unwrap()
            .insert((record.space_id, record.from_concept_id), record.links);
        Ok(())
    }

    async fn list_inbound_concept_ids(
        &self,
        space_id: u64,
        to_concept_id: &str,
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        let inbound = self
            .outbound
            .lock()
            .unwrap()
            .iter()
            .filter_map(|((link_space_id, from_concept_id), links)| {
                if *link_space_id != space_id {
                    return None;
                }
                links
                    .iter()
                    .any(|link| link.to_concept_id == to_concept_id)
                    .then(|| from_concept_id.clone())
            })
            .collect();
        Ok(inbound)
    }

    async fn list_orphan_concept_ids(
        &self,
        space_id: u64,
        published_concept_ids: &[String],
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        let inbound: BTreeSet<String> = self
            .outbound
            .lock()
            .unwrap()
            .iter()
            .filter(|((link_space_id, _), _)| *link_space_id == space_id)
            .flat_map(|(_, links)| links.iter().map(|link| link.to_concept_id.clone()))
            .collect();
        Ok(published_concept_ids
            .iter()
            .filter(|concept_id| !inbound.contains(*concept_id))
            .cloned()
            .collect())
    }
}
