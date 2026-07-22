use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResolveWikiRouteRequest {
    pub route: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}
