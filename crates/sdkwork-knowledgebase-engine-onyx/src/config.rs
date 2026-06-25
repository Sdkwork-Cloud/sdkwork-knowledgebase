//! Onyx connector configuration from runtime environment.

pub const ONYX_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_BASE_URL";
pub const ONYX_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_API_KEY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnyxConnectorConfig {
    pub base_url: String,
    pub api_key: String,
}

impl OnyxConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(ONYX_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(ONYX_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty())?;

        Some(Self { base_url, api_key })
    }
}
