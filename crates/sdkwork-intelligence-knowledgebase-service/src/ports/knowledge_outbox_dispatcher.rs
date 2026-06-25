use async_trait::async_trait;
use thiserror::Error;

use super::knowledge_outbox_store::PendingOutboxEvent;

#[async_trait]
pub trait KnowledgeOutboxDispatcher: Send + Sync {
    async fn dispatch(
        &self,
        tenant_id: u64,
        event: &PendingOutboxEvent,
    ) -> Result<(), KnowledgeOutboxDispatchError>;
}

#[derive(Debug, Error)]
pub enum KnowledgeOutboxDispatchError {
    #[error("outbox dispatch failed: {0}")]
    DeliveryFailed(String),
    #[error("outbox dispatch internal error: {0}")]
    Internal(String),
}
