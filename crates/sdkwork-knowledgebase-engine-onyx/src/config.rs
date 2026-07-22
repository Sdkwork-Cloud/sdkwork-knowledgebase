//! Onyx connector configuration from runtime environment.

use zeroize::Zeroizing;

pub const ONYX_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_BASE_URL";

#[derive(Clone, PartialEq, Eq)]
pub struct OnyxConnectorConfig {
    pub base_url: String,
    pub api_key: Zeroizing<String>,
}

impl OnyxConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(ONYX_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = Zeroizing::new(String::new());

        Some(Self { base_url, api_key })
    }
}
