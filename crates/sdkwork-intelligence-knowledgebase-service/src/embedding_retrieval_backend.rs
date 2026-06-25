use async_trait::async_trait;
use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;
use std::sync::Arc;

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
                tokio::task::spawn_blocking(move || embedder.embed_text(&query, None))
                    .await
                    .map_err(|error| {
                        KnowledgeRetrievalBackendError::Internal(format!(
                            "embedding worker join failed: {error}"
                        ))
                    })?
                    .map_err(KnowledgeRetrievalBackendError::Internal)?,
            );
        }

        self.inner.search_chunks(request).await
    }
}

fn should_embed_query(method: KnowledgeRetrievalMethod) -> bool {
    matches!(
        method,
        KnowledgeRetrievalMethod::Vector | KnowledgeRetrievalMethod::Hybrid
    )
}
