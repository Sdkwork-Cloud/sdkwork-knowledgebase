use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeProviderHealth {
    pub status: String,

    #[serde(rename = "providerId")]
    pub provider_id: String,

    #[serde(rename = "checkedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
}
