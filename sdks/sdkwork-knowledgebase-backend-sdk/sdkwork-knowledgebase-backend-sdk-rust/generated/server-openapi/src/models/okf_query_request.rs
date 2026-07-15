use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfQueryRequest {
    #[serde(rename = "spaceId")]
    pub space_id: i64,

    pub query: String,
}
