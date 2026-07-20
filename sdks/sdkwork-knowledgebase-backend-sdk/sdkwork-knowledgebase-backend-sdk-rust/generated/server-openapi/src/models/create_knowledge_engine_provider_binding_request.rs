use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateKnowledgeEngineProviderBindingRequest {
    #[serde(rename = "implementationId")]
    pub implementation_id: String,

    #[serde(rename = "remoteResourceType")]
    pub remote_resource_type: String,

    #[serde(rename = "remoteResourceId")]
    pub remote_resource_id: String,

    #[serde(rename = "credentialReferenceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_reference_id: Option<String>,
}
