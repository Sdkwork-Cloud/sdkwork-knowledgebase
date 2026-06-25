use crate::knowledge_embedding_index::{
    KnowledgeEmbeddingIndexService, KnowledgeEmbeddingIndexServiceError,
};
use crate::ports::knowledge_chunk_store::KnowledgeChunkStore;
use crate::ports::knowledge_embedding_store::KnowledgeEmbeddingStore;
use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeIndex;
use thiserror::Error;

pub struct KnowledgePostIngestEmbeddingService<'a> {
    chunks: &'a dyn KnowledgeChunkStore,
    embeddings: &'a dyn KnowledgeEmbeddingStore,
    embedder: ClawRouterEmbeddingClient,
}

impl<'a> KnowledgePostIngestEmbeddingService<'a> {
    pub fn new(
        chunks: &'a dyn KnowledgeChunkStore,
        embeddings: &'a dyn KnowledgeEmbeddingStore,
        embedder: ClawRouterEmbeddingClient,
    ) -> Self {
        Self {
            chunks,
            embeddings,
            embedder,
        }
    }

    pub async fn embed_document_version(
        &self,
        tenant_id: u64,
        index: &KnowledgeIndex,
        document_version_id: u64,
    ) -> Result<usize, KnowledgePostIngestEmbeddingServiceError> {
        if tenant_id == 0 || index.index_id == 0 || document_version_id == 0 {
            return Err(KnowledgePostIngestEmbeddingServiceError::InvalidRequest(
                "tenant_id, index_id, and document_version_id are required".to_string(),
            ));
        }

        let chunk_pairs = self
            .chunks
            .list_chunk_id_content_for_document_version(document_version_id)
            .await
            .map_err(KnowledgePostIngestEmbeddingServiceError::Chunk)?;

        let indexer = KnowledgeEmbeddingIndexService::new(self.embeddings, self.embedder.clone());
        indexer
            .index_chunks(tenant_id, index.index_id, &chunk_pairs, None, None)
            .await
            .map_err(KnowledgePostIngestEmbeddingServiceError::Index)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgePostIngestEmbeddingServiceError {
    #[error("invalid post-ingest embedding request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Chunk(#[from] crate::ports::knowledge_chunk_store::KnowledgeChunkStoreError),
    #[error(transparent)]
    Index(#[from] KnowledgeEmbeddingIndexServiceError),
}
