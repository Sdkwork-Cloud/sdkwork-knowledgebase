//! Routes vector/hybrid retrieval to PostgreSQL pgvector when available.

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchHit, KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
    KnowledgeRetrievalBackendError,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;
use std::sync::Arc;

use crate::postgres_pgvector_retrieval::PgVectorKnowledgeRetrievalBackend;
use crate::retrieval_store::{merge_hybrid_hits, SqliteKnowledgeChunkRetrievalStore};

#[derive(Debug, Clone)]
pub struct PgVectorLayeredRetrievalBackend {
    keyword: Arc<SqliteKnowledgeChunkRetrievalStore>,
    pgvector: Arc<PgVectorKnowledgeRetrievalBackend>,
}

impl PgVectorLayeredRetrievalBackend {
    pub fn new(
        keyword: Arc<SqliteKnowledgeChunkRetrievalStore>,
        pgvector: Arc<PgVectorKnowledgeRetrievalBackend>,
    ) -> Self {
        Self { keyword, pgvector }
    }
}

#[async_trait]
impl KnowledgeRetrievalBackend for PgVectorLayeredRetrievalBackend {
    async fn search_chunks(
        &self,
        request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        match request.method {
            KnowledgeRetrievalMethod::Vector if request.query_embedding.is_some() => {
                self.pgvector.search_chunks(request).await
            }
            KnowledgeRetrievalMethod::Hybrid if request.query_embedding.is_some() => {
                let embedding_dimension = request
                    .query_embedding
                    .as_ref()
                    .map(|values| values.len())
                    .unwrap_or(0);
                let mut keyword_request = request.clone();
                keyword_request.method = KnowledgeRetrievalMethod::Exact;
                let keyword_backend = self.keyword.clone();
                let vector_backend = self.pgvector.clone();
                let (keyword_result, vector_result) = tokio::join!(
                    keyword_backend.search_chunks(keyword_request),
                    vector_backend.search_chunks(request),
                );
                Ok(merge_hybrid_hits(
                    keyword_result?,
                    vector_result?,
                    embedding_dimension,
                ))
            }
            _ => self.keyword.search_chunks(request).await,
        }
    }
}
