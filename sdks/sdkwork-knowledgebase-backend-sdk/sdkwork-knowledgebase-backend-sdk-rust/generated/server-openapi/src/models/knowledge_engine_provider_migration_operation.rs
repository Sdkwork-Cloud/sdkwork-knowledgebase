use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderMigrationOperation {
    pub id: String,

    pub uuid: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "organizationId")]
    pub organization_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "sourceBindingId")]
    pub source_binding_id: String,

    #[serde(rename = "targetBindingId")]
    pub target_binding_id: String,

    #[serde(rename = "operationState")]
    pub operation_state: String,

    #[serde(rename = "requestedBy")]
    pub requested_by: String,

    #[serde(rename = "attemptCount")]
    pub attempt_count: i64,

    #[serde(rename = "cutoverAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cutover_at: Option<String>,

    #[serde(rename = "observationUntil")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_until: Option<String>,

    #[serde(rename = "completedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,

    #[serde(rename = "lastErrorCategory")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_category: Option<String>,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,

    pub version: String,
}
