use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeDocument};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeDocumentList {
    pub items: Vec<KnowledgeDocument>,
}
