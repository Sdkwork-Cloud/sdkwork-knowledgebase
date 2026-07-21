use async_trait::async_trait;
use thiserror::Error;

pub const KNOWLEDGEBASE_RAW_CONSUMER_KIND: &str = "knowledgebase_raw";
pub const ROOT_SCOPE_SUBSCRIPTION_TYPE: &str = "ROOT_SCOPE_SUBSCRIPTION";
pub const MAX_WIKI_SOURCE_READ_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureKnowledgebaseRawScopeRequest {
    pub drive_space_id: String,
    pub knowledgebase_uuid: String,
    pub raw_folder_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgebaseRawScope {
    pub subscription_uuid: String,
    pub drive_space_id: String,
    pub consumer_kind: String,
    pub knowledgebase_uuid: String,
    pub raw_folder_node_id: String,
    pub scope_status: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveKnowledgeWikiSourceRequest {
    pub subscription_uuid: String,
    pub relative_path: String,
    pub pinned_generation: Option<String>,
    pub pinned_node_version_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWikiSourceResource {
    pub scope_type: String,
    pub subscription_uuid: String,
    pub scope_generation: String,
    pub normalized_relative_path: String,
    pub resource_type: String,
    pub drive_node_id: String,
    pub drive_node_version_id: String,
    pub version_no: String,
    pub checksum_sha256_hex: String,
    pub etag: String,
    pub content_type: String,
    pub content_length: u64,
    pub last_modified: String,
    pub scope_status: String,
    pub node_status: String,
    pub eligibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadKnowledgeWikiSourceRequest {
    pub resource: KnowledgeWikiSourceResource,
    pub maximum_bytes: u64,
}

#[async_trait]
pub trait KnowledgeWikiDriveScope: Send + Sync {
    async fn ensure_raw_scope(
        &self,
        request: EnsureKnowledgebaseRawScopeRequest,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError>;

    async fn retrieve_raw_scope(
        &self,
        subscription_uuid: &str,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError>;
}

#[async_trait]
pub trait KnowledgeWikiDriveSource: KnowledgeWikiDriveScope {
    async fn resolve_source(
        &self,
        request: ResolveKnowledgeWikiSourceRequest,
    ) -> Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError>;

    async fn read_pinned_source(
        &self,
        request: ReadKnowledgeWikiSourceRequest,
    ) -> Result<Vec<u8>, KnowledgeWikiDriveSourceError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeWikiDriveSourceError {
    #[error("Wiki Drive source invalid request: {0}")]
    InvalidRequest(String),
    #[error("Wiki Drive source not found: {0}")]
    NotFound(String),
    #[error("Wiki Drive source conflict: {0}")]
    Conflict(String),
    #[error("Wiki Drive source integrity failed: {0}")]
    IntegrityFailed(String),
    #[error("Wiki Drive source upstream error: {0}")]
    Upstream(String),
}
