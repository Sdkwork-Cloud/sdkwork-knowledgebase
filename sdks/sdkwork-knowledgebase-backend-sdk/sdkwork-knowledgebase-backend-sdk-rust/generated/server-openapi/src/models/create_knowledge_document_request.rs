use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateKnowledgeDocumentRequest {
    #[serde(rename = "spaceId")]
    pub space_id: i64,

    #[serde(rename = "collectionId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<i64>,

    #[serde(rename = "sourceId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<i64>,

    pub title: String,

    #[serde(rename = "mimeType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
