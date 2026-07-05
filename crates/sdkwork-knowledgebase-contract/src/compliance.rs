use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportKnowledgeAuditEventsRequest {
    pub actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAuditEventItem {
    pub id: String,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: String,
    pub trace_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAuditEventExport {
    pub items: Vec<KnowledgeAuditEventItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizeKnowledgeAuditSubjectRequest {
    pub actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizeKnowledgeAuditSubjectResult {
    pub anonymized_count: u64,
}
