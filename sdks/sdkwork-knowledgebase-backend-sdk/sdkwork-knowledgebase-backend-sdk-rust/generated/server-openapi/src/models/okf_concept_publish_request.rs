use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfConceptPublishRequest {
    #[serde(rename = "publisherId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_id: Option<i64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
