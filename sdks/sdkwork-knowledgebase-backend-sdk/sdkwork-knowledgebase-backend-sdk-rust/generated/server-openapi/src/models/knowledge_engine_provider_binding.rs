use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderBinding {
    pub id: String,

    pub uuid: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "organizationId")]
    pub organization_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "implementationId")]
    pub implementation_id: String,

    #[serde(rename = "remoteResourceType")]
    pub remote_resource_type: String,

    #[serde(rename = "remoteResourceId")]
    pub remote_resource_id: String,

    #[serde(rename = "credentialReferenceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_reference_id: Option<String>,

    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: String,

    #[serde(rename = "capabilitySnapshot")]
    pub capability_snapshot: Vec<String>,

    #[serde(rename = "capabilitySnapshotVersion")]
    pub capability_snapshot_version: String,

    #[serde(rename = "lastTestedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tested_at: Option<String>,

    #[serde(rename = "activatedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<String>,

    #[serde(rename = "disabledAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<String>,

    #[serde(rename = "lastErrorCategory")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_category: Option<String>,

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
