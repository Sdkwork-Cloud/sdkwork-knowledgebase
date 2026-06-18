use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeChunkRecord {
    pub space_id: u64,
    pub collection_id: u64,
    pub document_id: u64,
    pub document_version_id: u64,
    pub chunk_index: u32,
    pub content_text: String,
    pub content_hash: String,
    pub token_count: Option<u32>,
    pub locator: Option<String>,
}

#[async_trait]
pub trait KnowledgeChunkStore: Send + Sync {
    async fn replace_version_chunks(
        &self,
        document_version_id: u64,
        chunks: Vec<CreateKnowledgeChunkRecord>,
    ) -> Result<usize, KnowledgeChunkStoreError>;

    async fn list_chunk_ids_for_document_version(
        &self,
        document_version_id: u64,
    ) -> Result<Vec<u64>, KnowledgeChunkStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeChunkStoreError {
    #[error("knowledge chunk store invalid record: {0}")]
    InvalidRecord(String),
    #[error("knowledge chunk store internal error: {0}")]
    Internal(String),
}
