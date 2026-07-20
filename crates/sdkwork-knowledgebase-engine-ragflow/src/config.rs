//! RAGFlow connector configuration from runtime environment.

use zeroize::Zeroizing;

pub const RAGFLOW_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_BASE_URL";
pub const RAGFLOW_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_CREDENTIAL";
pub const RAGFLOW_DATASET_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RAGFLOW_DATASET_ID";

#[derive(Clone, PartialEq, Eq)]
pub struct RagflowConnectorConfig {
    pub base_url: String,
    pub api_key: Zeroizing<String>,
    pub default_dataset_id: Option<String>,
}

impl RagflowConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(RAGFLOW_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = Zeroizing::new(String::new());
        let default_dataset_id = std::env::var(RAGFLOW_DATASET_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_dataset_id,
        })
    }
}
