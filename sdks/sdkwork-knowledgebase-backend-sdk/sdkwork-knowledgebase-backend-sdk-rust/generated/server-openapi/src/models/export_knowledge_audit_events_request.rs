use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExportKnowledgeAuditEventsRequest {
    #[serde(rename = "actorId")]
    pub actor_id: String,
}
