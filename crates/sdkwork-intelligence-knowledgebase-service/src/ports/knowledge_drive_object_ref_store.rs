use async_trait::async_trait;
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use thiserror::Error;

use super::knowledge_drive_storage::KnowledgeObjectRef;

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

    async fn list_object_refs_by_logical_path_prefix(
        &self,
        space_id: u64,
        prefix: &str,
    ) -> Result<Vec<KnowledgeDriveObjectRef>, KnowledgeDriveObjectRefStoreError>;

    async fn get_object_ref_by_id(
        &self,
        object_ref_id: u64,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError>;
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

pub fn managed_drive_object_ref_record(
    space_id: u64,
    object_ref: &KnowledgeObjectRef,
    drive_space_id: Option<&str>,
    drive_node_id: Option<String>,
) -> CreateKnowledgeDriveObjectRefRecord {
    CreateKnowledgeDriveObjectRefRecord {
        space_id,
        drive_space_id: drive_space_id.map(str::to_string),
        drive_node_id,
        logical_path: Some(object_ref.logical_path.clone()),
        drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
        drive_storage_provider_id: object_ref.storage_provider_id.clone(),
        drive_bucket: object_ref.bucket.clone(),
        drive_object_key: object_ref.object_key.clone(),
        drive_object_version: object_ref.version_id.clone(),
        drive_etag: object_ref.etag.clone(),
        content_type: Some(object_ref.content_type.clone()),
        size_bytes: object_ref.size_bytes,
        checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
        object_role: object_ref.object_role.clone(),
        access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeDriveObjectRefStoreError {
    #[error("knowledge drive object ref store internal error: {0}")]
    Internal(String),
}
