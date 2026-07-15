use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeAuditEventItem {
    pub id: String,

    #[serde(rename = "eventType")]
    pub event_type: String,

    #[serde(rename = "actorType")]
    pub actor_type: String,

    #[serde(rename = "actorId")]
    pub actor_id: String,

    #[serde(rename = "resourceType")]
    pub resource_type: String,

    #[serde(rename = "resourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    pub result: String,

    #[serde(rename = "traceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: String,
}
