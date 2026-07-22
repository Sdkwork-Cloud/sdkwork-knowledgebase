use async_trait::async_trait;
use thiserror::Error;

pub const KNOWLEDGEBASE_RAW_CONSUMER_KIND: &str = "knowledgebase_raw";
pub const ROOT_SCOPE_SUBSCRIPTION_TYPE: &str = "ROOT_SCOPE_SUBSCRIPTION";
pub const MAX_WIKI_SOURCE_READ_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureKnowledgebaseRawScopeRequest {
    pub drive_space_id: String,
    pub knowledgebase_uuid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenewKnowledgebaseRawScopeEventDeliveryRequest {
    pub subscription_uuid: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeWikiDriveEventDeliveryMode {
    CloudWebhook,
    EmbeddedTrustedRelay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgebaseRawScopeEventDelivery {
    pub subscription_uuid: String,
    pub channel_id: String,
    pub expiration_epoch_ms: Option<i64>,
    pub mode: KnowledgeWikiDriveEventDeliveryMode,
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

    async fn renew_raw_scope_event_delivery(
        &self,
        _request: RenewKnowledgebaseRawScopeEventDeliveryRequest,
    ) -> Result<KnowledgebaseRawScopeEventDelivery, KnowledgeWikiDriveSourceError> {
        Err(KnowledgeWikiDriveSourceError::Upstream(
            "Drive event delivery renewal is not configured for this source adapter".to_string(),
        ))
    }
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

impl KnowledgeWikiDriveSourceError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "wiki_drive_source_request_invalid",
            Self::NotFound(_) => "wiki_drive_source_not_found",
            Self::Conflict(_) => "wiki_drive_source_conflict",
            Self::IntegrityFailed(_) => "wiki_drive_source_integrity_failed",
            Self::Upstream(_) => "wiki_drive_source_upstream_failed",
        }
    }
}
