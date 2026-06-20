use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfBundleFile {
    pub id: u64,
    pub space_id: u64,
    pub logical_path: String,
    pub file_kind: OkfBundleFileKind,
    pub artifact_role: String,
    pub drive_bucket: String,
    pub drive_object_key: String,
    pub checksum_sha256_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staged_import_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfBundleFileList {
    pub items: Vec<KnowledgeOkfBundleFile>,
}

pub use crate::enums::OkfBundleFileKind;
