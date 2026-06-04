use async_trait::async_trait;
use sdkwork_knowledgebase_contract::document::KnowledgeDocument;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeDocumentStore: Send + Sync {
    async fn create_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError>;

    async fn create_or_get_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        self.create_document(record).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeDocumentRecord {
    pub space_id: u64,
    pub collection_id: u64,
    pub source_id: Option<u64>,
    pub original_file_drive_node_id: Option<String>,
    pub title: String,
    pub mime_type: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeDocumentStoreError {
    #[error("knowledge document store internal error: {0}")]
    Internal(String),
}
