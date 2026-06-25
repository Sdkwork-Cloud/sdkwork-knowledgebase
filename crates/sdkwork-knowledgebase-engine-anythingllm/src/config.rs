//! AnythingLLM connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::workspace_slug_from_connector_metadata_json as workspace_slug_from_connector_metadata;

pub const ANYTHINGLLM_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_BASE_URL";
pub const ANYTHINGLLM_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_API_KEY";
pub const ANYTHINGLLM_WORKSPACE_SLUG_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_WORKSPACE_SLUG";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnythingLlmConnectorConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_workspace_slug: Option<String>,
}

impl AnythingLlmConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(ANYTHINGLLM_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(ANYTHINGLLM_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty())?;
        let default_workspace_slug = std::env::var(ANYTHINGLLM_WORKSPACE_SLUG_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_workspace_slug,
        })
    }
}
