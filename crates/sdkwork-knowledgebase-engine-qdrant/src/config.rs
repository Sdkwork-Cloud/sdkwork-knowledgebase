//! Qdrant connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as collection_name_from_connector_metadata;

pub const QDRANT_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_BASE_URL";
pub const QDRANT_CREDENTIAL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_CREDENTIAL";
pub const QDRANT_CREDENTIAL_FILE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_CREDENTIAL_FILE";
pub const QDRANT_COLLECTION_NAME_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_COLLECTION_NAME";
pub const QDRANT_QUERY_MODEL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_QUERY_MODEL";
pub const QDRANT_USING_VECTOR_ENV: &str = "SDKWORK_KNOWLEDGEBASE_QDRANT_USING_VECTOR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QdrantConnectorConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_collection_name: Option<String>,
    pub query_model: Option<String>,
    pub using_vector: Option<String>,
}

impl QdrantConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(QDRANT_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = read_credential(QDRANT_CREDENTIAL_FILE_ENV, QDRANT_CREDENTIAL_ENV);
        let default_collection_name = std::env::var(QDRANT_COLLECTION_NAME_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let query_model = std::env::var(QDRANT_QUERY_MODEL_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let using_vector = std::env::var(QDRANT_USING_VECTOR_ENV)
            .ok()
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            api_key,
            default_collection_name,
            query_model,
            using_vector,
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
