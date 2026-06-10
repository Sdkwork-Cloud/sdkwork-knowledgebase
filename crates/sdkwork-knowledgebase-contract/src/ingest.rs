use crate::document::{KnowledgeDocument, KnowledgeDocumentVersion};
use crate::drive::KnowledgeDriveObjectRef;
use crate::source::KnowledgeSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIngestionJobRequest {
    pub space_id: u64,
    pub source_type: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIngestRequest {
    pub space_id: u64,
    pub title: String,
    pub payload_markdown: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDriveImportRequest {
    pub space_id: u64,
    pub title: String,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub drive_storage_provider_id: String,
    pub drive_bucket: String,
    pub drive_object_key: String,
    pub idempotency_key: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDriveImportResult {
    pub source: KnowledgeSource,
    pub document: KnowledgeDocument,
    pub version: KnowledgeDocumentVersion,
    pub original_object_ref: KnowledgeDriveObjectRef,
    pub job: IngestionJob,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestionJob {
    pub id: u64,
    pub space_id: u64,
    pub source_type: String,
    pub idempotency_key: String,
    pub state: IngestionJobState,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionJobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
