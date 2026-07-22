use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DriveCloudEvent {
    pub specversion: String,

    pub id: String,

    pub source: String,

    pub r#type: String,

    pub time: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "organizationId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    #[serde(rename = "actorId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,

    #[serde(rename = "sequenceNo")]
    pub sequence_no: String,

    pub data: std::collections::HashMap<String, serde_json::Value>,
}
