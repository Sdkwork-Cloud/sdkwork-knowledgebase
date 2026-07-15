use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeAuditEventItem};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeAuditEventExport {
    pub items: Vec<KnowledgeAuditEventItem>,
}
