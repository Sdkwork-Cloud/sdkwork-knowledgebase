use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisioner,
    KnowledgeDriveSpaceProvisionerError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_intelligence_knowledgebase_service::space::KnowledgeSpaceService;
use sdkwork_intelligence_knowledgebase_service::wiki::{
    KnowledgeWikiFileRegistryService, KnowledgeWikiInitializerService,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::space::{
    CreateKnowledgeSpaceRequest, KnowledgeSpace, KnowledgeSpaceStatus,
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
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
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
        .with_drive_context("tenant-9001", "user-123")
        .with_drive_space_provisioner(&drive_spaces);

    let created = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
        })
        .await
        .unwrap();

    assert_eq!(created.drive_space_id.as_deref(), Some("drv-kb-001"));
    assert_eq!(drive_spaces.calls(), 1);
    assert_eq!(
        drive_spaces.requested_tenant_id(),
        Some("tenant-9001".to_string())
    );
    assert_eq!(
        drive_spaces.requested_operator_id(),
        Some("user-123".to_string())
    );
    assert_eq!(
        drive_spaces.requested_knowledge_space_uuid(),
        Some("space-1".to_string())
    );
    assert_eq!(
        drive_spaces.requested_owner_subject_type(),
        Some("user".to_string())
    );
    assert_eq!(
        drive_spaces.requested_owner_subject_id(),
        Some("test-owner".to_string())
    );
    assert!(created.llm_wiki_initialized);
    assert_eq!(drive.paths().len(), 4);
}

#[tokio::test]
async fn drive_space_provisioning_requires_context_before_creating_space_record() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let drive_spaces = RecordingDriveSpaceProvisioner::new("drv-kb-001");
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer)
        .with_drive_space_provisioner(&drive_spaces);

    let error = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
        })
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("drive tenant_id and operator_id are required"));
    assert_eq!(store.space_count(), 0);
    assert_eq!(drive_spaces.calls(), 0);
    assert!(drive.paths().is_empty());
}

#[tokio::test]
async fn drive_space_provisioning_failure_does_not_leave_half_initialized_space() {
    let store = MemorySpaceStore::default();
    let drive = RecordingDrive::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let drive_spaces = FailingDriveSpaceProvisioner;
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer)
        .with_drive_context("tenant-9001", "user-123")
        .with_drive_space_provisioner(&drive_spaces);

    let error = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
        })
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("drive space create failed intentionally"));
    assert_eq!(store.space_count(), 0);
    assert!(drive.paths().is_empty());
    assert!(file_entries.logical_paths().is_empty());
}

#[tokio::test]
async fn wiki_initialization_failure_releases_created_drive_space_and_local_space() {
    let store = MemorySpaceStore::default();
    let drive = FailingDrive;
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let drive_spaces = RecordingDriveSpaceProvisioner::new("drv-kb-001");
    let service = KnowledgeSpaceService::new(&store, &wiki_initializer)
        .with_drive_context("tenant-9001", "user-123")
        .with_drive_space_provisioner(&drive_spaces);

    let error = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
        })
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("standard wiki file write failed intentionally"));
    assert_eq!(store.space_count(), 0);
    assert_eq!(drive_spaces.delete_calls(), 1);
    assert_eq!(
        drive_spaces.deleted_drive_space_id(),
        Some("drv-kb-001".to_string())
    );
    assert!(file_entries.logical_paths().is_empty());
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
        .with_drive_context("tenant-9001", "user-123")
        .with_drive_space_provisioner(&drive_spaces);

    service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: None,
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
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
    assert_eq!(nodes[27], folder_node("wiki/pages/indexes"));
    assert_eq!(nodes[28], folder_node("wiki/pages/policies"));
    assert_eq!(nodes[29], folder_node("wiki/pages/runbooks"));
    assert_eq!(nodes[30], folder_node("graph"));
    assert_eq!(nodes[31], folder_node("candidates"));
    assert_eq!(nodes[32], folder_node("indexes"));
    assert_eq!(nodes[33], folder_node("datasets"));
    assert_eq!(nodes[34], folder_node("inventory"));
    assert_eq!(nodes[35], folder_node("context_packs"));
    assert_eq!(nodes[36], folder_node("eval"));
    assert_eq!(nodes[37], folder_node("output"));
    assert_eq!(nodes[38], folder_node("output/answers"));
    assert_eq!(nodes[39], folder_node("output/reports"));
    assert_eq!(nodes[40], folder_node("output/decks"));
    assert_eq!(nodes[41], folder_node("output/charts"));
    assert_eq!(nodes[42], folder_node("output/plans"));
    assert_eq!(nodes[43], folder_node("output/study_guides"));
    assert_eq!(nodes[44], folder_node("output/exports"));
    assert_eq!(nodes[45], folder_node("mirror"));
    assert_eq!(nodes[46], folder_node("logs"));
    assert_standard_file_node(&nodes[47], "wiki/schema/AGENTS.md", "wiki/schema/AGENTS.md");
    assert_standard_file_node(
        &nodes[48],
        "wiki/schema/wiki_schema.yaml",
        "wiki/schema/wiki_schema.yaml",
    );
    assert_standard_file_node(&nodes[49], "wiki/index.md", "wiki/index.md");
    assert_standard_file_node(&nodes[50], "wiki/log.md", "wiki/log.md");
    assert_eq!(nodes.len(), 51);
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
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: KnowledgeAgentKnowledgeMode::default(),
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("drive_space_id is required"));
    assert_eq!(store.space_count(), 0);
    assert!(drive.paths().is_empty());
    assert!(file_entries.logical_paths().is_empty());
    assert!(drive_workspace.request().is_none());
}

#[tokio::test]
async fn initializer_requires_drive_space_before_persisting_standard_files_when_workspace_enabled()
{
    let drive = RecordingDrive::default();
    let drive_workspace = RecordingDriveWorkspace::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let registry = KnowledgeWikiFileRegistryService::new(&file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive)
        .with_registry(&registry)
        .with_drive_workspace(&drive_workspace);

    let error = wiki_initializer
        .initialize_standard_files(1, "Research Space", None)
        .await
        .unwrap_err();

    assert!(error.to_string().contains("drive_space_id is required"));
    assert!(drive.paths().is_empty());
    assert!(file_entries.logical_paths().is_empty());
    assert!(drive_workspace.request().is_none());
}

#[derive(Default)]
struct MemorySpaceStore {
    next_id: Mutex<u64>,
    spaces: Mutex<Vec<KnowledgeSpace>>,
}

impl MemorySpaceStore {
    fn space_count(&self) -> usize {
        self.spaces
            .lock()
            .unwrap()
            .iter()
            .filter(|space| space.status == KnowledgeSpaceStatus::Active)
            .count()
    }
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
            knowledge_mode: record.knowledge_mode,
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

    async fn mark_space_deleted(&self, space_id: u64) -> Result<(), KnowledgeSpaceStoreError> {
        let mut spaces = self.spaces.lock().unwrap();
        let space = spaces
            .iter_mut()
            .find(|space| space.id == space_id)
            .ok_or_else(|| KnowledgeSpaceStoreError::Internal("missing space".to_string()))?;
        space.status = KnowledgeSpaceStatus::Deleted;
        Ok(())
    }
}

struct RecordingDriveSpaceProvisioner {
    drive_space_id: String,
    calls: Mutex<u32>,
    delete_calls: Mutex<u32>,
    requested_tenant_id: Mutex<Option<String>>,
    requested_operator_id: Mutex<Option<String>>,
    requested_knowledge_space_uuid: Mutex<Option<String>>,
    requested_owner_subject_type: Mutex<Option<String>>,
    requested_owner_subject_id: Mutex<Option<String>>,
    deleted_drive_space_id: Mutex<Option<String>>,
}

impl RecordingDriveSpaceProvisioner {
    fn new(drive_space_id: &str) -> Self {
        Self {
            drive_space_id: drive_space_id.to_string(),
            calls: Mutex::new(0),
            delete_calls: Mutex::new(0),
            requested_tenant_id: Mutex::new(None),
            requested_operator_id: Mutex::new(None),
            requested_knowledge_space_uuid: Mutex::new(None),
            requested_owner_subject_type: Mutex::new(None),
            requested_owner_subject_id: Mutex::new(None),
            deleted_drive_space_id: Mutex::new(None),
        }
    }

    fn calls(&self) -> u32 {
        *self.calls.lock().unwrap()
    }

    fn requested_tenant_id(&self) -> Option<String> {
        self.requested_tenant_id.lock().unwrap().clone()
    }

    fn requested_operator_id(&self) -> Option<String> {
        self.requested_operator_id.lock().unwrap().clone()
    }

    fn requested_knowledge_space_uuid(&self) -> Option<String> {
        self.requested_knowledge_space_uuid.lock().unwrap().clone()
    }

    fn requested_owner_subject_type(&self) -> Option<String> {
        self.requested_owner_subject_type.lock().unwrap().clone()
    }

    fn requested_owner_subject_id(&self) -> Option<String> {
        self.requested_owner_subject_id.lock().unwrap().clone()
    }

    fn delete_calls(&self) -> u32 {
        *self.delete_calls.lock().unwrap()
    }

    fn deleted_drive_space_id(&self) -> Option<String> {
        self.deleted_drive_space_id.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveSpaceProvisioner for RecordingDriveSpaceProvisioner {
    async fn create_knowledge_drive_space(
        &self,
        request: CreateKnowledgeDriveSpaceRequest,
    ) -> Result<KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisionerError> {
        *self.calls.lock().unwrap() += 1;
        *self.requested_tenant_id.lock().unwrap() = Some(request.tenant_id);
        *self.requested_operator_id.lock().unwrap() = Some(request.operator_id);
        *self.requested_knowledge_space_uuid.lock().unwrap() = Some(request.knowledge_space_uuid);
        *self.requested_owner_subject_type.lock().unwrap() = Some(request.owner_subject_type);
        *self.requested_owner_subject_id.lock().unwrap() = Some(request.owner_subject_id);
        Ok(KnowledgeDriveSpaceBinding {
            drive_space_id: self.drive_space_id.clone(),
        })
    }

    async fn delete_knowledge_drive_space(
        &self,
        request: sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_space::DeleteKnowledgeDriveSpaceRequest,
    ) -> Result<(), KnowledgeDriveSpaceProvisionerError> {
        *self.delete_calls.lock().unwrap() += 1;
        *self.deleted_drive_space_id.lock().unwrap() = Some(request.drive_space_id);
        Ok(())
    }
}

struct FailingDriveSpaceProvisioner;

#[async_trait]
impl KnowledgeDriveSpaceProvisioner for FailingDriveSpaceProvisioner {
    async fn create_knowledge_drive_space(
        &self,
        _request: CreateKnowledgeDriveSpaceRequest,
    ) -> Result<KnowledgeDriveSpaceBinding, KnowledgeDriveSpaceProvisionerError> {
        Err(KnowledgeDriveSpaceProvisionerError::Upstream(
            "drive space create failed intentionally".to_string(),
        ))
    }

    async fn delete_knowledge_drive_space(
        &self,
        _request: sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_space::DeleteKnowledgeDriveSpaceRequest,
    ) -> Result<(), KnowledgeDriveSpaceProvisionerError> {
        Ok(())
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

struct FailingDrive;

#[async_trait]
impl KnowledgeDriveStorage for FailingDrive {
    async fn put_object(
        &self,
        _request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal(
            "standard wiki file write failed intentionally",
        ))
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
