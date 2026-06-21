//! Chroma connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json as collection_id_from_connector_metadata;

pub const CHROMA_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_BASE_URL";
pub const CHROMA_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_CHROMA_API_KEY";
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
        let api_key = std::env::var(CHROMA_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty());
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
