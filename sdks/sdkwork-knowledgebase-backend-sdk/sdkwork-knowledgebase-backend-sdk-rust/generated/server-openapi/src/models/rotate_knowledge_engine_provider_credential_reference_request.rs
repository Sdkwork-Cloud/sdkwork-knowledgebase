use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RotateKnowledgeEngineProviderCredentialReferenceRequest {
    #[serde(rename = "referenceLocator")]
    pub reference_locator: String,

    #[serde(rename = "expectedVersion")]
    pub expected_version: String,
}
