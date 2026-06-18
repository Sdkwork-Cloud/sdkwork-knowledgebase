use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeIndexRequest;
use thiserror::Error;

use crate::ports::knowledge_embedding_store::{
    ChunkEmbeddingIndexRequest, KnowledgeEmbeddingStore, KnowledgeEmbeddingStoreError,
};

pub struct KnowledgeEmbeddingIndexService<'a> {
    embeddings: &'a dyn KnowledgeEmbeddingStore,
    embedder: ClawRouterEmbeddingClient,
}

impl<'a> KnowledgeEmbeddingIndexService<'a> {
    pub fn new(
        embeddings: &'a dyn KnowledgeEmbeddingStore,
        embedder: ClawRouterEmbeddingClient,
    ) -> Self {
        Self {
            embeddings,
            embedder,
        }
    }

    pub async fn index_chunk(
        &self,
        request: ChunkEmbeddingIndexRequest,
    ) -> Result<(), KnowledgeEmbeddingIndexServiceError> {
        if request.chunk_id == 0 || request.index_id == 0 {
            return Err(KnowledgeEmbeddingIndexServiceError::InvalidRequest(
                "chunk_id and index_id are required".to_string(),
            ));
        }

        let content = match request.content {
            Some(content) if !content.trim().is_empty() => content,
            _ => self
                .embeddings
                .load_chunk_content(request.chunk_id)
                .await
                .map_err(KnowledgeEmbeddingIndexServiceError::Store)?
                .ok_or_else(|| {
                    KnowledgeEmbeddingIndexServiceError::InvalidRequest(format!(
                        "chunk content was not found for chunk_id {}",
                        request.chunk_id
                    ))
                })?,
        };

        let model_id = request
            .embedding_model
            .as_deref()
            .or(request.index_embedding_model.as_deref());
        let vector = self
            .embedder
            .embed_text(&content, model_id)
            .map_err(KnowledgeEmbeddingIndexServiceError::Embedding)?;

        self.embeddings
            .upsert_chunk_embedding(
                crate::ports::knowledge_embedding_store::ChunkEmbeddingUpsertRequest {
                    tenant_id: request.tenant_id,
                    index_id: request.index_id,
                    chunk_id: request.chunk_id,
                    vector,
                    provider_id: request.embedding_provider_id,
                    model: request.embedding_model.or(request.index_embedding_model),
                },
            )
            .await
            .map_err(KnowledgeEmbeddingIndexServiceError::Store)?;

        Ok(())
    }

    pub fn index_request_from_knowledge_index(
        tenant_id: u64,
        index: &sdkwork_knowledgebase_contract::rag::KnowledgeIndex,
        chunk_id: u64,
        embedding_provider_id: Option<String>,
        embedding_model: Option<String>,
    ) -> ChunkEmbeddingIndexRequest {
        ChunkEmbeddingIndexRequest {
            tenant_id,
            index_id: index.index_id,
            chunk_id,
            content: None,
            embedding_provider_id,
            embedding_model,
            index_embedding_model: None,
        }
    }

    pub fn index_request_from_create_index(
        request: &KnowledgeIndexRequest,
        index_id: u64,
        chunk_id: u64,
        content: Option<String>,
    ) -> ChunkEmbeddingIndexRequest {
        ChunkEmbeddingIndexRequest {
            tenant_id: request.tenant_id,
            index_id,
            chunk_id,
            content,
            embedding_provider_id: request.embedding_provider_id.clone(),
            embedding_model: request.embedding_model.clone(),
            index_embedding_model: request.embedding_model.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeEmbeddingIndexServiceError {
    #[error("invalid embedding index request: {0}")]
    InvalidRequest(String),
    #[error("embedding provider error: {0}")]
    Embedding(String),
    #[error(transparent)]
    Store(#[from] KnowledgeEmbeddingStoreError),
}
