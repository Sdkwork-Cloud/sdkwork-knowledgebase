use async_trait::async_trait;
use sdkwork_drive_config::DatabaseEngine;
use sdkwork_drive_product::infrastructure::sql::install_any_schema;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CopyObjectRequest, CopyObjectResponse, CreateBucketRequest, CreateBucketResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteBucketRequest,
    DeleteBucketResponse, DeleteObjectRequest, DeleteObjectResponse, DriveObjectChunkStream,
    DriveObjectLocator, DriveObjectStore, DriveObjectStoreError, DriveObjectStoreErrorKind,
    DriveStorageProviderCapabilities, DriveStorageProviderKind, HeadBucketRequest,
    HeadBucketResponse, HeadObjectRequest, HeadObjectResponse, ListObjectsRequest,
    ListObjectsResponse, ListedObject, PresignDownloadRequest, PresignUploadPartRequest,
    PresignedDownloadResponse, PresignedUploadPartResponse, PutObjectRequest, PutObjectResponse,
    ReadObjectRangeRequest, ReadObjectRangeResponse,
};
use sdkwork_knowledgebase_drive::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveStorageAdapter,
    KnowledgebaseDriveWorkspaceAdapter,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_node_tree::{
    DriveNodeKind, KnowledgeDriveNodeTree, ListKnowledgeDriveNodeChildrenRequest,
    ResolveKnowledgeDriveNodePathRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace,
};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn adapter_puts_and_reads_objects_through_drive_object_store() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter =
        KnowledgebaseDriveStorageAdapter::new(store, "kb-bucket", "knowledge/tenant/space");

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
    let adapter =
        KnowledgebaseDriveStorageAdapter::new(store, "kb-bucket", "knowledge/tenant/space");

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
async fn adapter_rejects_unsafe_managed_logical_paths_before_drive_write() {
    let store = Arc::new(FakeDriveObjectStore::default());
    let adapter =
        KnowledgebaseDriveStorageAdapter::new(store.clone(), "kb-bucket", "knowledge/tenant/space");

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
    let adapter =
        KnowledgebaseDriveStorageAdapter::new(store.clone(), "kb-bucket", "knowledge/tenant/space");

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
    seed_drive_space(&pool, "tenant-001", "drv-kb-001").await;
    let adapter = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");

    adapter
        .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
            drive_space_id: "drv-kb-001".to_string(),
            nodes: vec![
                folder_node("wiki"),
                folder_node("wiki/schema"),
                file_node(
                    "wiki/schema/AGENTS.md",
                    "kb-bucket",
                    "knowledge/space/wiki/schema/AGENTS.md",
                    64,
                ),
            ],
        })
        .await
        .unwrap();

    let node_rows = sqlx::query(
        "SELECT node_name, node_type, content_state
         FROM drive_node
         WHERE tenant_id=$1 AND space_id=$2
         ORDER BY node_name",
    )
    .bind("tenant-001")
    .bind("drv-kb-001")
    .fetch_all(&pool)
    .await
    .unwrap();
    let node_names = node_rows
        .iter()
        .map(|row| row.get::<String, _>("node_name"))
        .collect::<Vec<_>>();
    assert_eq!(node_names, vec!["AGENTS.md", "schema", "wiki"]);
    assert_eq!(node_rows[0].get::<String, _>("node_type"), "file");
    assert_eq!(node_rows[0].get::<String, _>("content_state"), "ready");

    let object_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1)
         FROM drive_storage_object
         WHERE tenant_id=$1
           AND bucket=$2
           AND object_key=$3
           AND content_length=$4
           AND lifecycle_status='active'",
    )
    .bind("tenant-001")
    .bind("kb-bucket")
    .bind("knowledge/space/wiki/schema/AGENTS.md")
    .bind(64_i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(object_count, 1);
}

#[tokio::test]
async fn workspace_adapter_is_idempotent_for_repeated_initialization() {
    let pool = sqlite_drive_pool().await;
    seed_drive_space(&pool, "tenant-001", "drv-kb-001").await;
    let adapter = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");
    let request = EnsureKnowledgeDriveNodesRequest {
        drive_space_id: "drv-kb-001".to_string(),
        nodes: vec![
            folder_node("wiki"),
            folder_node("wiki/schema"),
            file_node(
                "wiki/schema/AGENTS.md",
                "kb-bucket",
                "knowledge/space/wiki/schema/AGENTS.md",
                64,
            ),
        ],
    };

    adapter.ensure_nodes(request.clone()).await.unwrap();
    adapter.ensure_nodes(request).await.unwrap();

    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM drive_node")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM drive_storage_object")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(node_count, 3);
    assert_eq!(object_count, 1);
}

#[tokio::test]
async fn node_tree_adapter_resolves_paths_and_pages_children_from_drive_nodes() {
    let pool = sqlite_drive_pool().await;
    seed_drive_space(&pool, "tenant-001", "drv-kb-001").await;
    let workspace = KnowledgebaseDriveWorkspaceAdapter::new(pool.clone(), "tenant-001", "system");
    workspace
        .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
            drive_space_id: "drv-kb-001".to_string(),
            nodes: vec![
                folder_node("wiki"),
                folder_node("wiki/schema"),
                file_node(
                    "wiki/index.md",
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
            drive_space_id: "drv-kb-001".to_string(),
            logical_path: "wiki".to_string(),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(root.name, "wiki");
    assert_eq!(root.kind, DriveNodeKind::Folder);

    let page = tree
        .list_children(ListKnowledgeDriveNodeChildrenRequest {
            drive_space_id: "drv-kb-001".to_string(),
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
    assert_eq!(
        page.nodes[1].content_type.as_deref(),
        Some("text/markdown; charset=utf-8")
    );
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
    sqlx::query(
        "INSERT INTO drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type, display_name,
            lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, 'app', 'sdkwork-knowledgebase', 'knowledge_base', 'Knowledge',
            'active', 1, 'system', 'system')",
    )
    .bind(drive_space_id)
    .bind(tenant_id)
    .execute(pool)
    .await
    .unwrap();
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
    bucket: &str,
    object_key: &str,
    size_bytes: u64,
) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::File,
        object_ref: Some(
            sdkwork_knowledgebase_product::ports::knowledge_drive_storage::KnowledgeObjectRef {
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
