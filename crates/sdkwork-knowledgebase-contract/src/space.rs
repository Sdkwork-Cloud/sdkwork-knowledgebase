use crate::rag::KnowledgeAgentKnowledgeMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeSpaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub owner_subject_type: Option<String>,
    pub owner_subject_id: Option<String>,
    #[serde(default)]
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKnowledgeSpaceRequest {
    pub name: Option<String>,
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
    pub okf_bundle_initialized: bool,
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSpaceStatus {
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpaceList {
    pub items: Vec<KnowledgeSpace>,
}
