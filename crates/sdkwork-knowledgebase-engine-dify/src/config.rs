//! Dify connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as dataset_id_from_connector_metadata;

pub const DIFY_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_BASE_URL";
pub const DIFY_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_API_KEY";
pub const DIFY_DATASET_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_DATASET_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifyConnectorConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_dataset_id: Option<String>,
}

impl DifyConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(DIFY_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(DIFY_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty())?;
        let default_dataset_id = std::env::var(DIFY_DATASET_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_dataset_id,
        })
    }
}
