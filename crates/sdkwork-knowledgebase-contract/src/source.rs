use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeSourceRequest {
    pub space_id: u64,
    pub source_type: KnowledgeSourceType,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
    pub connector_metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSourceList {
    pub items: Vec<KnowledgeSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSource {
    pub id: u64,
    pub space_id: u64,
    pub source_type: KnowledgeSourceType,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
    pub connector_metadata_json: Option<String>,
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

/// Per-space connector metadata for external knowledge engine adapters (`kb_source.connector_metadata_json`).
#[derive(Debug, Deserialize)]
pub struct ExternalConnectorMetadata {
    #[serde(rename = "datasetId", alias = "dataset_id")]
    pub dataset_id: Option<String>,
}

/// Resolves a dataset/knowledge-base id from connector metadata JSON when present.
pub fn dataset_id_from_connector_metadata_json(metadata_json: Option<&str>) -> Option<String> {
    let raw = metadata_json?.trim();
    if raw.is_empty() {
        return None;
    }
    serde_json::from_str::<ExternalConnectorMetadata>(raw)
        .ok()
        .and_then(|metadata| metadata.dataset_id)
        .filter(|value| !value.is_empty())
}
