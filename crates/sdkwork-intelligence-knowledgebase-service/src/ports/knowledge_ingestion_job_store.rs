use async_trait::async_trait;
use sdkwork_knowledgebase_contract::drive::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use thiserror::Error;

use crate::ports::knowledge_chunk_store::CreateKnowledgeChunkRecord;
use crate::ports::knowledge_outbox_store::AppendOutboxEventRecord;

#[async_trait]
pub trait IngestionJobStore: Send + Sync {
    async fn create_or_get_job(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<CreateOrGetIngestionJobResult, IngestionJobStoreError>;

    async fn get_job(&self, job_id: u64) -> Result<IngestionJob, IngestionJobStoreError>;

    async fn update_job_state(
        &self,
        job_id: u64,
        expected_state: IngestionJobState,
        state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, IngestionJobStoreError>;

    async fn list_jobs_by_state(
        &self,
        state: IngestionJobState,
        limit: u32,
    ) -> Result<Vec<IngestionJob>, IngestionJobStoreError>;

    async fn attach_drive_import_linkage(
        &self,
        job_id: u64,
        linkage: DriveImportJobLinkage,
    ) -> Result<(), IngestionJobStoreError>;

    async fn get_drive_import_linkage(
        &self,
        job_id: u64,
    ) -> Result<Option<DriveImportJobLinkage>, IngestionJobStoreError>;

    async fn mark_running_job_succeeded_with_outbox(
        &self,
        job_id: u64,
        outbox: AppendOutboxEventRecord,
    ) -> Result<IngestionJob, IngestionJobStoreError>;

    async fn complete_running_ingestion_with_chunks_and_outbox(
        &self,
        record: CompleteRunningIngestionRecord,
    ) -> Result<CompletedIngestionResult, IngestionJobStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteRunningIngestionRecord {
    pub job_id: u64,
    pub document_version_id: u64,
    pub chunks: Vec<CreateKnowledgeChunkRecord>,
    pub outbox: AppendOutboxEventRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveImportJobLinkage {
    pub source_id: u64,
    pub document_id: u64,
    pub document_version_id: u64,
    pub original_object_ref: KnowledgeDriveObjectRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateIngestionJobRecord {
    pub space_id: u64,
    pub source_type: String,
    pub idempotency_key: String,
    pub idempotency_fingerprint_sha256_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOrGetIngestionJobResult {
    pub job: IngestionJob,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedIngestionResult {
    pub job: IngestionJob,
    pub chunk_count: usize,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IngestionJobStoreError {
    #[error("ingestion job not found: {0}")]
    NotFound(u64),
    #[error("ingestion job conflict: {0}")]
    Conflict(String),
    #[error("ingestion job store internal error: {0}")]
    Internal(String),
}
