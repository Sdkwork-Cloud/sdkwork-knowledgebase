use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteKnowledgeSiteArtifactRequest {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub operator_id: String,
    pub site_id: u64,
    pub release_id: u64,
    pub public_path: String,
    pub file_name: String,
    pub content_type: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSiteArtifactRef {
    pub drive_uri: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub content_type: String,
    pub content_length: u64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadKnowledgeSiteArtifactRequest {
    pub tenant_id: u64,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub max_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSiteArtifact {
    pub content_type: String,
    pub checksum_sha256_hex: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSiteArtifactStoreError {
    #[error("invalid site artifact request: {0}")]
    InvalidRequest(String),
    #[error("site artifact not found")]
    NotFound,
    #[error("site artifact integrity failed: {0}")]
    IntegrityFailed(String),
    #[error("site artifact store internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait KnowledgeSiteArtifactStore: Send + Sync {
    async fn write_artifact(
        &self,
        request: WriteKnowledgeSiteArtifactRequest,
    ) -> Result<KnowledgeSiteArtifactRef, KnowledgeSiteArtifactStoreError>;

    async fn read_artifact(
        &self,
        request: ReadKnowledgeSiteArtifactRequest,
    ) -> Result<KnowledgeSiteArtifact, KnowledgeSiteArtifactStoreError>;
}
