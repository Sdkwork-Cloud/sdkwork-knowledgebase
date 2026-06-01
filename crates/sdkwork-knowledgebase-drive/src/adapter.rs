use async_trait::async_trait;
use sdkwork_drive_storage_contract::{
    DriveByteRange, DriveObjectLocator, DriveObjectStore, DriveObjectStoreError,
    DriveObjectStoreErrorKind, HeadObjectRequest, PutObjectRequest, ReadObjectRangeRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct KnowledgebaseDriveStorageAdapter {
    store: Arc<dyn DriveObjectStore>,
    bucket: String,
    object_key_root: String,
}

impl KnowledgebaseDriveStorageAdapter {
    pub fn new<S>(
        store: Arc<S>,
        bucket: impl Into<String>,
        object_key_root: impl Into<String>,
    ) -> Self
    where
        S: DriveObjectStore + 'static,
    {
        Self {
            store,
            bucket: bucket.into(),
            object_key_root: trim_slashes(&object_key_root.into()),
        }
    }

    fn locator_for(&self, logical_path: &str) -> Result<DriveObjectLocator, KnowledgeStorageError> {
        let safe_logical_path = safe_logical_path(logical_path)?;
        let object_key = if self.object_key_root.is_empty() {
            safe_logical_path
        } else {
            format!("{}/{}", self.object_key_root, safe_logical_path)
        };

        Ok(DriveObjectLocator {
            bucket: self.bucket.clone(),
            object_key,
        })
    }
}

#[async_trait]
impl KnowledgeDriveStorage for KnowledgebaseDriveStorageAdapter {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let locator = self.locator_for(&request.logical_path)?;
        let size_bytes = request.body.len() as u64;
        let mut metadata = BTreeMap::new();
        metadata.insert("logical_path".to_string(), request.logical_path.clone());
        metadata.insert("object_role".to_string(), request.object_role.clone());

        let response = self
            .store
            .put_object(PutObjectRequest {
                locator: locator.clone(),
                content_type: Some(request.content_type.clone()),
                metadata,
                body: request.body,
                checksum_sha256_hex: request.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(map_drive_error)?;

        Ok(KnowledgeObjectRef {
            bucket: response.locator.bucket,
            object_key: response.locator.object_key,
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes,
            checksum_sha256_hex: request.checksum_sha256_hex,
            etag: response.etag,
            version_id: response.version_id,
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let logical_path = request
            .logical_path
            .clone()
            .unwrap_or_else(|| request.object_key.clone());
        let locator = if request.bucket.is_empty() {
            self.locator_for(&logical_path)?
        } else {
            DriveObjectLocator {
                bucket: request.bucket,
                object_key: request.object_key,
            }
        };
        let response = self
            .store
            .head_object(HeadObjectRequest { locator })
            .await
            .map_err(map_drive_error)?;

        Ok(KnowledgeObjectRef {
            bucket: response.locator.bucket,
            object_key: response.locator.object_key,
            logical_path,
            object_role: request.object_role,
            content_type: response
                .content_type
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size_bytes: response.content_length,
            checksum_sha256_hex: response.checksum_sha256_hex,
            etag: response.etag,
            version_id: response.version_id,
        })
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        let end_inclusive = object_ref.size_bytes.saturating_sub(1);
        let (_, mut stream) = self
            .store
            .read_object_range(ReadObjectRangeRequest {
                locator: DriveObjectLocator {
                    bucket: object_ref.bucket.clone(),
                    object_key: object_ref.object_key.clone(),
                },
                range: DriveByteRange {
                    start_inclusive: 0,
                    end_inclusive,
                },
            })
            .await
            .map_err(map_drive_error)?;

        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next_chunk().await.map_err(map_drive_error)? {
            bytes.extend_from_slice(&chunk);
        }

        String::from_utf8(bytes)
            .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))
    }
}

fn trim_slashes(value: &str) -> String {
    value.trim_matches('/').replace('\\', "/")
}

fn safe_logical_path(value: &str) -> Result<String, KnowledgeStorageError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.contains(':')
    {
        return Err(KnowledgeStorageError::InvalidRequest(format!(
            "unsafe logical_path: {value}"
        )));
    }

    let normalized = trimmed.replace('\\', "/");
    let mut segments = Vec::new();
    for segment in normalized.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || !segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
        {
            return Err(KnowledgeStorageError::InvalidRequest(format!(
                "unsafe logical_path: {value}"
            )));
        }
        segments.push(segment);
    }

    Ok(segments.join("/"))
}

fn map_drive_error(error: DriveObjectStoreError) -> KnowledgeStorageError {
    match error.kind {
        DriveObjectStoreErrorKind::NotFound => KnowledgeStorageError::NotFound(error.message),
        DriveObjectStoreErrorKind::InvalidRequest => {
            KnowledgeStorageError::InvalidRequest(error.message)
        }
        DriveObjectStoreErrorKind::IntegrityFailed => {
            KnowledgeStorageError::IntegrityFailed(error.message)
        }
        DriveObjectStoreErrorKind::PermissionDenied
        | DriveObjectStoreErrorKind::Timeout
        | DriveObjectStoreErrorKind::Unavailable
        | DriveObjectStoreErrorKind::RateLimited
        | DriveObjectStoreErrorKind::Conflict
        | DriveObjectStoreErrorKind::UpstreamError
        | DriveObjectStoreErrorKind::NotSupported => KnowledgeStorageError::Upstream(error.message),
        DriveObjectStoreErrorKind::Internal => KnowledgeStorageError::Internal(error.message),
    }
}
