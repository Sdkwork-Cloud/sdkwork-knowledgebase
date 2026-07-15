use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnonymizeKnowledgeAuditSubjectRequest {
    #[serde(rename = "actorId")]
    pub actor_id: String,
}
