use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeOkfBundleFile};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeOkfBundleFileList {
    pub items: Vec<KnowledgeOkfBundleFile>,
}
