use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkfLogEntry {
    #[serde(rename = "occurredAt")]
    pub occurred_at: String,

    #[serde(rename = "eventType")]
    pub event_type: String,

    pub title: String,

    pub actor: String,

    #[serde(rename = "affectedPages")]
    pub affected_pages: Vec<String>,

    #[serde(rename = "auditEventId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_event_id: Option<String>,

    pub warnings: Vec<String>,
}
