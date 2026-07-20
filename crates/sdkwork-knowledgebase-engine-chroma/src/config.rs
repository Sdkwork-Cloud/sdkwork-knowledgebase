//! Chroma connector configuration from runtime environment.

pub const CHROMA_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_BASE_URL";
pub const CHROMA_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_CREDENTIAL";
pub const CHROMA_CREDENTIAL_FILE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_CREDENTIAL_FILE";
pub const CHROMA_COLLECTION_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_COLLECTION_ID";
pub const CHROMA_TENANT_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_TENANT";
pub const CHROMA_DATABASE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_DATABASE";

pub const DEFAULT_CHROMA_TENANT: &str = "default_tenant";
pub const DEFAULT_CHROMA_DATABASE: &str = "default_database";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromaConnectorConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_collection_id: Option<String>,
    pub tenant: String,
    pub database: String,
}

impl ChromaConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(CHROMA_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = read_credential(CHROMA_CREDENTIAL_FILE_ENV, CHROMA_CREDENTIAL_ENV);
        let default_collection_id = std::env::var(CHROMA_COLLECTION_ID_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let tenant = std::env::var(CHROMA_TENANT_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_CHROMA_TENANT.to_string());
        let database = std::env::var(CHROMA_DATABASE_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_CHROMA_DATABASE.to_string());

        Some(Self {
            base_url,
            api_key,
            default_collection_id,
            tenant,
            database,
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
