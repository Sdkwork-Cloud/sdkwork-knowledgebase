use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSpaceMemberSubjectType {
    User,
    Group,
    Domain,
    App,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeSpaceMemberRole {
    Reader,
    Writer,
    Owner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpaceMember {
    pub subject_type: KnowledgeSpaceMemberSubjectType,
    pub subject_id: String,
    pub role: KnowledgeSpaceMemberRole,
    pub inherited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSpaceMemberList {
    pub members: Vec<KnowledgeSpaceMember>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantKnowledgeSpaceMemberRequest {
    pub subject_type: KnowledgeSpaceMemberSubjectType,
    pub subject_id: String,
    pub role: KnowledgeSpaceMemberRole,
}
