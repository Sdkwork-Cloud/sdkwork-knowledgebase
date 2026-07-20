use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateKnowledgeEngineProviderMigrationOperationRequest {
    #[serde(rename = "sourceBindingId")]
    pub source_binding_id: String,

    #[serde(rename = "targetBindingId")]
    pub target_binding_id: String,

    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,

    #[serde(rename = "expectedSourceVersion")]
    pub expected_source_version: String,

    #[serde(rename = "expectedTargetVersion")]
    pub expected_target_version: String,

    #[serde(rename = "observationSeconds")]
    pub observation_seconds: i64,
}
