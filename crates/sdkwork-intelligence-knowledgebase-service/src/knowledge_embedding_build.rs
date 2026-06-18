use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use thiserror::Error;

use crate::knowledge_embedding_index::{
    KnowledgeEmbeddingIndexService, KnowledgeEmbeddingIndexServiceError,
};
use crate::ports::knowledge_embedding_store::{
    ChunkEmbeddingIndexRequest, KnowledgeEmbeddingStore, KnowledgeEmbeddingStoreError,
};

pub struct KnowledgeEmbeddingBuildService<'a> {
    embeddings: &'a dyn KnowledgeEmbeddingStore,
    embedder: ClawRouterEmbeddingClient,
}

impl<'a> KnowledgeEmbeddingBuildService<'a> {
    pub fn new(
        embeddings: &'a dyn KnowledgeEmbeddingStore,
        embedder: ClawRouterEmbeddingClient,
    ) -> Self {
        Self {
            embeddings,
            embedder,
        }
    }

    pub async fn embed_space_chunks(
        &self,
        tenant_id: u64,
        index_id: u64,
        space_id: u64,
        embedding_provider_id: Option<String>,
        embedding_model: Option<String>,
    ) -> Result<usize, KnowledgeEmbeddingBuildServiceError> {
        if tenant_id == 0 || index_id == 0 || space_id == 0 {
            return Err(KnowledgeEmbeddingBuildServiceError::InvalidRequest(
                "tenant_id, index_id, and space_id are required".to_string(),
            ));
        }

        let chunk_ids = self
            .embeddings
            .list_active_chunk_ids_for_space(space_id)
            .await
            .map_err(KnowledgeEmbeddingBuildServiceError::Store)?;

        let indexer = KnowledgeEmbeddingIndexService::new(self.embeddings, self.embedder.clone());
        let mut indexed = 0usize;
        for chunk_id in chunk_ids {
            indexer
                .index_chunk(ChunkEmbeddingIndexRequest {
                    tenant_id,
                    index_id,
                    chunk_id,
                    content: None,
                    embedding_provider_id: embedding_provider_id.clone(),
                    embedding_model: embedding_model.clone(),
                    index_embedding_model: embedding_model.clone(),
                })
                .await
                .map_err(KnowledgeEmbeddingBuildServiceError::Index)?;
            indexed += 1;
        }

        Ok(indexed)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeEmbeddingBuildServiceError {
    #[error("invalid embedding build request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Store(#[from] KnowledgeEmbeddingStoreError),
    #[error(transparent)]
    Index(#[from] KnowledgeEmbeddingIndexServiceError),
}
