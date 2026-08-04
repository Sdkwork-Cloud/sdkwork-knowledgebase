use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupKnowledgebaseLaunchCapability {
    /// Whether managed group knowledgebase launch is configured for this deployment.
    pub state: String,
}
