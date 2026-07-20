use serde::{Deserialize, Serialize};

use crate::models::{KnowledgeEngineProviderCredentialReference, PageInfo};

/// One bounded cursor page of Provider credential references.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeEngineProviderCredentialReferencePage {
    pub items: Vec<KnowledgeEngineProviderCredentialReference>,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}
