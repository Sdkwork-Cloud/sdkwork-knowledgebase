use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpaceContextBinding {
    pub id: u64,
    pub tenant_id: u64,
    pub space_id: u64,
    pub context_type: KnowledgeContextType,
    pub context_id: String,
    pub context_name: Option<String>,
    pub access_level: KnowledgeAccessLevel,
    pub status: KnowledgeContextBindingStatus,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeSpaceContextBindingRequest {
    pub space_id: u64,
    pub context_type: KnowledgeContextType,
    pub context_id: String,
    pub context_name: Option<String>,
    pub access_level: Option<KnowledgeAccessLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKnowledgeSpaceContextBindingRequest {
    pub context_name: Option<String>,
    pub access_level: Option<KnowledgeAccessLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpaceContextBindingList {
    pub items: Vec<KnowledgeSpaceContextBinding>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListKnowledgeSpaceContextBindingsRequest {
    pub space_id: u64,
    pub context_type: Option<KnowledgeContextType>,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContextBoundSpacesRequest {
    pub context_type: KnowledgeContextType,
    pub context_id: String,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeContextType {
    ChatGroup,
    Organization,
    Circle,
    Channel,
    Team,
    Project,
}

impl KnowledgeContextType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChatGroup => "chat_group",
            Self::Organization => "organization",
            Self::Circle => "circle",
            Self::Channel => "channel",
            Self::Team => "team",
            Self::Project => "project",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "chat_group" => Some(Self::ChatGroup),
            "organization" => Some(Self::Organization),
            "circle" => Some(Self::Circle),
            "channel" => Some(Self::Channel),
            "team" => Some(Self::Team),
            "project" => Some(Self::Project),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeAccessLevel {
    Reader,
    Writer,
}

impl KnowledgeAccessLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reader => "reader",
            Self::Writer => "writer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "reader" => Some(Self::Reader),
            "writer" => Some(Self::Writer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeContextBindingStatus {
    Active,
    Deleted,
}
