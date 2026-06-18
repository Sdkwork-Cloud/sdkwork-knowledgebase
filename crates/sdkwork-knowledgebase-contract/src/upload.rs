use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeUploadSessionRequest {
    pub space_id: u64,
    pub title: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteKnowledgeUploadSessionRequest {
    pub space_id: u64,
    pub title: String,
    pub idempotency_key: String,
    pub payload_markdown: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeUploadSession {
    pub id: u64,
    pub space_id: u64,
    pub title: String,
    pub upload_logical_path: String,
    pub status: KnowledgeUploadSessionStatus,
    pub expires_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeUploadSessionStatus {
    Pending,
    Completed,
    Expired,
}
