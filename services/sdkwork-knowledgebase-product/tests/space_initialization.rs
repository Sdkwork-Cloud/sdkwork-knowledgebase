use async_trait::async_trait;
use sdkwork_knowledgebase_contract::space::{
    CreateKnowledgeSpaceRequest, KnowledgeSpace, KnowledgeSpaceStatus,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisioner,
    KnowledgeDriveSpaceProvisionerError,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
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

#[tokio::test]
async fn creating_space_binds_dedicated_drive_knowledge_space_before_wiki_initialization() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let drive_spaces = RecordingDriveSpaceProvisioner::new("drv-kb-001");
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer)
        .with_drive_space_provisioner(&drive_spaces);

    let created = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
        })
        .await
        .unwrap();

    assert_eq!(created.drive_space_id.as_deref(), Some("drv-kb-001"));
    assert_eq!(drive_spaces.calls(), 1);
    assert_eq!(
        drive_spaces.requested_knowledge_space_uuid(),
        Some("space-1".to_string())
    );
    assert!(created.llm_wiki_initialized);
    assert_eq!(drive.paths().len(), 4);
}

#[tokio::test]
async fn creating_bound_space_initializes_browser_visible_llm_wiki_drive_nodes() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let drive_workspace = RecordingDriveWorkspace::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive)
        .with_registry(&registry)
        .with_drive_workspace(&drive_workspace);
    let drive_spaces = RecordingDriveSpaceProvisioner::new("drv-kb-001");
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer)
        .with_drive_space_provisioner(&drive_spaces);

    service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
        })
        .await
        .unwrap();

    assert_eq!(drive_workspace.request_count(), 1);
    let request = drive_workspace.request().unwrap();
    assert_eq!(request.drive_space_id, "drv-kb-001");
    let nodes = request.nodes;
    assert_eq!(nodes[0], folder_node("manifest"));
    assert_eq!(nodes[1], folder_node("inbox"));
    assert_eq!(nodes[2], folder_node("inbox/uploads"));
    assert_eq!(nodes[3], folder_node("inbox/drive-imports"));
    assert_eq!(nodes[4], folder_node("inbox/api"));
    assert_eq!(nodes[5], folder_node("sources"));
    assert_eq!(nodes[6], folder_node("sources/raw"));
    assert_eq!(nodes[7], folder_node("sources/urls"));
    assert_eq!(nodes[8], folder_node("sources/repos"));
    assert_eq!(nodes[9], folder_node("sources/message_archives"));
    assert_eq!(nodes[10], folder_node("sources/media"));
    assert_eq!(nodes[11], folder_node("parsed"));
    assert_eq!(nodes[12], folder_node("wiki"));
    assert_eq!(nodes[13], folder_node("wiki/schema"));
    assert_eq!(nodes[14], folder_node("wiki/pages"));
    assert_eq!(nodes[15], folder_node("wiki/pages/sources"));
    assert_eq!(nodes[16], folder_node("wiki/pages/entities"));
    assert_eq!(nodes[17], folder_node("wiki/pages/concepts"));
    assert_eq!(nodes[18], folder_node("wiki/pages/topics"));
    assert_eq!(nodes[19], folder_node("wiki/pages/references"));
    assert_eq!(nodes[20], folder_node("wiki/pages/how_to"));
    assert_eq!(nodes[21], folder_node("wiki/pages/faq"));
    assert_eq!(nodes[22], folder_node("wiki/pages/glossary"));
    assert_eq!(nodes[23], folder_node("wiki/pages/answers"));
    assert_eq!(nodes[24], folder_node("wiki/pages/comparisons"));
    assert_eq!(nodes[25], folder_node("wiki/pages/presentations"));
    assert_eq!(nodes[26], folder_node("wiki/pages/charts"));
    assert_eq!(nodes[27], folder_node("graph"));
    assert_eq!(nodes[28], folder_node("candidates"));
    assert_eq!(nodes[29], folder_node("indexes"));
    assert_eq!(nodes[30], folder_node("datasets"));
    assert_eq!(nodes[31], folder_node("inventory"));
    assert_eq!(nodes[32], folder_node("context_packs"));
    assert_eq!(nodes[33], folder_node("eval"));
    assert_eq!(nodes[34], folder_node("output"));
    assert_eq!(nodes[35], folder_node("output/answers"));
    assert_eq!(nodes[36], folder_node("output/reports"));
    assert_eq!(nodes[37], folder_node("output/decks"));
    assert_eq!(nodes[38], folder_node("output/charts"));
    assert_eq!(nodes[39], folder_node("output/plans"));
    assert_eq!(nodes[40], folder_node("output/study_guides"));
    assert_eq!(nodes[41], folder_node("output/exports"));
    assert_eq!(nodes[42], folder_node("mirror"));
    assert_eq!(nodes[43], folder_node("logs"));
    assert_standard_file_node(&nodes[44], "wiki/schema/AGENTS.md", "wiki/schema/AGENTS.md");
    assert_standard_file_node(
        &nodes[45],
        "wiki/schema/wiki_schema.yaml",
        "wiki/schema/wiki_schema.yaml",
    );
    assert_standard_file_node(&nodes[46], "wiki/index.md", "wiki/index.md");
    assert_standard_file_node(&nodes[47], "wiki/log.md", "wiki/log.md");
    assert_eq!(nodes.len(), 48);
}

#[tokio::test]
async fn workspace_backed_initialization_requires_drive_space_binding() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let drive_workspace = RecordingDriveWorkspace::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive)
        .with_registry(&registry)
        .with_drive_workspace(&drive_workspace);
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer);

    let error = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("drive_space_id is required"));
    assert!(drive_workspace.request().is_none());
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
            drive_space_id: None,
            status: KnowledgeSpaceStatus::Active,
            llm_wiki_initialized: false,
        };
        self.spaces.lock().unwrap().push(space.clone());
        Ok(space)
    }

    async fn get_space(&self, space_id: u64) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        self.spaces
            .lock()
            .unwrap()
            .iter()
            .find(|space| space.id == space_id)
            .cloned()
            .ok_or_else(|| KnowledgeSpaceStoreError::Internal("missing space".to_string()))
    }

    async fn mark_drive_space_bound(
        &self,
        space_id: u64,
        drive_space_id: String,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let mut spaces = self.spaces.lock().unwrap();
        let space = spaces
            .iter_mut()
            .find(|space| space.id == space_id)
            .ok_or_else(|| KnowledgeSpaceStoreError::Internal("missing space".to_string()))?;
        space.drive_space_id = Some(drive_space_id);
        Ok(space.clone())
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

struct RecordingDriveSpaceProvisioner {
    drive_space_id: String,
    calls: Mutex<u32>,
    requested_knowledge_space_uuid: Mutex<Option<String>>,
}

impl RecordingDriveSpaceProvisioner {
    fn new(drive_space_id: &str) -> Self {
        Self {
            drive_space_id: drive_space_id.to_string(),
            calls: Mutex::new(0),
            requested_knowledge_space_uuid: Mutex::new(None),
        }
    }

    fn calls(&self) -> u32 {
        *self.calls.lock().unwrap()
    }

    fn requested_knowledge_space_uuid(&self) -> Option<String> {
        self.requested_knowledge_space_uuid.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveSpaceProvisioner for RecordingDriveSpaceProvisioner {
    async fn create_knowledge_drive_space(
        &self,
        request: CreateKnowledgeDriveSpaceRequest,
    ) -> Result<KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisionerError> {
        *self.calls.lock().unwrap() += 1;
        *self.requested_knowledge_space_uuid.lock().unwrap() = Some(request.knowledge_space_uuid);
        Ok(KnowledgeDriveSpaceBinding {
            drive_space_id: self.drive_space_id.clone(),
        })
    }
}

#[derive(Default)]
struct RecordingDrive {
    paths: Arc<Mutex<Vec<String>>>,
}

#[derive(Default)]
struct RecordingDriveWorkspace {
    requests: Mutex<Vec<EnsureKnowledgeDriveNodesRequest>>,
}

impl RecordingDriveWorkspace {
    fn request(&self) -> Option<EnsureKnowledgeDriveNodesRequest> {
        let requests = self.requests.lock().unwrap();
        requests.first().cloned()
    }

    fn request_count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDriveWorkspace for RecordingDriveWorkspace {
    async fn ensure_nodes(
        &self,
        request: EnsureKnowledgeDriveNodesRequest,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        self.requests.lock().unwrap().push(request);
        Ok(())
    }
}

fn folder_node(logical_path: &str) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::Folder,
        object_ref: None,
    }
}

fn assert_standard_file_node(
    node: &EnsureKnowledgeDriveNodeRequest,
    logical_path: &str,
    object_key: &str,
) {
    assert_eq!(node.logical_path, logical_path);
    assert_eq!(node.kind, EnsureKnowledgeDriveNodeKind::File);
    let object_ref = node.object_ref.as_ref().unwrap();
    assert_eq!(object_ref.bucket, "test");
    assert_eq!(object_ref.object_key, object_key);
    assert_eq!(object_ref.logical_path, logical_path);
    assert_eq!(object_ref.content_type, "text/markdown; charset=utf-8");
    assert!(object_ref.size_bytes > 0);
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
