//! Flowise connector configuration from runtime environment.

use zeroize::Zeroizing;

pub const FLOWISE_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_FLOWISE_BASE_URL";
pub const FLOWISE_STORE_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_FLOWISE_STORE_ID";

#[derive(Clone, PartialEq, Eq)]
pub struct FlowiseConnectorConfig {
    pub base_url: String,
    pub api_key: Zeroizing<String>,
    pub default_store_id: Option<String>,
}

impl FlowiseConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(FLOWISE_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = Zeroizing::new(String::new());
        let default_store_id = std::env::var(FLOWISE_STORE_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_store_id,
        })
    }
}
