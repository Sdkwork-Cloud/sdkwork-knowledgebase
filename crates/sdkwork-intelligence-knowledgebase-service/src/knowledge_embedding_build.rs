use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use thiserror::Error;

use crate::knowledge_embedding_index::{
    KnowledgeEmbeddingIndexService, KnowledgeEmbeddingIndexServiceError,
};
use crate::ports::knowledge_embedding_store::{
    KnowledgeEmbeddingStore, KnowledgeEmbeddingStoreError,
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

        const CHUNK_PAGE_SIZE: u32 = 128;
        let indexer = KnowledgeEmbeddingIndexService::new(self.embeddings, self.embedder.clone());
        let mut indexed = 0usize;
        let mut after_chunk_id = 0u64;

        loop {
            let batch = self
                .embeddings
                .list_active_chunk_id_content_page(space_id, after_chunk_id, CHUNK_PAGE_SIZE)
                .await
                .map_err(KnowledgeEmbeddingBuildServiceError::Store)?;
            if batch.is_empty() {
                break;
            }

            after_chunk_id = batch
                .last()
                .map(|(chunk_id, _)| *chunk_id)
                .unwrap_or(after_chunk_id);

            indexed += indexer
                .index_chunks(
                    tenant_id,
                    index_id,
                    &batch,
                    embedding_provider_id.clone(),
                    embedding_model.clone(),
                )
                .await
                .map_err(KnowledgeEmbeddingBuildServiceError::Index)?;

            if batch.len() < CHUNK_PAGE_SIZE as usize {
                break;
            }
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
