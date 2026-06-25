use sdkwork_knowledgebase_agent_provider::ClawRouterEmbeddingClient;
use sdkwork_knowledgebase_contract::rag::KnowledgeIndexRequest;
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

use crate::ports::knowledge_embedding_store::{
    ChunkEmbeddingIndexRequest, ChunkEmbeddingUpsertRequest, KnowledgeEmbeddingStore,
    KnowledgeEmbeddingStoreError,
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
            Some(content) if !is_blank(Some(content.as_str())) => content,
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

    pub async fn index_chunks(
        &self,
        tenant_id: u64,
        index_id: u64,
        chunks: &[(u64, String)],
        embedding_provider_id: Option<String>,
        embedding_model: Option<String>,
    ) -> Result<usize, KnowledgeEmbeddingIndexServiceError> {
        if index_id == 0 {
            return Err(KnowledgeEmbeddingIndexServiceError::InvalidRequest(
                "index_id is required".to_string(),
            ));
        }

        const EMBED_BATCH_SIZE: usize = 16;
        let mut embedded = 0usize;
        for batch in chunks.chunks(EMBED_BATCH_SIZE) {
            let texts: Vec<String> = batch.iter().map(|(_, text)| text.clone()).collect();
            let vectors = tokio::task::spawn_blocking({
                let embedder = self.embedder.clone();
                let model = embedding_model.clone();
                move || embedder.embed_texts(&texts, model.as_deref())
            })
            .await
            .map_err(|error| {
                KnowledgeEmbeddingIndexServiceError::Internal(format!(
                    "embedding worker join failed: {error}"
                ))
            })?
            .map_err(KnowledgeEmbeddingIndexServiceError::Embedding)?;

            let upsert_requests = batch
                .iter()
                .zip(vectors.iter())
                .filter(|((chunk_id, _), _)| *chunk_id != 0)
                .map(|((chunk_id, _), vector)| ChunkEmbeddingUpsertRequest {
                    tenant_id,
                    index_id,
                    chunk_id: *chunk_id,
                    vector: vector.clone(),
                    provider_id: embedding_provider_id.clone(),
                    model: embedding_model.clone(),
                })
                .collect::<Vec<_>>();

            self.embeddings
                .upsert_chunk_embeddings_batch(&upsert_requests)
                .await
                .map_err(KnowledgeEmbeddingIndexServiceError::Store)?;
            embedded += upsert_requests.len();
        }

        Ok(embedded)
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
    #[error("embedding index internal error: {0}")]
    Internal(String),
    #[error("embedding provider error: {0}")]
    Embedding(String),
    #[error(transparent)]
    Store(#[from] KnowledgeEmbeddingStoreError),
}
