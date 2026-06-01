use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSource {
    pub id: u64,
    pub space_id: u64,
    pub source_type: KnowledgeSourceType,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSourceType {
    Upload,
    DriveObject,
    DriveFolder,
    Url,
    Connector,
    Api,
}

impl KnowledgeSourceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::DriveObject => "drive_object",
            Self::DriveFolder => "drive_folder",
            Self::Url => "url",
            Self::Connector => "connector",
            Self::Api => "api",
        }
    }
}
