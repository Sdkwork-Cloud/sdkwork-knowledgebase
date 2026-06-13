use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeDriveStorage: Send + Sync {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError>;

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError>;

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutKnowledgeObjectRequest {
    pub logical_path: String,
    pub object_role: String,
    pub content_type: String,
    pub body: Vec<u8>,
    pub checksum_sha256_hex: Option<String>,
}

impl PutKnowledgeObjectRequest {
    pub fn text(
        logical_path: impl Into<String>,
        object_role: impl Into<String>,
        body: impl Into<String>,
        checksum_sha256_hex: Option<String>,
    ) -> Self {
        Self {
            logical_path: logical_path.into(),
            object_role: object_role.into(),
            content_type: "text/markdown; charset=utf-8".to_string(),
            body: body.into().into_bytes(),
            checksum_sha256_hex,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadKnowledgeObjectRequest {
    pub storage_provider_id: Option<String>,
    pub bucket: String,
    pub object_key: String,
    pub logical_path: Option<String>,
    pub object_role: String,
}

impl HeadKnowledgeObjectRequest {
    pub fn original_document(
        storage_provider_id: impl Into<String>,
        bucket: impl Into<String>,
        object_key: impl Into<String>,
    ) -> Self {
        let object_key = object_key.into();
        Self {
            storage_provider_id: Some(storage_provider_id.into()),
            bucket: bucket.into(),
            logical_path: Some(object_key.clone()),
            object_key,
            object_role: "original_document".to_string(),
        }
    }

    pub fn managed_artifact(
        logical_path: impl Into<String>,
        object_role: impl Into<String>,
    ) -> Self {
        let logical_path = logical_path.into();
        Self {
            storage_provider_id: None,
            bucket: String::new(),
            object_key: logical_path.clone(),
            logical_path: Some(logical_path),
            object_role: object_role.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeObjectRef {
    pub storage_provider_id: String,
    pub bucket: String,
    pub object_key: String,
    pub logical_path: String,
    pub object_role: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub checksum_sha256_hex: Option<String>,
    pub etag: Option<String>,
    pub version_id: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeStorageError {
    #[error("knowledge storage not found: {0}")]
    NotFound(String),
    #[error("knowledge storage invalid request: {0}")]
    InvalidRequest(String),
    #[error("knowledge storage integrity failed: {0}")]
    IntegrityFailed(String),
    #[error("knowledge storage upstream error: {0}")]
    Upstream(String),
    #[error("knowledge storage internal error: {0}")]
    Internal(String),
}

impl KnowledgeStorageError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }
}
