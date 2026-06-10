use async_trait::async_trait;
use sdkwork_knowledgebase_contract::rag::{KnowledgeRetrievalBinding, KnowledgeRetrievalMethod};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeRetrievalBackend: Send + Sync {
    async fn search_chunks(
        &self,
        request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnowledgeChunkSearchRequest {
    pub tenant_id: u64,
    pub query: String,
    pub binding: KnowledgeRetrievalBinding,
    pub method: KnowledgeRetrievalMethod,
    pub top_k: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnowledgeChunkSearchHit {
    pub chunk_id: u64,
    pub document_id: u64,
    pub document_version_id: Option<u64>,
    pub space_id: u64,
    pub collection_id: Option<u64>,
    pub title: String,
    pub content: String,
    pub score: f64,
    pub token_count: Option<u32>,
    pub locator: Option<String>,
    pub source_uri: Option<String>,
    pub retrieval_method: KnowledgeRetrievalMethod,
    pub match_reason: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeRetrievalBackendError {
    #[error("knowledge retrieval backend internal error: {0}")]
    Internal(String),
}
