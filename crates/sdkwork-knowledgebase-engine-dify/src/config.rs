//! Dify connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as dataset_id_from_connector_metadata;

pub const DIFY_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_BASE_URL";
pub const DIFY_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_CREDENTIAL";
pub const DIFY_CREDENTIAL_FILE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_CREDENTIAL_FILE";
pub const DIFY_DATASET_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DIFY_DATASET_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifyConnectorConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_dataset_id: Option<String>,
}

impl DifyConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(DIFY_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = read_credential(DIFY_CREDENTIAL_FILE_ENV, DIFY_CREDENTIAL_ENV)?;
        let default_dataset_id = std::env::var(DIFY_DATASET_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_dataset_id,
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
