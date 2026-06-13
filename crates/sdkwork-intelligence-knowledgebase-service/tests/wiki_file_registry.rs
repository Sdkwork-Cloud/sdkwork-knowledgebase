use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeObjectRef;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_intelligence_knowledgebase_service::wiki::{
    KnowledgeWikiFileRegistryService, PersistedStandardFiles,
};
use sdkwork_knowledgebase_contract::wiki_file::{KnowledgeWikiFileEntry, WikiFileEntryType};
use std::sync::Mutex;

#[tokio::test]
async fn registry_records_standard_llm_wiki_files_after_drive_persistence() {
    let store = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&store);

    let entries = registry
        .register_standard_files(
            7,
            &PersistedStandardFiles {
                agents_md: object_ref("wiki/schema/AGENTS.md", "wiki_schema"),
                schema_yaml: object_ref("wiki/schema/wiki_schema.yaml", "wiki_schema"),
                index_md: object_ref("wiki/index.md", "wiki_index"),
                log_md: object_ref("wiki/log.md", "wiki_log"),
            },
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].space_id, 7);
    assert_eq!(entries[0].entry_type, WikiFileEntryType::WikiSchema);
    assert_eq!(entries[1].entry_type, WikiFileEntryType::WikiSchema);
    assert_eq!(entries[2].entry_type, WikiFileEntryType::WikiIndex);
    assert_eq!(entries[3].entry_type, WikiFileEntryType::WikiLog);
    assert_eq!(entries[2].logical_path, "wiki/index.md");
    assert_eq!(entries[2].drive_object_key, "objects/wiki/index.md");
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
struct MemoryWikiFileEntryStore {
    next_id: Mutex<u64>,
    entries: Mutex<Vec<KnowledgeWikiFileEntry>>,
}

#[async_trait]
impl KnowledgeWikiFileEntryStore for MemoryWikiFileEntryStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let entry = KnowledgeWikiFileEntry {
            id: *next_id,
            space_id: record.space_id,
            logical_path: record.logical_path,
            entry_type: record.entry_type,
            artifact_role: record.artifact_role,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            checksum_sha256_hex: record.checksum_sha256_hex,
        };
        self.entries.lock().unwrap().push(entry.clone());
        Ok(entry)
    }
}
