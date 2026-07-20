use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderCredentialReference {
    pub id: String,

    pub uuid: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "organizationId")]
    pub organization_id: String,

    #[serde(rename = "implementationId")]
    pub implementation_id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "rotationState")]
    pub rotation_state: String,

    #[serde(rename = "lastRotatedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_rotated_at: Option<String>,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "updatedBy")]
    pub updated_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "updatedAt")]
    pub updated_at: String,

    pub version: String,
}
