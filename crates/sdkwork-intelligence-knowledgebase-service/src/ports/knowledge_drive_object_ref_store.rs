use async_trait::async_trait;
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use thiserror::Error;

pub const SDKWORK_DRIVE_PROVIDER_KIND: &str = "sdkwork-drive";
pub const MANAGED_DRIVE_ACCESS_MODE: &str = "managed";

#[async_trait]
pub trait KnowledgeDriveObjectRefStore: Send + Sync {
    async fn create_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError>;

    async fn create_or_get_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        self.create_object_ref(record).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeDriveObjectRefRecord {
    pub space_id: u64,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub logical_path: Option<String>,
    pub drive_provider_kind: String,
    pub drive_storage_provider_id: String,
    pub drive_bucket: String,
    pub drive_object_key: String,
    pub drive_object_version: Option<String>,
    pub drive_etag: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub checksum_sha256_hex: Option<String>,
    pub object_role: String,
    pub access_mode: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeDriveObjectRefStoreError {
    #[error("knowledge drive object ref store internal error: {0}")]
    Internal(String),
}
