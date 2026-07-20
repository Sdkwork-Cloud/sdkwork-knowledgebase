//! Open WebUI connector configuration from runtime environment.

pub const OPEN_WEBUI_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_BASE_URL";
pub const OPEN_WEBUI_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_CREDENTIAL";
pub const OPEN_WEBUI_CREDENTIAL_FILE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_CREDENTIAL_FILE";
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
        let api_key = read_credential(OPEN_WEBUI_CREDENTIAL_FILE_ENV, OPEN_WEBUI_CREDENTIAL_ENV)?;
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
