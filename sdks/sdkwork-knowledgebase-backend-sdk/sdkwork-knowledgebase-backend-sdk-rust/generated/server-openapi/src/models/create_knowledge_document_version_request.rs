use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateKnowledgeDocumentVersionRequest {
    #[serde(rename = "documentId")]
    pub document_id: i64,

    #[serde(rename = "originalObjectRefId")]
    pub original_object_ref_id: i64,

    #[serde(rename = "checksumSha256Hex")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum_sha256_hex: Option<String>,

    #[serde(rename = "sizeBytes")]
    pub size_bytes: i64,

    #[serde(rename = "mimeType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}
