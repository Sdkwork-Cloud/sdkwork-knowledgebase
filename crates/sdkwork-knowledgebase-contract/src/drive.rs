use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDriveObjectRef {
    pub id: u64,
    pub space_id: u64,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub logical_path: Option<String>,
    #[serde(skip_serializing)]
    pub drive_provider_kind: String,
    #[serde(skip_serializing)]
    pub drive_storage_provider_id: String,
    #[serde(skip_serializing)]
    pub drive_bucket: String,
    #[serde(skip_serializing)]
    pub drive_object_key: String,
    #[serde(skip_serializing)]
    pub drive_object_version: Option<String>,
    #[serde(skip_serializing)]
    pub drive_etag: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub checksum_sha256_hex: Option<String>,
    pub object_role: String,
    pub access_mode: String,
}
