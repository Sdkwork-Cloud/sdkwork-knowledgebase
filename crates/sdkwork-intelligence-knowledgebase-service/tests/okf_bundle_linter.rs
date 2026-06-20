use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::okf::{
    lint_bundle_summaries, lint_published_concept_markdown, render_okf_concept_markdown,
    to_contract_lint_result, OkfBundleLinterService, OkfConceptDocument,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_link_store::{
    KnowledgeOkfConceptLinkStore, KnowledgeOkfConceptLinkStoreError,
    ReplaceKnowledgeOkfConceptLinksRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
    KnowledgeOkfConceptProjection, KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
    MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptPublishState, OkfConceptSummary,
    OkfLogEntry,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[tokio::test]
async fn bundle_linter_reports_broken_links_and_orphans() {
    let drive = MemoryDrive::default();
    let concepts = MemoryOkfConceptStore::default();
    let links = MemoryLinkStore::default();

    let markdown = render_okf_concept_markdown(&OkfConceptDocument {
        concept_type: "Entity".to_string(),
        title: Some("Entity A".to_string()),
        description: Some("Summary.".to_string()),
        resource: None,
        tags: vec![],
        timestamp: None,
        body: "See [missing](entities/missing.md).".to_string(),
    });
    drive.put("okf/entities/a.md", &markdown).await;

    concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 1,
            concept_id: "entities/a".to_string(),
            title: "Entity A".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/a.md".to_string(),
            description: "Summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: OkfConceptPublishState::Published,
        })
        .await
        .unwrap();
    concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 1,
            concept_id: "entities/orphan".to_string(),
            title: "Orphan".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/orphan.md".to_string(),
            description: "Orphan summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: OkfConceptPublishState::Published,
        })
        .await
        .unwrap();
    drive
        .put(
            "okf/entities/orphan.md",
            &render_okf_concept_markdown(&OkfConceptDocument {
                concept_type: "Entity".to_string(),
                title: Some("Orphan".to_string()),
                description: Some("Orphan summary.".to_string()),
                resource: None,
                tags: vec![],
                timestamp: None,
                body: "Standalone.".to_string(),
            }),
        )
        .await;

    let report = OkfBundleLinterService::new(&drive, &concepts)
        .with_link_store(&links)
        .lint_space(1)
        .await
        .unwrap();
    let contract = to_contract_lint_result(&report);
    assert!(contract
        .issues
        .iter()
        .any(|issue| issue.code == "broken_links"));
    assert!(contract
        .issues
        .iter()
        .any(|issue| issue.code == "orphan_concepts"));
}

#[test]
fn lint_published_concept_detects_missing_frontmatter() {
    let issues = lint_published_concept_markdown("entities/a", "# No frontmatter\n", &[]);
    assert!(issues.iter().any(|issue| issue.check == "okf_conformance"));
}

#[test]
fn lint_bundle_summaries_flags_missing_description() {
    let report = lint_bundle_summaries(
        &[OkfConceptSummary {
            title: "Entity".to_string(),
            concept_id: "entities/a".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/a.md".to_string(),
            bundle_relative_path: "entities/a.md".to_string(),
            description: String::new(),
            source_count: 0,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec![],
        }],
        &[],
    );
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.check == "missing_citations"));
}

#[derive(Default)]
struct MemoryDrive {
    objects: Mutex<HashMap<String, String>>,
}

impl MemoryDrive {
    async fn put(&self, logical_path: &str, body: &str) {
        self.objects
            .lock()
            .unwrap()
            .insert(logical_path.to_string(), body.to_string());
    }
}

#[async_trait]
impl KnowledgeDriveStorage for MemoryDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let body = String::from_utf8_lossy(&request.body).into_owned();
        self.put(&request.logical_path, &body).await;
        Ok(KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "test".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: request.checksum_sha256_hex,
            etag: None,
            version_id: None,
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let logical_path = request
            .logical_path
            .as_deref()
            .ok_or_else(|| KnowledgeStorageError::internal("missing logical_path"))?;
        if !self.objects.lock().unwrap().contains_key(logical_path) {
            return Err(KnowledgeStorageError::internal("missing object"));
        }
        Ok(KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "test".to_string(),
            object_key: logical_path.to_string(),
            logical_path: logical_path.to_string(),
            object_role: request.object_role,
            content_type: "text/markdown; charset=utf-8".to_string(),
            size_bytes: 0,
            checksum_sha256_hex: None,
            etag: None,
            version_id: None,
        })
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.objects
            .lock()
            .unwrap()
            .get(&object_ref.logical_path)
            .cloned()
            .ok_or_else(|| KnowledgeStorageError::internal("missing object"))
    }
}

#[derive(Default)]
struct MemoryLinkStore;

#[async_trait]
impl KnowledgeOkfConceptLinkStore for MemoryLinkStore {
    async fn replace_outbound_links(
        &self,
        _record: ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), KnowledgeOkfConceptLinkStoreError> {
        Ok(())
    }

    async fn list_inbound_concept_ids(
        &self,
        _space_id: u64,
        _to_concept_id: &str,
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        Ok(vec![])
    }

    async fn list_orphan_concept_ids(
        &self,
        _space_id: u64,
        published_concept_ids: &[String],
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
        Ok(published_concept_ids
            .iter()
            .filter(|concept_id| *concept_id == "entities/orphan")
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct MemoryOkfConceptStore {
    concepts: Mutex<Vec<KnowledgeOkfConcept>>,
}

#[async_trait]
impl KnowledgeOkfConceptStore for MemoryOkfConceptStore {
    async fn upsert_concept(
        &self,
        record: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        let bundle_relative_path = record
            .logical_path
            .strip_prefix("okf/")
            .unwrap_or(&record.logical_path)
            .to_string();
        let concept = KnowledgeOkfConcept {
            id: self.concepts.lock().unwrap().len() as u64 + 1,
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
            updated_at: "2026-06-01T00:00:00Z".to_string(),
        };
        self.concepts.lock().unwrap().push(concept.clone());
        Ok(concept)
    }

    async fn create_revision(
        &self,
        _record: CreateKnowledgeOkfConceptRevisionRecord,
    ) -> Result<KnowledgeOkfConceptRevision, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "not implemented".to_string(),
        ))
    }

    async fn next_revision_no(
        &self,
        _concept_row_id: u64,
    ) -> Result<u64, KnowledgeOkfConceptStoreError> {
        Ok(1)
    }

    async fn mark_current_revision(
        &self,
        _record: MarkKnowledgeOkfConceptCurrentRevisionRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "not implemented".to_string(),
        ))
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
            .filter(|concept| concept.space_id == space_id)
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
        _record: AppendKnowledgeOkfLogEntryRecord,
    ) -> Result<OkfLogEntry, KnowledgeOkfConceptStoreError> {
        Err(KnowledgeOkfConceptStoreError::Internal(
            "not implemented".to_string(),
        ))
    }

    async fn list_log_entries(
        &self,
        _space_id: u64,
    ) -> Result<Vec<OkfLogEntry>, KnowledgeOkfConceptStoreError> {
        Ok(vec![])
    }

    async fn batch_concept_projections_by_paths(
        &self,
        _space_id: u64,
        _logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeOkfConceptProjection>, KnowledgeOkfConceptStoreError> {
        Ok(vec![])
    }
}
