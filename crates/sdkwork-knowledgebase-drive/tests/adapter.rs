use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    AbortMultipartUploadRequest, CompleteMultipartUploadRequest, CompleteMultipartUploadResponse,
    CreateMultipartUploadRequest, CreateMultipartUploadResponse, DeleteObjectRequest,
    DeleteObjectResponse, DriveObjectChunkStream, DriveObjectLocator, DriveObjectStore,
    DriveObjectStoreError, DriveObjectStoreErrorKind, DriveStorageProviderCapabilities,
    DriveStorageProviderKind, HeadObjectRequest, HeadObjectResponse, PresignDownloadRequest,
    PresignUploadPartRequest, PresignedDownloadResponse, PresignedUploadPartResponse,
    PutObjectRequest, PutObjectResponse, ReadObjectRangeRequest, ReadObjectRangeResponse,
};
use sdkwork_knowledgebase_drive::KnowledgebaseDriveStorageAdapter;
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
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

#[derive(Default)]
struct FakeDriveObjectStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
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
        let objects = self.objects.lock().unwrap();
        let body = objects.get(&request.locator.object_key).ok_or_else(|| {
            DriveObjectStoreError::new(DriveObjectStoreErrorKind::NotFound, "missing")
        })?;

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
