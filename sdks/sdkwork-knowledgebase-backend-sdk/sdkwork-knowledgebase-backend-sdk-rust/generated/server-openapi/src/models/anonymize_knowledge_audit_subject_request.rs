use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnonymizeKnowledgeAuditSubjectRequest {
    /// Non-blank IAM subject identifier.
    #[serde(rename = "actorId")]
    pub actor_id: String,
}
