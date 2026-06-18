use async_trait::async_trait;
use sdkwork_drive_config::DatabaseEngine;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectLocator, DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
    DriveStorageProviderCapabilities, DriveStorageProviderKind, HeadBucketRequest,
    HeadBucketResponse, HeadObjectRequest, HeadObjectResponse, ListBucketsRequest,
    ListBucketsResponse, ListObjectsRequest, ListObjectsResponse, ListedObject,
    PresignDownloadRequest, PresignUploadPartRequest, PresignedDownloadResponse,
    PresignedUploadPartResponse, PutObjectRequest, PutObjectResponse, ReadObjectRangeRequest,
    ReadObjectRangeResponse,
};
use sdkwork_drive_workspace_service::application::space_service::{
    GetSpaceCommand, SqlDriveSpaceService,
};
use sdkwork_drive_workspace_service::domain::space::DriveSpaceType;
use sdkwork_drive_workspace_service::infrastructure::sql::install_any_schema;
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_node_tree::{
    DriveNodeKind, KnowledgeDriveNodeTree, ListKnowledgeDriveNodeChildrenRequest,
    ResolveKnowledgeDriveNodePathRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, DeleteKnowledgeDriveSpaceRequest,
    KnowledgeDriveSpaceProvisioner,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace,
};
use sdkwork_knowledgebase_drive::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn knowledgebase_drive_permission_adapter_does_not_reference_drive_physical_tables() {
    let adapter_source = include_str!("../src/permission_adapter.rs");
    for forbidden_reference in [
        "dr_drive_space",
        "dr_drive_node",
        "dr_drive_node_permission",
        "FROM dr_",
        "JOIN dr_",
        "INSERT INTO dr_",
        "UPDATE dr_",
        "sqlx::query(",
        "sqlx::query_scalar(",
        "sdkwork_drive_workspace_service::infrastructure",
        "SqlDriveWorkspaceStore",
    ] {
        assert!(
            !adapter_source.contains(forbidden_reference),
            "knowledgebase drive permission adapter must call sdkwork-drive workspace service APIs instead of referencing: {forbidden_reference}"
        );
    }
}

#[test]
fn knowledgebase_drive_adapter_does_not_reference_drive_physical_tables() {
    let adapter_source = include_str!("../src/adapter.rs");
    for forbidden_reference in [
        "dr_space",
        "dr_node",
        "dr_storage_object",
        "dr_drive_space",
        "dr_drive_node",
        "dr_drive_storage_object",
        "FROM drive_space",
        "JOIN drive_space",
        "INSERT INTO drive_space",
        "UPDATE drive_space",
        "FROM drive_node",
        "JOIN drive_node",
        "INSERT INTO drive_node",
        "UPDATE drive_node",
        "FROM drive_storage_object",
        "JOIN drive_storage_object",
        "INSERT INTO drive_storage_object",
        "UPDATE drive_storage_object",
        "sqlx::query(",
        "sqlx::query_scalar(",
        "sdkwork_drive_workspace_service::infrastructure",
        "SqlDriveWorkspaceStore",
    ] {
        assert!(
            !adapter_source.contains(forbidden_reference),
            "knowledgebase drive adapter must call sdkwork-drive workspace service APIs instead of referencing: {forbidden_reference}"
        );
    }
}

#[tokio::test]
async fn space_provisioner_adapter_creates_dedicated_drive_knowledge_space_idempotently() {
    let pool = sqlite_drive_pool().await;
    let adapter = KnowledgebaseDriveSpaceProvisionerAdapter::new(pool.clone());
    let request = CreateKnowledgeDriveSpaceRequest {
        tenant_id: "tenant-001".to_string(),
        knowledge_space_id: 42,
        knowledge_space_uuid: "space-uuid-001".to_string(),
        display_name: "Research Space".to_string(),
        owner_subject_type: "app".to_string(),
        owner_subject_id: "sdkwork-knowledgebase:space-uuid-001".to_string(),
        operator_id: "system".to_string(),
    };

    let first = adapter
        .create_knowledge_drive_space(request.clone())
        .await
        .unwrap();
    let replay = adapter.create_knowledge_drive_space(request).await.unwrap();

    assert_eq!(first, replay);
    assert_eq!(first.drive_space_id, "kb-space-uuid-001");

    let drive_space = SqlDriveSpaceService::new(pool)
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: first.drive_space_id.clone(),
        })
        .await
        .unwrap();
    assert_eq!(
        drive_space.owner_subject_id,
        "sdkwork-knowledgebase:space-uuid-001"
    );
    assert_eq!(drive_space.owner_subject_type, "app");
    assert_eq!(drive_space.space_type, DriveSpaceType::KnowledgeBase);
    assert_eq!(drive_space.tenant_id, "tenant-001");
    assert_eq!(drive_space.display_name, "Research Space");
}

#[tokio::test]
async fn space_provisioner_adapter_deletes_only_matching_knowledge_space_idempotently() {
    let pool = sqlite_drive_pool().await;
    let adapter = KnowledgebaseDriveSpaceProvisionerAdapter::new(pool.clone());
    let create_request = CreateKnowledgeDriveSpaceRequest {
        tenant_id: "tenant-001".to_string(),
        knowledge_space_id: 42,
        knowledge_space_uuid: "space-uuid-delete".to_string(),
        display_name: "Research Space".to_string(),
        owner_subject_type: "app".to_string(),
        owner_subject_id: "sdkwork-knowledgebase:space-uuid-delete".to_string(),
        operator_id: "system".to_string(),
    };

    let binding = adapter
        .create_knowledge_drive_space(create_request)
        .await
        .unwrap();
    let delete_request = DeleteKnowledgeDriveSpaceRequest {
        tenant_id: "tenant-001".to_string(),
        drive_space_id: binding.drive_space_id.clone(),
        owner_subject_type: "app".to_string(),
        owner_subject_id: "sdkwork-knowledgebase:space-uuid-delete".to_string(),
        operator_id: "system".to_string(),
    };

    adapter
        .delete_knowledge_drive_space(delete_request.clone())
        .await
        .unwrap();
    adapter
        .delete_knowledge_drive_space(delete_request)
        .await
        .unwrap();

    let error = SqlDriveSpaceService::new(pool)
        .get_space(GetSpaceCommand {
            tenant_id: "tenant-001".to_string(),
            space_id: binding.drive_space_id,
        })
        .await
        .unwrap_err();
    assert_eq!(
        error,
        DriveServiceError::NotFound("space not found".to_string())
    );
}

#[tokio::test]
async fn adapter_puts_and_reads_objects_through_drive_object_store() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store,
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let object_ref = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/index.md",
            "wiki_index",
            "# Index",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        object_ref.object_key,
        "knowledge/tenant/space/wiki/index.md"
    );
    assert_eq!(
        adapter.get_object_text(&object_ref).await.unwrap(),
        "# Index"
    );
}

#[tokio::test]
async fn storage_adapter_returns_computed_checksum_when_request_omits_checksum() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store,
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let object_ref = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/index.md",
            "wiki_index",
            "# Index",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        object_ref.checksum_sha256_hex.as_deref(),
        Some("f084b1db4213219779bb8482e2256389c8e0a2ececf5996a3139afaa314c3e0a")
    );
}

#[tokio::test]
async fn storage_adapter_rejects_mismatched_request_checksum_before_drive_write() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store.clone(),
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let error = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/index.md",
            "wiki_index",
            "# Index",
            Some("0000000000000000000000000000000000000000000000000000000000000000".to_string()),
        ))
        .await
        .unwrap_err();

    assert!(matches!(error, KnowledgeStorageError::IntegrityFailed(_)));
    assert!(store.objects.lock().unwrap().is_empty());
}

#[tokio::test]
async fn storage_adapter_synthesizes_content_version_for_versionless_drive_store() {
    let store = Arc::new(VersionlessDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store,
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let first = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/index.md",
            "wiki_index",
            "# Index v1",
            None,
        ))
        .await
        .unwrap();
    let second = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/index.md",
            "wiki_index",
            "# Index v2",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        first.version_id.as_deref(),
        Some("sha256:3ae523fc4f79094da2a298d84b6c5bdb60c2039fe16f7e4c6b317ba07e4f78ca")
    );
    assert_eq!(
        second.version_id.as_deref(),
        Some("sha256:2d968e05caff078d47e09670f7ade6067ffd3d4a9e19aaa350c7cf7247534bb6")
    );
    assert_ne!(first.version_id, second.version_id);
}

#[tokio::test]
async fn storage_adapter_treats_blank_provider_version_as_versionless() {
    let store = Arc::new(BlankVersionDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store,
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let object_ref = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/log.md",
            "wiki_log",
            "# Log",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        object_ref.version_id.as_deref(),
        Some("sha256:d92b6f58e6ce298267202e148cedb2a48c2b73afd0134e1ac5acab957a5f1195")
    );
}

#[tokio::test]
async fn adapter_rejects_unsafe_managed_logical_paths_before_drive_write() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store.clone(),
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let error = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "../escape.md",
            "wiki_index",
            "# Escape",
            None,
        ))
        .await
        .unwrap_err();

    assert!(matches!(error, KnowledgeStorageError::InvalidRequest(_)));
    assert!(store.objects.lock().unwrap().is_empty());
}

#[tokio::test]
async fn adapter_reads_empty_text_object_without_requesting_invalid_range() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter = KnowledgebaseDriveStorageAdapter::new(
        store.clone(),
        "provider-kb",
        "kb-bucket",
        "knowledge/tenant/space",
    );

    let object_ref = adapter
        .put_object(PutKnowledgeObjectRequest::text(
            "wiki/empty.md",
            "wiki_page_markdown",
            "",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(adapter.get_object_text(&object_ref).await.unwrap(), "");
    assert_eq!(store.read_count(), 0);
}

#[tokio::test]
async fn workspace_adapter_creates_browser_visible_drive_nodes_and_file_object_bindings() {
    let pool = sqlite_drive_pool().await;
    seed_drive_space(&pool, "tenant-001", "kb-drv-kb-001").await;
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;
    let adapter = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");

    adapter
        .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            nodes: vec![
                folder_node("wiki"),
                folder_node("wiki/schema"),
                file_node(
                    "wiki/schema/AGENTS.md",
                    "provider-kb",
                    "kb-bucket",
                    "knowledge/space/wiki/schema/AGENTS.md",
                    64,
                ),
            ],
        })
        .await
        .unwrap();

    let tree = KnowledgebaseDriveNodeTreeAdapter::new(pool.clone(), "tenant-001");
    let wiki = tree
        .resolve_path(ResolveKnowledgeDriveNodePathRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            logical_path: "wiki".to_string(),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(wiki.kind, DriveNodeKind::Folder);

    let wiki_page = tree
        .list_children(ListKnowledgeDriveNodeChildrenRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            parent_drive_node_id: Some(wiki.drive_node_id),
            cursor: None,
            page_size: 200,
        })
        .await
        .unwrap();
    assert_eq!(wiki_page.nodes.len(), 1);
    assert_eq!(wiki_page.nodes[0].name, "schema");
    assert_eq!(wiki_page.nodes[0].kind, DriveNodeKind::Folder);

    let schema_page = tree
        .list_children(ListKnowledgeDriveNodeChildrenRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            parent_drive_node_id: Some(wiki_page.nodes[0].drive_node_id.clone()),
            cursor: None,
            page_size: 200,
        })
        .await
        .unwrap();
    assert_eq!(schema_page.nodes.len(), 1);
    assert_eq!(schema_page.nodes[0].name, "AGENTS.md");
    assert_eq!(schema_page.nodes[0].kind, DriveNodeKind::File);
    assert_eq!(
        schema_page.nodes[0].content_type.as_deref(),
        Some("text/markdown")
    );
    assert_eq!(schema_page.nodes[0].size_bytes, Some(64));

    let stored_provider_id: String = sqlx::query_scalar(
        "SELECT storage_provider_id
         FROM dr_drive_storage_object
         WHERE tenant_id = ?1 AND bucket = ?2 AND object_key = ?3",
    )
    .bind("tenant-001")
    .bind("kb-bucket")
    .bind("knowledge/space/wiki/schema/AGENTS.md")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored_provider_id, "provider-kb");
}

#[tokio::test]
async fn workspace_adapter_is_idempotent_for_repeated_initialization() {
    let pool = sqlite_drive_pool().await;
    seed_drive_space(&pool, "tenant-001", "kb-drv-kb-001").await;
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;
    let adapter = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");
    let request = EnsureKnowledgeDriveNodesRequest {
        drive_space_id: "kb-drv-kb-001".to_string(),
        nodes: vec![
            folder_node("wiki"),
            folder_node("wiki/schema"),
            file_node(
                "wiki/schema/AGENTS.md",
                "provider-kb",
                "kb-bucket",
                "knowledge/space/wiki/schema/AGENTS.md",
                64,
            ),
        ],
    };

    adapter.ensure_nodes(request.clone()).await.unwrap();
    adapter.ensure_nodes(request).await.unwrap();

    let tree = KnowledgebaseDriveNodeTreeAdapter::new(pool, "tenant-001");
    let schema = tree
        .resolve_path(ResolveKnowledgeDriveNodePathRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            logical_path: "wiki/schema".to_string(),
        })
        .await
        .unwrap()
        .unwrap();
    let schema_page = tree
        .list_children(ListKnowledgeDriveNodeChildrenRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            parent_drive_node_id: Some(schema.drive_node_id),
            cursor: None,
            page_size: 200,
        })
        .await
        .unwrap();
    assert_eq!(schema_page.nodes.len(), 1);
    assert_eq!(schema_page.nodes[0].name, "AGENTS.md");
    assert_eq!(schema_page.nodes[0].kind, DriveNodeKind::File);
    assert_eq!(schema_page.nodes[0].size_bytes, Some(64));
}

#[tokio::test]
async fn node_tree_adapter_resolves_paths_and_pages_children_from_drive_nodes() {
    let pool = sqlite_drive_pool().await;
    seed_drive_space(&pool, "tenant-001", "kb-drv-kb-001").await;
    seed_storage_provider(&pool, "provider-kb", "kb-bucket").await;
    let workspace = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");
    workspace
        .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            nodes: vec![
                folder_node("wiki"),
                folder_node("wiki/schema"),
                file_node(
                    "wiki/index.md",
                    "provider-kb",
                    "kb-bucket",
                    "knowledge/space/wiki/index.md",
                    11,
                ),
            ],
        })
        .await
        .unwrap();
    let tree = KnowledgebaseDriveNodeTreeAdapter::new(pool, "tenant-001");

    let root = tree
        .resolve_path(ResolveKnowledgeDriveNodePathRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            logical_path: "wiki".to_string(),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(root.name, "wiki");
    assert_eq!(root.kind, DriveNodeKind::Folder);

    let page = tree
        .list_children(ListKnowledgeDriveNodeChildrenRequest {
            drive_space_id: "kb-drv-kb-001".to_string(),
            parent_drive_node_id: Some(root.drive_node_id),
            cursor: None,
            page_size: 200,
        })
        .await
        .unwrap();
    assert_eq!(page.nodes.len(), 2);
    assert_eq!(page.nodes[0].name, "schema");
    assert_eq!(page.nodes[0].path, "wiki/schema");
    assert_eq!(page.nodes[1].name, "index.md");
    assert_eq!(page.nodes[1].kind, DriveNodeKind::File);
    assert_eq!(page.nodes[1].content_type.as_deref(), Some("text/markdown"));
    assert_eq!(page.nodes[1].size_bytes, Some(11));
    assert_eq!(page.next_cursor, None);
}

#[derive(Default)]
struct FakeDriveObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
    read_count: Mutex<usize>,
}

impl FakeDriveObjectStore {
    fn read_count(&self) -> usize {
        *self.read_count.lock().unwrap()
    }
}

#[async_trait]
impl DriveObjectStore for FakeDriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        DriveStorageProviderKind::LocalFilesystem
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities::default_local_filesystem()
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        self.objects
            .lock()
            .unwrap()
            .insert(request.locator.object_key.clone(), request.body);

        Ok(PutObjectResponse {
            locator: request.locator,
            etag: Some("etag".to_string()),
            version_id: Some("v1".to_string()),
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

        Ok(HeadObjectResponse {
            locator: request.locator,
            content_length: body.len() as u64,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            etag: Some("etag".to_string()),
            version_id: Some("v1".to_string()),
            checksum_sha256_hex: None,
            metadata: Default::default(),
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        *self.read_count.lock().unwrap() += 1;
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;
        if body.is_empty() && request.range.start_inclusive == 0 && request.range.end_inclusive == 0
        {
            return Err(DriveObjectStoreError::new(
                DriveObjectStoreErrorKind::InvalidRequest,
                "empty objects have no readable byte range",
            ));
        }

        Ok((
            ReadObjectRangeResponse {
                locator: request.locator,
                content_type: Some("text/markdown; charset=utf-8".to_string()),
                etag: Some("etag".to_string()),
                content_length: body.len() as u64,
            },
            Box::new(SingleChunkStream {
                next: Some(body.clone()),
            }),
        ))
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        let deleted = self
            .objects
            .lock()
            .unwrap()
            .remove(&request.locator.object_key)
            .is_some();
        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        Ok(HeadBucketResponse {
            bucket: request.bucket,
            exists: true,
        })
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        Ok(ListBucketsResponse { items: Vec::new() })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        Ok(CreateBucketResponse {
            bucket: request.bucket,
            created: false,
        })
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        Ok(DeleteBucketResponse {
            bucket: request.bucket,
            deleted: false,
        })
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        let prefix = request.prefix.clone().unwrap_or_default();
        let items = self
            .objects
            .lock()
            .unwrap()
            .iter()
            .filter(|(object_key, _)| object_key.starts_with(&prefix))
            .take(request.max_keys as usize)
            .map(|(object_key, body)| ListedObject {
                object_key: object_key.clone(),
                content_length: body.len() as u64,
                etag: Some("etag".to_string()),
                storage_class: None,
                last_modified_epoch_ms: None,
            })
            .collect();
        Ok(ListObjectsResponse {
            bucket: request.bucket,
            prefix: request.prefix,
            items,
            next_continuation_token: None,
            is_truncated: false,
        })
    }

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError> {
        let body = self
            .objects
            .lock()
            .unwrap()
            .get(&request.source.object_key)
            .cloned()
            .ok_or_else(|| {
                DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
            })?;
        self.objects
            .lock()
            .unwrap()
            .insert(request.destination.object_key.clone(), body);
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: Some("etag".to_string()),
            version_id: Some("v1".to_string()),
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn presign_upload_part(
        &self,
        _request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn abort_multipart_upload(
        &self,
        _request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn presign_download(
        &self,
        _request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }
}

struct SingleChunkStream {
    next: Option<Vec<u8>>,
}

#[async_trait]
impl DriveObjectChunkStream for SingleChunkStream {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, DriveObjectStoreError> {
        Ok(self.next.take())
    }
}

fn not_supported(_locator: DriveObjectLocator) -> DriveObjectStoreError {
    not_supported_message()
}

fn not_supported_message() -> DriveObjectStoreError {
    DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotSupported, "not supported")
}

#[derive(Default)]
struct VersionlessDriveObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl DriveObjectStore for VersionlessDriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        DriveStorageProviderKind::LocalFilesystem
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities::default_local_filesystem()
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        self.objects
            .lock()
            .unwrap()
            .insert(request.locator.object_key.clone(), request.body);

        Ok(PutObjectResponse {
            locator: request.locator,
            etag: None,
            version_id: None,
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

        Ok(HeadObjectResponse {
            locator: request.locator,
            content_length: body.len() as u64,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            etag: None,
            version_id: None,
            checksum_sha256_hex: None,
            metadata: Default::default(),
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

        Ok((
            ReadObjectRangeResponse {
                locator: request.locator,
                content_type: Some("text/markdown; charset=utf-8".to_string()),
                etag: None,
                content_length: body.len() as u64,
            },
            Box::new(SingleChunkStream {
                next: Some(body.clone()),
            }),
        ))
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        let deleted = self
            .objects
            .lock()
            .unwrap()
            .remove(&request.locator.object_key)
            .is_some();
        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        Ok(HeadBucketResponse {
            bucket: request.bucket,
            exists: true,
        })
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        Ok(ListBucketsResponse { items: Vec::new() })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        Ok(CreateBucketResponse {
            bucket: request.bucket,
            created: false,
        })
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        Ok(DeleteBucketResponse {
            bucket: request.bucket,
            deleted: false,
        })
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        let prefix = request.prefix.clone().unwrap_or_default();
        let items = self
            .objects
            .lock()
            .unwrap()
            .iter()
            .filter(|(object_key, _)| object_key.starts_with(&prefix))
            .take(request.max_keys as usize)
            .map(|(object_key, body)| ListedObject {
                object_key: object_key.clone(),
                content_length: body.len() as u64,
                etag: None,
                storage_class: None,
                last_modified_epoch_ms: None,
            })
            .collect();
        Ok(ListObjectsResponse {
            bucket: request.bucket,
            prefix: request.prefix,
            items,
            next_continuation_token: None,
            is_truncated: false,
        })
    }

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError> {
        let body = self
            .objects
            .lock()
            .unwrap()
            .get(&request.source.object_key)
            .cloned()
            .ok_or_else(|| {
                DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
            })?;
        self.objects
            .lock()
            .unwrap()
            .insert(request.destination.object_key.clone(), body);
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: None,
            version_id: None,
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn presign_upload_part(
        &self,
        _request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn abort_multipart_upload(
        &self,
        _request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn presign_download(
        &self,
        _request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }
}

#[derive(Default)]
struct BlankVersionDriveObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl DriveObjectStore for BlankVersionDriveObjectStore {
    fn provider_kind(&self) -> DriveStorageProviderKind {
        DriveStorageProviderKind::LocalFilesystem
    }

    fn capabilities(&self) -> DriveStorageProviderCapabilities {
        DriveStorageProviderCapabilities::default_local_filesystem()
    }

    async fn put_object(
        &self,
        request: PutObjectRequest,
    ) -> Result<PutObjectResponse, DriveObjectStoreError> {
        self.objects
            .lock()
            .unwrap()
            .insert(request.locator.object_key.clone(), request.body);

        Ok(PutObjectResponse {
            locator: request.locator,
            etag: None,
            version_id: Some("   ".to_string()),
        })
    }

    async fn head_object(
        &self,
        request: HeadObjectRequest,
    ) -> Result<HeadObjectResponse, DriveObjectStoreError> {
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

        Ok(HeadObjectResponse {
            locator: request.locator,
            content_length: body.len() as u64,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            etag: None,
            version_id: Some(String::new()),
            checksum_sha256_hex: Some(test_checksum_sha256_hex(body)),
            metadata: Default::default(),
        })
    }

    async fn read_object_range(
        &self,
        request: ReadObjectRangeRequest,
    ) -> Result<(ReadObjectRangeResponse, Box<dyn DriveObjectChunkStream>), DriveObjectStoreError>
    {
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

        Ok((
            ReadObjectRangeResponse {
                locator: request.locator,
                content_type: Some("text/markdown; charset=utf-8".to_string()),
                etag: None,
                content_length: body.len() as u64,
            },
            Box::new(SingleChunkStream {
                next: Some(body.clone()),
            }),
        ))
    }

    async fn delete_object(
        &self,
        request: DeleteObjectRequest,
    ) -> Result<DeleteObjectResponse, DriveObjectStoreError> {
        let deleted = self
            .objects
            .lock()
            .unwrap()
            .remove(&request.locator.object_key)
            .is_some();
        Ok(DeleteObjectResponse {
            locator: request.locator,
            deleted,
        })
    }

    async fn head_bucket(
        &self,
        request: HeadBucketRequest,
    ) -> Result<HeadBucketResponse, DriveObjectStoreError> {
        Ok(HeadBucketResponse {
            bucket: request.bucket,
            exists: true,
        })
    }

    async fn list_buckets(
        &self,
        _request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, DriveObjectStoreError> {
        Ok(ListBucketsResponse { items: Vec::new() })
    }

    async fn create_bucket(
        &self,
        request: CreateBucketRequest,
    ) -> Result<CreateBucketResponse, DriveObjectStoreError> {
        Ok(CreateBucketResponse {
            bucket: request.bucket,
            created: false,
        })
    }

    async fn delete_bucket(
        &self,
        request: DeleteBucketRequest,
    ) -> Result<DeleteBucketResponse, DriveObjectStoreError> {
        Ok(DeleteBucketResponse {
            bucket: request.bucket,
            deleted: false,
        })
    }

    async fn list_objects(
        &self,
        request: ListObjectsRequest,
    ) -> Result<ListObjectsResponse, DriveObjectStoreError> {
        let prefix = request.prefix.clone().unwrap_or_default();
        let items = self
            .objects
            .lock()
            .unwrap()
            .iter()
            .filter(|(object_key, _)| object_key.starts_with(&prefix))
            .take(request.max_keys as usize)
            .map(|(object_key, body)| ListedObject {
                object_key: object_key.clone(),
                content_length: body.len() as u64,
                etag: None,
                storage_class: None,
                last_modified_epoch_ms: None,
            })
            .collect();
        Ok(ListObjectsResponse {
            bucket: request.bucket,
            prefix: request.prefix,
            items,
            next_continuation_token: None,
            is_truncated: false,
        })
    }

    async fn copy_object(
        &self,
        request: CopyObjectRequest,
    ) -> Result<CopyObjectResponse, DriveObjectStoreError> {
        let body = self
            .objects
            .lock()
            .unwrap()
            .get(&request.source.object_key)
            .cloned()
            .ok_or_else(|| {
                DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
            })?;
        self.objects
            .lock()
            .unwrap()
            .insert(request.destination.object_key.clone(), body);
        Ok(CopyObjectResponse {
            locator: request.destination,
            etag: None,
            version_id: Some(String::new()),
        })
    }

    async fn create_multipart_upload(
        &self,
        request: CreateMultipartUploadRequest,
    ) -> Result<CreateMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn presign_upload_part(
        &self,
        _request: PresignUploadPartRequest,
    ) -> Result<PresignedUploadPartResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn complete_multipart_upload(
        &self,
        request: CompleteMultipartUploadRequest,
    ) -> Result<CompleteMultipartUploadResponse, DriveObjectStoreError> {
        Err(not_supported(request.locator))
    }

    async fn abort_multipart_upload(
        &self,
        _request: AbortMultipartUploadRequest,
    ) -> Result<(), DriveObjectStoreError> {
        Err(not_supported_message())
    }

    async fn presign_download(
        &self,
        _request: PresignDownloadRequest,
    ) -> Result<PresignedDownloadResponse, DriveObjectStoreError> {
        Err(not_supported_message())
    }
}

fn test_checksum_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

async fn sqlite_drive_pool() -> sqlx::AnyPool {
    sqlx::any::install_default_drivers();
    let pool = sqlx::any::AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    install_any_schema(&pool, DatabaseEngine::Sqlite)
        .await
        .unwrap();
    pool
}

async fn seed_drive_space(pool: &sqlx::AnyPool, tenant_id: &str, drive_space_id: &str) {
    let knowledge_space_uuid = drive_space_id
        .strip_prefix("kb-")
        .unwrap_or(drive_space_id)
        .to_string();
    let binding = KnowledgebaseDriveSpaceProvisionerAdapter::new(pool.clone())
        .create_knowledge_drive_space(CreateKnowledgeDriveSpaceRequest {
            tenant_id: tenant_id.to_string(),
            knowledge_space_id: 1,
            knowledge_space_uuid,
            display_name: "Knowledge".to_string(),
            owner_subject_type: "app".to_string(),
            owner_subject_id: format!("sdkwork-knowledgebase:{drive_space_id}"),
            operator_id: "system".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(binding.drive_space_id, drive_space_id);
}

async fn seed_storage_provider(pool: &sqlx::AnyPool, provider_id: &str, bucket: &str) {
    sqlx::query(
        "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            ?1, 's3_compatible', ?1, 'https://s3.example.com', 'us-east-1',
            ?2, 1, 1, 'plain:test-access:test-secret', NULL, NULL,
            'active', 1, 'test', 'test'
        )",
    )
    .bind(provider_id)
    .bind(bucket)
    .execute(pool)
    .await
    .expect("seed storage provider should succeed");
}

fn folder_node(logical_path: &str) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::Folder,
        object_ref: None,
    }
}

fn file_node(
    logical_path: &str,
    storage_provider_id: &str,
    bucket: &str,
    object_key: &str,
    size_bytes: u64,
) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::File,
        object_ref: Some(
            sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeObjectRef {
                storage_provider_id: storage_provider_id.to_string(),
                bucket: bucket.to_string(),
                object_key: object_key.to_string(),
                logical_path: logical_path.to_string(),
                object_role: "wiki_schema".to_string(),
                content_type: "text/markdown; charset=utf-8".to_string(),
                size_bytes,
                checksum_sha256_hex: Some(
                    "9cb34ab8b2d953ad722c1d727df449e0a216e4fe12f5433a3a945db596d792fb".to_string(),
                ),
                etag: None,
                version_id: None,
            },
        ),
    }
}
