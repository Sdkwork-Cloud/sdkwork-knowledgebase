use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeRetrievalTrace};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeRetrievalTraceList {
    pub items: Vec<KnowledgeRetrievalTrace>,
}
