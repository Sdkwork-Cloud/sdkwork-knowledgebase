use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KnowledgeIndex {
    #[serde(rename = "indexId")]
    pub index_id: String,

    #[serde(rename = "tenantId")]
    pub tenant_id: String,

    #[serde(rename = "spaceId")]
    pub space_id: String,

    #[serde(rename = "indexKind")]
    pub index_kind: String,

    pub status: String,
}
