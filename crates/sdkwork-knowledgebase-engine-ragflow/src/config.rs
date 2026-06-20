//! RAGFlow connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as dataset_id_from_connector_metadata;

pub const RAGFLOW_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_BASE_URL";
pub const RAGFLOW_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_API_KEY";
pub const RAGFLOW_DATASET_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_DATASET_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RagflowConnectorConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_dataset_id: Option<String>,
}

impl RagflowConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(RAGFLOW_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(RAGFLOW_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty())?;
        let default_dataset_id = std::env::var(RAGFLOW_DATASET_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_dataset_id,
        })
    }
}
