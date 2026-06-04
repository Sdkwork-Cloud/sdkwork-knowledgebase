use sdkwork_knowledgebase_contract::wiki::{
    WikiLogEntry, WikiLogEventType, WikiPageSummary, WikiPageType,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::wiki::{
    render_agents_md, render_index_md, render_log_md, render_wiki_schema_yaml,
    LlmWikiStandardFileService, PersistStandardFilesRequest,
};
use std::sync::{Arc, Mutex};

#[test]
fn agents_md_names_llm_wiki_layers_and_drive_storage() {
    let content = render_agents_md("Research Space");

    assert!(content.contains("raw sources"));
    assert!(content.contains("wiki"));
    assert!(content.contains("schema"));
    assert!(content.contains("ingest"));
    assert!(content.contains("query"));
    assert!(content.contains("lint"));
    assert!(content.contains("sdkwork-drive"));
}

#[test]
fn wiki_schema_yaml_declares_llm_wiki_workflows_and_standard_paths() {
    let content = render_wiki_schema_yaml();

    assert!(content.contains("workflows:"));
    assert!(content.contains("ingest"));
    assert!(content.contains("query"));
    assert!(content.contains("lint"));
    assert!(content.contains("wiki/index.md"));
    assert!(content.contains("wiki/log.md"));
}

#[test]
fn index_md_renders_categories_and_wikilinks() {
    let pages = vec![WikiPageSummary {
        title: "Entity Name".to_string(),
        slug: "entity-name".to_string(),
        page_type: WikiPageType::Entity,
        logical_path: "wiki/pages/entities/entity-name/current.md".to_string(),
        summary: "One-line entity summary.".to_string(),
        source_count: 4,
        updated_at: "2026-06-01T00:00:00Z".to_string(),
        tags: vec!["entity".to_string()],
    }];

    let content = render_index_md("Research Space", &pages);

    assert!(content.contains("# Index"));
    assert!(content.contains("## Entities"));
    assert!(content.contains("[[entity-name|Entity Name]]"));
    assert!(content.contains("sources: 4"));
}

#[test]
fn log_md_uses_parseable_heading_prefix() {
    let entries = vec![WikiLogEntry {
        occurred_at: "2026-06-01T00:00:00Z".to_string(),
        event_type: WikiLogEventType::Ingest,
        title: "Source Title".to_string(),
        actor: "system".to_string(),
        affected_pages: vec!["Entity Name".to_string()],
        audit_event_id: Some("audit-1".to_string()),
        warnings: vec![],
    }];

    let content = render_log_md(&entries);

    assert!(content.contains("## [2026-06-01T00:00:00Z] ingest | Source Title"));
    assert!(content.contains("- actor: system"));
    assert!(content.contains("- auditEventId: audit-1"));
}

#[tokio::test]
async fn standard_files_are_persisted_through_drive_port() {
    let drive = RecordingDrive::default();
    let service = LlmWikiStandardFileService::new(&drive);

    let refs = service
        .persist_standard_files(PersistStandardFilesRequest {
            space_name: "Research Space".to_string(),
            pages: vec![],
            log_entries: vec![],
        })
        .await
        .unwrap();

    assert_eq!(refs.agents_md.logical_path, "wiki/schema/AGENTS.md");
    assert_eq!(
        refs.schema_yaml.logical_path,
        "wiki/schema/wiki_schema.yaml"
    );
    assert_eq!(refs.index_md.logical_path, "wiki/index.md");
    assert_eq!(refs.log_md.logical_path, "wiki/log.md");
    assert_eq!(
        drive.paths(),
        vec![
            "wiki/schema/AGENTS.md",
            "wiki/schema/wiki_schema.yaml",
            "wiki/index.md",
            "wiki/log.md"
        ]
    );
}

#[derive(Default)]
struct RecordingDrive {
    paths: Arc<Mutex<Vec<String>>>,
}

impl RecordingDrive {
    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl KnowledgeDriveStorage for RecordingDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        self.paths
            .lock()
            .unwrap()
            .push(request.logical_path.clone());
        Ok(KnowledgeObjectRef {
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
        _request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal("not needed"))
    }

    async fn get_object_text(
        &self,
        _object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal("not needed"))
    }
}
