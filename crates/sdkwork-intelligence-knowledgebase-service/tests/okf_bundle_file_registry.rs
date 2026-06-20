use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::okf::{
    OkfBundleFileRegistryService, PersistedStandardFiles,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeObjectRef;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_bundle_file_store::{
    CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
    KnowledgeOkfBundleFileStoreError,
};
use sdkwork_knowledgebase_contract::{KnowledgeOkfBundleFile, OkfBundleFileKind};
use std::sync::Mutex;

#[tokio::test]
async fn registry_records_standard_okf_bundle_files_after_drive_persistence() {
    let store = MemoryOkfBundleFileStore::default();
    let registry = OkfBundleFileRegistryService::new(&store);

    let entries = registry
        .register_standard_files(
            7,
            &PersistedStandardFiles {
                agents_md: object_ref("okf/schema/AGENTS.md", "bundle_profile"),
                profile_yaml: object_ref("okf/schema/okf_profile.yaml", "bundle_profile"),
                index_md: object_ref("okf/index.md", "bundle_index"),
                log_md: object_ref("okf/log.md", "bundle_log"),
            },
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].space_id, 7);
    assert_eq!(entries[0].file_kind, OkfBundleFileKind::BundleAgents);
    assert_eq!(entries[1].file_kind, OkfBundleFileKind::BundleProfile);
    assert_eq!(entries[2].file_kind, OkfBundleFileKind::BundleIndex);
    assert_eq!(entries[3].file_kind, OkfBundleFileKind::BundleLog);
    assert_eq!(entries[2].logical_path, "okf/index.md");
    assert_eq!(entries[2].drive_object_key, "objects/okf/index.md");
}

fn object_ref(logical_path: &str, object_role: &str) -> KnowledgeObjectRef {
    KnowledgeObjectRef {
        storage_provider_id: "provider-kb".to_string(),
        bucket: "kb".to_string(),
        object_key: format!("objects/{logical_path}"),
        logical_path: logical_path.to_string(),
        object_role: object_role.to_string(),
        content_type: "text/markdown; charset=utf-8".to_string(),
        size_bytes: 10,
        checksum_sha256_hex: Some("checksum".to_string()),
        etag: None,
        version_id: None,
    }
}

#[derive(Default)]
struct MemoryOkfBundleFileStore {
    next_id: Mutex<u64>,
    entries: Mutex<Vec<KnowledgeOkfBundleFile>>,
}

#[async_trait]
impl KnowledgeOkfBundleFileStore for MemoryOkfBundleFileStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let entry = KnowledgeOkfBundleFile {
            id: *next_id,
            space_id: record.space_id,
            logical_path: record.logical_path,
            file_kind: record.file_kind,
            artifact_role: record.artifact_role,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            checksum_sha256_hex: record.checksum_sha256_hex,
            staged_import_root: None,
            import_id: None,
        };
        self.entries.lock().unwrap().push(entry.clone());
        Ok(entry)
    }
}
