use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeEngineProviderBinding, PageInfo};

/// One bounded cursor page of Provider bindings.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderBindingPage {
    pub items: Vec<KnowledgeEngineProviderBinding>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
