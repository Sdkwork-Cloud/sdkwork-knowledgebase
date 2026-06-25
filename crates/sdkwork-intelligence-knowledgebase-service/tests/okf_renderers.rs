use sdkwork_intelligence_knowledgebase_service::okf::{
    render_agents_md, render_index_md, render_log_md, render_okf_profile_yaml,
    OkfBundleStandardFileService, PersistStandardFilesRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_contract::okf::{OkfConceptSummary, OkfLogEntry, OkfLogEventType};
use std::sync::{Arc, Mutex};

#[test]
fn agents_md_names_okf_layers_and_drive_storage() {
    let content = render_agents_md("Research Space");

    assert!(content.contains("raw sources"));
    assert!(content.contains("okf/index.md"));
    assert!(content.contains("schema"));
    assert!(content.contains("ingest"));
    assert!(content.contains("compile"));
    assert!(content.contains("eval"));
    assert!(content.contains("query"));
    assert!(content.contains("lint"));
    assert!(content.contains("sdkwork-drive"));
}

#[test]
fn okf_profile_yaml_declares_workflows_and_standard_paths() {
    let content = render_okf_profile_yaml();

    assert!(content.contains("workflows:"));
    assert!(content.contains("ingest"));
    assert!(content.contains("compile"));
    assert!(content.contains("eval"));
    assert!(content.contains("query"));
    assert!(content.contains("lint"));
    assert!(content.contains("refresh_standard_files"));
    assert!(content.contains("okfVersion: \"0.1\""));
    assert!(content.contains("bundleRoot: \"okf\""));
    assert!(content.contains("index: \"index.md\""));
    assert!(content.contains("log: \"log.md\""));
}

#[test]
fn index_md_renders_categories_and_okf_links() {
    let pages = vec![OkfConceptSummary {
        title: "Entity Name".to_string(),
        concept_id: "entities/entity-name".to_string(),
        concept_type: "Entity".to_string(),
        logical_path: "okf/entities/entity-name.md".to_string(),
        bundle_relative_path: "entities/entity-name.md".to_string(),
        description: "One-line entity summary.".to_string(),
        source_count: 4,
        updated_at: "2026-06-01T00:00:00Z".to_string(),
        tags: vec!["entity".to_string()],
    }];

    let content = render_index_md("Research Space", &pages);

    assert!(content.contains("okf_version: \"0.1\""));
    assert!(content.contains("## Sections"));
    assert!(content.contains("* [Entities](/entities/index.md)"));
}

#[test]
fn log_md_uses_okf_daily_sections() {
    let entries = vec![OkfLogEntry {
        occurred_at: "2026-06-01T00:00:00Z".to_string(),
        event_type: OkfLogEventType::Ingest,
        title: "Source Title".to_string(),
        actor: "system".to_string(),
        affected_concepts: vec!["Entity Name".to_string()],
        audit_event_id: Some("audit-1".to_string()),
        warnings: vec![],
    }];

    let content = render_log_md(&entries);

    assert!(content.contains("# Log"));
    assert!(content.contains("## 2026-06-01"));
    assert!(content.contains("* **Creation**: Source Title"));
    assert!(content.contains("Entity Name"));
}

#[tokio::test]
async fn standard_files_are_persisted_through_drive_port() {
    let drive = RecordingDrive::default();
    let service = OkfBundleStandardFileService::new(&drive);

    let refs = service
        .persist_standard_files(PersistStandardFilesRequest {
            space_name: "Research Space".to_string(),
            concepts: vec![OkfConceptSummary {
                title: "Entity Name".to_string(),
                concept_id: "entities/entity-name".to_string(),
                concept_type: "Entity".to_string(),
                logical_path: "okf/entities/entity-name.md".to_string(),
                bundle_relative_path: "entities/entity-name.md".to_string(),
                description: "One-line entity summary.".to_string(),
                source_count: 4,
                updated_at: "2026-06-01T00:00:00Z".to_string(),
                tags: vec!["entity".to_string()],
            }],
            log_entries: vec![],
            drive_space_id: None,
        })
        .await
        .unwrap();

    assert_eq!(refs.agents_md.logical_path, "okf/schema/AGENTS.md");
    assert_eq!(
        refs.profile_yaml.logical_path,
        "okf/schema/okf_profile.yaml"
    );
    assert_eq!(refs.index_md.logical_path, "okf/index.md");
    assert_eq!(refs.log_md.logical_path, "okf/log.md");
    let mut paths = drive.paths();
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "okf/entities/index.md",
            "okf/index.md",
            "okf/log.md",
            "okf/schema/AGENTS.md",
            "okf/schema/okf_profile.yaml",
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
