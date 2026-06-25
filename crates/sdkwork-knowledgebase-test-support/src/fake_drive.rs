use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct StoredObject {
    object_ref: KnowledgeObjectRef,
    body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FakeKnowledgeDriveStorage {
    bucket: String,
    objects: Arc<Mutex<HashMap<String, StoredObject>>>,
}

impl Default for FakeKnowledgeDriveStorage {
    fn default() -> Self {
        Self {
            bucket: "knowledgebase-test".to_string(),
            objects: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl FakeKnowledgeDriveStorage {
    pub async fn put_text(
        &self,
        logical_path: impl Into<String>,
        object_role: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        self.put_object(PutKnowledgeObjectRequest::text(
            logical_path,
            object_role,
            body,
            None,
        ))
        .await
    }

    pub async fn read_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.get_object_text(object_ref).await
    }

    pub async fn clear(&self) {
        self.objects.lock().await.clear();
    }
}

#[async_trait]
impl KnowledgeDriveStorage for FakeKnowledgeDriveStorage {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        if is_blank(Some(request.logical_path.as_str())) {
            return Err(KnowledgeStorageError::invalid_request(
                "logical_path is required",
            ));
        }

        let checksum = request
            .checksum_sha256_hex
            .clone()
            .unwrap_or_else(|| sha256_hash(&request.body));
        let object_key = request.logical_path.clone();
        let object_ref = KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: self.bucket.clone(),
            object_key: object_key.clone(),
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: Some(checksum),
            etag: None,
            version_id: None,
        };

        let object = StoredObject {
            object_ref: object_ref.clone(),
            body: request.body,
        };
        self.objects.lock().await.insert(object_key, object);

        Ok(object_ref)
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let objects = self.objects.lock().await;
        let object = objects
            .get(&request.object_key)
            .ok_or_else(|| KnowledgeStorageError::NotFound(request.object_key.clone()))?;

        if let Some(expected_logical_path) = request.logical_path {
            if object.object_ref.logical_path != expected_logical_path {
                return Err(KnowledgeStorageError::IntegrityFailed(
                    request.object_key.clone(),
                ));
            }
        }

        Ok(object.object_ref.clone())
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        let bytes = self.get_object_bytes(object_ref).await?;
        String::from_utf8(bytes)
            .map_err(|error| KnowledgeStorageError::InvalidRequest(error.to_string()))
    }

    async fn get_object_bytes(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<Vec<u8>, KnowledgeStorageError> {
        let objects = self.objects.lock().await;
        let object = objects
            .get(&object_ref.object_key)
            .ok_or_else(|| KnowledgeStorageError::NotFound(object_ref.object_key.clone()))?;

        if object.object_ref.checksum_sha256_hex != object_ref.checksum_sha256_hex {
            return Err(KnowledgeStorageError::IntegrityFailed(
                object_ref.object_key.clone(),
            ));
        }

        Ok(object.body.clone())
    }
}
