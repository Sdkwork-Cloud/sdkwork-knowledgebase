//! AnythingLLM connector configuration from runtime environment.

use zeroize::Zeroizing;

pub const ANYTHINGLLM_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_BASE_URL";
pub const ANYTHINGLLM_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_CREDENTIAL";
pub const ANYTHINGLLM_WORKSPACE_SLUG_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ANYTHINGLLM_WORKSPACE_SLUG";

#[derive(Clone, PartialEq, Eq)]
pub struct AnythingLlmConnectorConfig {
    pub base_url: String,
    pub api_key: Zeroizing<String>,
    pub default_workspace_slug: Option<String>,
}

impl AnythingLlmConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(ANYTHINGLLM_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = Zeroizing::new(String::new());
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
