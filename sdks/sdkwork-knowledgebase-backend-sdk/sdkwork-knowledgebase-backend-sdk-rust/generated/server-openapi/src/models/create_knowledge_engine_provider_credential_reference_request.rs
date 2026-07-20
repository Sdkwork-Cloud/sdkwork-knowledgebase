use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateKnowledgeEngineProviderCredentialReferenceRequest {
    #[serde(rename = "implementationId")]
    pub implementation_id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    #[serde(rename = "referenceLocator")]
    pub reference_locator: String,
}
