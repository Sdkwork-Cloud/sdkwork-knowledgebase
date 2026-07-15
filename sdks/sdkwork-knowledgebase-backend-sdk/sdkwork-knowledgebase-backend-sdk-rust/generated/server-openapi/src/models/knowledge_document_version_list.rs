use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeDocumentVersion};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeDocumentVersionList {
    pub items: Vec<KnowledgeDocumentVersion>,
}
