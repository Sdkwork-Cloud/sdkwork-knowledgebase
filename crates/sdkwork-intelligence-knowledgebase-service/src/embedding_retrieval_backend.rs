use async_trait::async_trait;
use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;
use std::sync::Arc;

use crate::bounded_blocking::{run_bounded_blocking, BoundedBlockingError};
use crate::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchHit, KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
    KnowledgeRetrievalBackendError,
};

pub type SharedKnowledgeRetrievalBackend = Arc<dyn KnowledgeRetrievalBackend + Send + Sync>;

#[derive(Clone)]
pub struct ClawRouterEmbeddingRetrievalBackend {
    inner: SharedKnowledgeRetrievalBackend,
    embedder: ClawRouterEmbeddingClient,
}

impl ClawRouterEmbeddingRetrievalBackend {
    pub fn new(
        inner: SharedKnowledgeRetrievalBackend,
        embedder: ClawRouterEmbeddingClient,
    ) -> Self {
        Self { inner, embedder }
    }
}

#[async_trait]
impl KnowledgeRetrievalBackend for ClawRouterEmbeddingRetrievalBackend {
    async fn search_chunks(
        &self,
        mut request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        if should_embed_query(request.method) && request.query_embedding.is_none() {
            let embedder = self.embedder.clone();
            let query = request.query.clone();
            request.query_embedding = Some(
                run_bounded_blocking(move || embedder.embed_text(&query, None))
                    .await
                    .map_err(map_embedding_blocking_error)?
                    .map_err(KnowledgeRetrievalBackendError::Internal)?,
            );
        }

        self.inner.search_chunks(request).await
    }
}

fn map_embedding_blocking_error(error: BoundedBlockingError) -> KnowledgeRetrievalBackendError {
    match error {
        BoundedBlockingError::QueueSaturated { capacity } => {
            KnowledgeRetrievalBackendError::QueueSaturated { capacity }
        }
        BoundedBlockingError::TimedOut { timeout } => {
            KnowledgeRetrievalBackendError::TimedOut { timeout }
        }
        error @ (BoundedBlockingError::InvalidCapacity
        | BoundedBlockingError::TaskPanicked
        | BoundedBlockingError::TaskCancelled) => KnowledgeRetrievalBackendError::Internal(
            format!("embedding blocking operation failed: {error}"),
        ),
    }
}

fn should_embed_query(method: KnowledgeRetrievalMethod) -> bool {
    matches!(
        method,
        KnowledgeRetrievalMethod::Vector | KnowledgeRetrievalMethod::Hybrid
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounded_blocking::BoundedBlockingError;
    use std::time::Duration;

    #[test]
    fn blocking_error_mapping_preserves_overload_and_timeout() {
        let timeout = Duration::from_secs(11);
        assert!(matches!(
            map_embedding_blocking_error(BoundedBlockingError::QueueSaturated { capacity: 64 }),
            KnowledgeRetrievalBackendError::QueueSaturated { capacity: 64 }
        ));
        assert!(matches!(
            map_embedding_blocking_error(BoundedBlockingError::TimedOut { timeout }),
            KnowledgeRetrievalBackendError::TimedOut { timeout: actual } if actual == timeout
        ));

        for error in [
            BoundedBlockingError::InvalidCapacity,
            BoundedBlockingError::TaskPanicked,
            BoundedBlockingError::TaskCancelled,
        ] {
            assert!(matches!(
                map_embedding_blocking_error(error),
                KnowledgeRetrievalBackendError::Internal(_)
            ));
        }
    }
}
