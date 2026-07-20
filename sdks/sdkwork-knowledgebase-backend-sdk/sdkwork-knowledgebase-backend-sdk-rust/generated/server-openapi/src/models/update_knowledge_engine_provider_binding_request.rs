use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateKnowledgeEngineProviderBindingRequest {
    #[serde(rename = "remoteResourceType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_resource_type: Option<String>,

    #[serde(rename = "remoteResourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_resource_id: Option<String>,

    #[serde(rename = "credentialReferenceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_reference_id: Option<String>,

    #[serde(rename = "clearCredentialReference")]
    pub clear_credential_reference: bool,

    #[serde(rename = "expectedVersion")]
    pub expected_version: String,
}
