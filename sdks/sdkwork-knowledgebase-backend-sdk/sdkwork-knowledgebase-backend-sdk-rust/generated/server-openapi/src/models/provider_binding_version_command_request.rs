use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProviderBindingVersionCommandRequest {
    #[serde(rename = "expectedVersion")]
    pub expected_version: String,
}
