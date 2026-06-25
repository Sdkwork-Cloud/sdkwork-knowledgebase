use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeDocumentRequest {
    pub space_id: u64,
    pub collection_id: Option<u64>,
    pub source_id: Option<u64>,
    pub title: String,
    pub mime_type: Option<String>,
    pub language: Option<String>,
    pub visibility: Option<KnowledgeDocumentVisibility>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeDocumentVersionRequest {
    pub document_id: u64,
    pub original_object_ref_id: u64,
    pub checksum_sha256_hex: Option<String>,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDocumentContent {
    pub document_id: u64,
    pub content_markdown: String,
    pub content_source: String,
    pub content_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDocumentList {
    pub items: Vec<KnowledgeDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDocumentVersionList {
    pub items: Vec<KnowledgeDocumentVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDocument {
    pub id: u64,
    pub space_id: u64,
    pub collection_id: u64,
    pub source_id: Option<u64>,
    pub original_file_drive_node_id: Option<String>,
    pub title: String,
    pub mime_type: Option<String>,
    pub language: Option<String>,
    pub current_version_id: Option<u64>,
    pub visibility: KnowledgeDocumentVisibility,
    pub content_state: KnowledgeDocumentState,
    pub index_state: KnowledgeDocumentVersionState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDocumentVersion {
    pub id: u64,
    pub document_id: u64,
    pub version_no: u64,
    pub original_object_ref_id: u64,
    pub checksum_sha256_hex: Option<String>,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub parse_state: KnowledgeDocumentVersionState,
    pub index_state: KnowledgeDocumentVersionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeDocumentVisibility {
    Private,
    Space,
    Organization,
    Public,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeDocumentState {
    Draft,
    Ready,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeDocumentVersionState {
    Pending,
    Running,
    Succeeded,
    Failed,
}
