use async_trait::async_trait;
use sdkwork_knowledgebase_contract::space::{
    CreateKnowledgeSpaceRequest, KnowledgeSpace, KnowledgeSpaceStatus,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_knowledgebase_product::space::KnowledgeSpaceService;
use sdkwork_knowledgebase_product::wiki::{
    KnowledgeWikiFileRegistryService, KnowledgeWikiInitializerService,
};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn creating_space_initializes_llm_wiki_standard_files_through_drive() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer);

    let created = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: Some("LLM Wiki research".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(created.name, "Research Space");
    assert_eq!(created.status, KnowledgeSpaceStatus::Active);
    assert!(created.llm_wiki_initialized);
    assert_eq!(
        drive.paths(),
        vec![
            "wiki/schema/AGENTS.md",
            "wiki/schema/wiki_schema.yaml",
            "wiki/index.md",
            "wiki/log.md"
        ]
    );
    assert_eq!(file_entries.logical_paths(), drive.paths());
    assert_eq!(file_entries.space_ids(), vec![1, 1, 1, 1]);
}

#[derive(Default)]
struct MemorySpaceStore {
    next_id: Mutex<u64>,
    spaces: Mutex<Vec<KnowledgeSpace>>,
}

#[async_trait]
impl KnowledgeSpaceStore for MemorySpaceStore {
    async fn create_space(
        &self,
        record: CreateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let space = KnowledgeSpace {
            id: *next_id,
            uuid: format!("space-{}", *next_id),
            name: record.name,
            description: record.description,
            status: KnowledgeSpaceStatus::Active,
            llm_wiki_initialized: false,
        };
        self.spaces.lock().unwrap().push(space.clone());
        Ok(space)
    }

    async fn mark_llm_wiki_initialized(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let mut spaces = self.spaces.lock().unwrap();
        let space = spaces
            .iter_mut()
            .find(|space| space.id == space_id)
            .ok_or_else(|| KnowledgeSpaceStoreError::Internal("missing space".to_string()))?;
        space.llm_wiki_initialized = true;
        Ok(space.clone())
    }
}

#[derive(Default)]
struct RecordingDrive {
    paths: Arc<Mutex<Vec<String>>>,
}

#[derive(Default)]
struct MemoryWikiFileEntryStore {
    logical_paths: Mutex<Vec<String>>,
    space_ids: Mutex<Vec<u64>>,
}

impl MemoryWikiFileEntryStore {
    fn logical_paths(&self) -> Vec<String> {
        self.logical_paths.lock().unwrap().clone()
    }

    fn space_ids(&self) -> Vec<u64> {
        self.space_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeWikiFileEntryStore for MemoryWikiFileEntryStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry,
        KnowledgeWikiFileEntryStoreError,
    > {
        self.logical_paths
            .lock()
            .unwrap()
            .push(record.logical_path.clone());
        self.space_ids.lock().unwrap().push(record.space_id);

        Ok(
            sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry {
                id: self.logical_paths.lock().unwrap().len() as u64,
                space_id: record.space_id,
                logical_path: record.logical_path,
                entry_type: record.entry_type,
                artifact_role: record.artifact_role,
                drive_bucket: record.drive_bucket,
                drive_object_key: record.drive_object_key,
                checksum_sha256_hex: record.checksum_sha256_hex,
            },
        )
    }
}

impl RecordingDrive {
    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

#[async_trait]
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
