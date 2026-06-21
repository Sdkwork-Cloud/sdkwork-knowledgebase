//! Haystack connector configuration from runtime environment.

pub use sdkwork_knowledgebase_contract::source::{
    dataset_id_from_connector_metadata_json as pipeline_name_from_connector_metadata,
    workspace_slug_from_connector_metadata_json as workspace_name_from_connector_metadata,
};

pub const HAYSTACK_BASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_BASE_URL";
pub const HAYSTACK_API_KEY_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_API_KEY";
pub const HAYSTACK_PIPELINE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_PIPELINE";
pub const HAYSTACK_WORKSPACE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_WORKSPACE";
pub const HAYSTACK_DEPLOYMENT_MODE_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_DEPLOYMENT_MODE";
pub const HAYSTACK_QUERY_FIELD_ENV: &str = "SDKWORK_KNOWLEDGEBASE_HAYSTACK_QUERY_FIELD";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaystackDeploymentMode {
    Hayhooks,
    DeepsetCloud,
}

impl HaystackDeploymentMode {
    pub fn from_env_value(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "hayhooks" | "self_hosted" | "self-hosted" => Some(Self::Hayhooks),
            "cloud" | "deepset" | "deepset_cloud" | "deepset-cloud" => Some(Self::DeepsetCloud),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HaystackConnectorConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_pipeline: Option<String>,
    pub default_workspace: Option<String>,
    pub deployment_mode: HaystackDeploymentMode,
    pub query_field: String,
}

impl HaystackConnectorConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var(HAYSTACK_BASE_URL_ENV)
            .ok()
            .map(|value| value.trim_end_matches('/').to_string())
            .filter(|value| !value.is_empty())?;
        let api_key = std::env::var(HAYSTACK_API_KEY_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let default_pipeline = std::env::var(HAYSTACK_PIPELINE_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let default_workspace = std::env::var(HAYSTACK_WORKSPACE_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        let deployment_mode = std::env::var(HAYSTACK_DEPLOYMENT_MODE_ENV)
            .ok()
            .and_then(|value| HaystackDeploymentMode::from_env_value(&value))
            .unwrap_or_else(|| {
                if base_url.contains("deepset.ai") {
                    HaystackDeploymentMode::DeepsetCloud
                } else {
                    HaystackDeploymentMode::Hayhooks
                }
            });
        let query_field = std::env::var(HAYSTACK_QUERY_FIELD_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "query".to_string());

        Some(Self {
            base_url,
            api_key,
            default_pipeline,
            default_workspace,
            deployment_mode,
            query_field,
        })
    }
}
