//! Open WebUI connector configuration from runtime environment.

use zeroize::Zeroizing;

pub const OPEN_WEBUI_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_BASE_URL";
pub const OPEN_WEBUI_KNOWLEDGE_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPEN_WEBUI_KNOWLEDGE_ID";

#[derive(Clone, PartialEq, Eq)]
pub struct OpenWebuiConnectorConfig {
    pub base_url: String,
    pub api_key: Zeroizing<String>,
    pub default_knowledge_id: Option<String>,
}

impl OpenWebuiConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(OPEN_WEBUI_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = Zeroizing::new(String::new());
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
