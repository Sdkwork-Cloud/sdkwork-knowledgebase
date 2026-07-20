use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeEngineProviderMigrationOperation, PageInfo};

/// One bounded cursor page of Provider migration operations.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderMigrationOperationPage {
    pub items: Vec<KnowledgeEngineProviderMigrationOperation>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
