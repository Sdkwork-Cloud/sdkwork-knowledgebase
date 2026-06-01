use async_trait::async_trait;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use thiserror::Error;

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
        state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, IngestionJobStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateIngestionJobRecord {
    pub space_id: u64,
    pub source_type: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOrGetIngestionJobResult {
    pub job: IngestionJob,
    pub created: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IngestionJobStoreError {
    #[error("ingestion job not found: {0}")]
    NotFound(u64),
    #[error("ingestion job store internal error: {0}")]
    Internal(String),
}
