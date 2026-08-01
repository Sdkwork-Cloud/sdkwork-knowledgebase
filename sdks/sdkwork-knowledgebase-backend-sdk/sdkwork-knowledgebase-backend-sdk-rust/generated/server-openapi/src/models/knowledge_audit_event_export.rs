use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeAuditEventItem};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeAuditEventExport {
    /// Complete synchronous result for the subject. Requests matching more than 5,000 events fail with HTTP 413 and never return a truncated array.
    pub items: Vec<KnowledgeAuditEventItem>,
}
