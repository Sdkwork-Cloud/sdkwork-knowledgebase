use crate::ports::knowledge_outbox_dispatcher::KnowledgeOutboxDispatcher;
use crate::ports::knowledge_outbox_store::{KnowledgeOutboxStore, KnowledgeOutboxStoreError};
use thiserror::Error;

pub struct KnowledgeOutboxPublisherService<'a> {
    outbox: &'a dyn KnowledgeOutboxStore,
    dispatcher: &'a dyn KnowledgeOutboxDispatcher,
    tenant_id: u64,
}

impl<'a> KnowledgeOutboxPublisherService<'a> {
    pub fn new(
        tenant_id: u64,
        outbox: &'a dyn KnowledgeOutboxStore,
        dispatcher: &'a dyn KnowledgeOutboxDispatcher,
    ) -> Self {
        Self {
            outbox,
            dispatcher,
            tenant_id,
        }
    }

    pub async fn publish_pending(
        &self,
        limit: u32,
    ) -> Result<OutboxPublishBatchResult, KnowledgeOutboxPublisherServiceError> {
        let pending = self
            .outbox
            .claim_pending_events(limit)
            .await
            .map_err(KnowledgeOutboxPublisherServiceError::Store)?;

        let mut published = 0usize;
        let mut failed = 0usize;
        for event in pending {
            tracing::info!(
                event_id = event.id,
                event_type = %event.event_type,
                aggregate_type = %event.aggregate_type,
                aggregate_id = event.aggregate_id,
                "dispatching knowledgebase outbox event"
            );
            match self.dispatcher.dispatch(self.tenant_id, &event).await {
                Ok(()) => {
                    self.outbox
                        .mark_published(event.id)
                        .await
                        .map_err(KnowledgeOutboxPublisherServiceError::Store)?;
                    published += 1;
                }
                Err(error) => {
                    tracing::warn!(
                        event_id = event.id,
                        error = %error,
                        "knowledgebase outbox dispatch failed"
                    );
                    self.outbox
                        .mark_failed(event.id, &error.to_string())
                        .await
                        .map_err(KnowledgeOutboxPublisherServiceError::Store)?;
                    failed += 1;
                }
            }
        }

        Ok(OutboxPublishBatchResult { published, failed })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxPublishBatchResult {
    pub published: usize,
    pub failed: usize,
}

#[derive(Debug, Error)]
pub enum KnowledgeOutboxPublisherServiceError {
    #[error(transparent)]
    Store(#[from] KnowledgeOutboxStoreError),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::ports::knowledge_outbox_dispatcher::KnowledgeOutboxDispatchError;
    use crate::ports::knowledge_outbox_store::{AppendOutboxEventRecord, PendingOutboxEvent};

    struct InMemoryOutboxStore {
        pending: tokio::sync::Mutex<Vec<PendingOutboxEvent>>,
        published: tokio::sync::Mutex<Vec<u64>>,
        failed: tokio::sync::Mutex<Vec<(u64, String)>>,
    }

    impl InMemoryOutboxStore {
        fn new() -> Self {
            Self {
                pending: tokio::sync::Mutex::new(Vec::new()),
                published: tokio::sync::Mutex::new(Vec::new()),
                failed: tokio::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl KnowledgeOutboxStore for InMemoryOutboxStore {
        async fn append_event(
            &self,
            record: AppendOutboxEventRecord,
        ) -> Result<(), KnowledgeOutboxStoreError> {
            let mut pending = self.pending.lock().await;
            let id = pending.len() as u64 + 1;
            pending.push(PendingOutboxEvent {
                id,
                event_uuid: format!("event-{id}"),
                event_type: record.event_type,
                aggregate_type: record.aggregate_type,
                aggregate_id: record.aggregate_id,
                retry_count: 0,
                payload_json: record.payload_json,
            });
            Ok(())
        }

        async fn list_pending_events(
            &self,
            limit: u32,
        ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError> {
            let pending = self.pending.lock().await;
            Ok(pending.iter().take(limit as usize).cloned().collect())
        }

        async fn claim_pending_events(
            &self,
            limit: u32,
        ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError> {
            let mut pending = self.pending.lock().await;
            let claimed: Vec<PendingOutboxEvent> =
                pending.iter().take(limit as usize).cloned().collect();
            pending.drain(0..claimed.len());
            Ok(claimed)
        }

        async fn release_stale_claimed_events(
            &self,
            _stale_after_secs: u64,
        ) -> Result<usize, KnowledgeOutboxStoreError> {
            Ok(0)
        }

        async fn mark_published(&self, event_id: u64) -> Result<(), KnowledgeOutboxStoreError> {
            let mut pending = self.pending.lock().await;
            pending.retain(|event| event.id != event_id);
            self.published.lock().await.push(event_id);
            Ok(())
        }

        async fn mark_failed(
            &self,
            event_id: u64,
            error_message: &str,
        ) -> Result<(), KnowledgeOutboxStoreError> {
            let mut pending = self.pending.lock().await;
            pending.retain(|event| event.id != event_id);
            self.failed
                .lock()
                .await
                .push((event_id, error_message.to_string()));
            Ok(())
        }

        async fn requeue_failed_events(
            &self,
            _limit: u32,
            _max_retry_count: u32,
        ) -> Result<usize, KnowledgeOutboxStoreError> {
            Ok(0)
        }
    }

    struct AlwaysFailDispatcher;

    #[async_trait]
    impl KnowledgeOutboxDispatcher for AlwaysFailDispatcher {
        async fn dispatch(
            &self,
            _tenant_id: u64,
            _event: &PendingOutboxEvent,
        ) -> Result<(), KnowledgeOutboxDispatchError> {
            Err(KnowledgeOutboxDispatchError::DeliveryFailed(
                "simulated failure".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn publish_pending_marks_failed_without_publishing_on_dispatch_error() {
        let store = Arc::new(InMemoryOutboxStore::new());
        store
            .append_event(AppendOutboxEventRecord {
                aggregate_type: "ingestion_job".to_string(),
                aggregate_id: 1,
                event_type: "knowledge.ingest.succeeded".to_string(),
                payload_json: r#"{"spaceId":1}"#.to_string(),
            })
            .await
            .expect("append");

        let result = KnowledgeOutboxPublisherService::new(1, store.as_ref(), &AlwaysFailDispatcher)
            .publish_pending(10)
            .await
            .expect("publish batch");

        assert_eq!(result.published, 0);
        assert_eq!(result.failed, 1);
        assert!(store.published.lock().await.is_empty());
        assert_eq!(store.failed.lock().await.len(), 1);
    }
}
