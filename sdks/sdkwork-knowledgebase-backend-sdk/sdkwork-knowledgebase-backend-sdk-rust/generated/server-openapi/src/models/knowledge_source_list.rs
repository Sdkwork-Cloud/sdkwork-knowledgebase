use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeSource};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeSourceList {
    pub items: Vec<KnowledgeSource>,
}
