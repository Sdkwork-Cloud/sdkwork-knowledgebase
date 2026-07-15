use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeSpaceMember};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeSpaceMemberList {
    pub members: Vec<KnowledgeSpaceMember>,

    #[serde(rename = "nextCursor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
