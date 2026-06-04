use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeSpaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpace {
    pub id: u64,
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub drive_space_id: Option<String>,
    pub status: KnowledgeSpaceStatus,
    pub llm_wiki_initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSpaceStatus {
    Active,
    Archived,
    Deleted,
}
