use async_trait::async_trait;
use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;

use crate::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchHit, KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
    KnowledgeRetrievalBackendError,
};

#[derive(Clone)]
pub struct ClawRouterEmbeddingRetrievalBackend<B> {
    inner: B,
    embedder: ClawRouterEmbeddingClient,
}

impl<B> ClawRouterEmbeddingRetrievalBackend<B> {
    pub fn new(inner: B, embedder: ClawRouterEmbeddingClient) -> Self {
        Self { inner, embedder }
    }
}

#[async_trait]
impl<B> KnowledgeRetrievalBackend for ClawRouterEmbeddingRetrievalBackend<B>
where
    B: KnowledgeRetrievalBackend + Send + Sync,
{
    async fn search_chunks(
        &self,
        mut request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        if should_embed_query(request.method) && request.query_embedding.is_none() {
            request.query_embedding = self
                .embedder
                .embed_text(&request.query, None)
                .map_err(KnowledgeRetrievalBackendError::Internal)
                .ok();
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
