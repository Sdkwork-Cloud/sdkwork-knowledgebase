use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeOutboxStore: Send + Sync {
    async fn append_event(
        &self,
        record: AppendOutboxEventRecord,
    ) -> Result<(), KnowledgeOutboxStoreError>;

    async fn list_pending_events(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError>;

    async fn claim_pending_events(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError>;

    async fn release_stale_claimed_events(
        &self,
        stale_after_secs: u64,
    ) -> Result<usize, KnowledgeOutboxStoreError>;

    async fn mark_published(&self, event_id: u64) -> Result<(), KnowledgeOutboxStoreError>;

    async fn mark_failed(
        &self,
        event_id: u64,
        error_message: &str,
    ) -> Result<(), KnowledgeOutboxStoreError>;

    async fn requeue_failed_events(
        &self,
        limit: u32,
        max_retry_count: u32,
    ) -> Result<usize, KnowledgeOutboxStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOutboxEventRecord {
    pub aggregate_type: String,
    pub aggregate_id: u64,
    pub event_type: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOutboxEvent {
    pub id: u64,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: u64,
    pub payload_json: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeOutboxStoreError {
    #[error("invalid outbox event: {0}")]
    InvalidRequest(String),
    #[error("outbox store internal error: {0}")]
    Internal(String),
}
