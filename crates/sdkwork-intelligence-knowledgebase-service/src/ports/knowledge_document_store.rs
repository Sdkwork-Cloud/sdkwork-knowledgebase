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

    async fn get_document_by_id(
        &self,
        document_id: u64,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        let _ = document_id;
        Err(KnowledgeDocumentStoreError::Internal(
            "get_document_by_id is not implemented for this knowledge document store".to_string(),
        ))
    }

    async fn list_documents_for_space(
        &self,
        space_id: u64,
        limit: u32,
    ) -> Result<Vec<KnowledgeDocument>, KnowledgeDocumentStoreError> {
        let _ = (space_id, limit);
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeDocumentRecord {
    pub space_id: u64,
    pub collection_id: u64,
    pub source_id: Option<u64>,
    pub identity_scope: KnowledgeDocumentIdentityScope,
    pub original_file_drive_node_id: Option<String>,
    pub title: String,
    pub mime_type: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowledgeDocumentIdentityScope {
    SourceOnly,
    SourceAndOriginalDriveNode,
}

impl KnowledgeDocumentIdentityScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceOnly => "source_only",
            Self::SourceAndOriginalDriveNode => "source_and_original_drive_node",
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeDocumentStoreError {
    #[error("knowledge document store invalid record: {0}")]
    InvalidRecord(String),
    #[error("knowledge document store internal error: {0}")]
    Internal(String),
}
