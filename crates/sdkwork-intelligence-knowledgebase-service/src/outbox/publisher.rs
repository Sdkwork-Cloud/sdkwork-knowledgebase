use crate::ports::knowledge_outbox_store::{KnowledgeOutboxStore, KnowledgeOutboxStoreError};
use thiserror::Error;

pub struct KnowledgeOutboxPublisherService<'a> {
    outbox: &'a dyn KnowledgeOutboxStore,
}

impl<'a> KnowledgeOutboxPublisherService<'a> {
    pub fn new(outbox: &'a dyn KnowledgeOutboxStore) -> Self {
        Self { outbox }
    }

    pub async fn publish_pending(
        &self,
        limit: u32,
    ) -> Result<OutboxPublishBatchResult, KnowledgeOutboxPublisherServiceError> {
        let pending = self
            .outbox
            .list_pending_events(limit)
            .await
            .map_err(KnowledgeOutboxPublisherServiceError::Store)?;

        let mut published = 0usize;
        for event in pending {
            tracing::info!(
                event_id = event.id,
                event_type = %event.event_type,
                aggregate_type = %event.aggregate_type,
                aggregate_id = event.aggregate_id,
                "publishing knowledgebase outbox event"
            );
            self.outbox
                .mark_published(event.id)
                .await
                .map_err(KnowledgeOutboxPublisherServiceError::Store)?;
            published += 1;
        }

        Ok(OutboxPublishBatchResult { published })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxPublishBatchResult {
    pub published: usize,
}

#[derive(Debug, Error)]
pub enum KnowledgeOutboxPublisherServiceError {
    #[error(transparent)]
    Store(#[from] KnowledgeOutboxStoreError),
}
