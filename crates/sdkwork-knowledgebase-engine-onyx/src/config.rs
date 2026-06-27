//! Onyx connector configuration from runtime environment.

pub const ONYX_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_BASE_URL";
pub const ONYX_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_CREDENTIAL";
pub const ONYX_CREDENTIAL_FILE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ONYX_CREDENTIAL_FILE";

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
        let api_key = read_credential(ONYX_CREDENTIAL_FILE_ENV, ONYX_CREDENTIAL_ENV)?;

        Some(Self { base_url, api_key })
    }
}

/// Resolve a credential value, preferring file-based input over inline env var.
///
/// Production deployments mount credentials via `*_CREDENTIAL_FILE` (e.g. a
/// Kubernetes secret mount). Development profiles may set `*_CREDENTIAL`
/// directly. This helper never logs the resolved value.
fn read_credential(file_env: &str, inline_env: &str) -> Option<String> {
    if let Ok(path) = std::env::var(file_env) {
        if !path.is_empty() {
            return std::fs::read_to_string(&path)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
        }
    }
    std::env::var(inline_env)
        .ok()
        .filter(|value| !value.is_empty())
}
