//! Open WebUI connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as knowledge_id_from_connector_metadata;

pub const OPEN_WEBUI_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_BASE_URL";
pub const OPEN_WEBUI_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_API_KEY";
pub const OPEN_WEBUI_KNOWLEDGE_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_KNOWLEDGE_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWebuiConnectorConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_knowledge_id: Option<String>,
}

impl OpenWebuiConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(OPEN_WEBUI_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(OPEN_WEBUI_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty())?;
        let default_knowledge_id = std::env::var(OPEN_WEBUI_KNOWLEDGE_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_knowledge_id,
        })
    }
}
