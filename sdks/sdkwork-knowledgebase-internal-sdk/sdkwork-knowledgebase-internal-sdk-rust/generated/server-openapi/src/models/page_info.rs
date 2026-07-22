use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PageInfo {
    pub mode: String,

    #[serde(rename = "pageSize")]
    pub page_size: i64,

    #[serde(rename = "nextCursor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    #[serde(rename = "hasMore")]
    pub has_more: bool,
}
