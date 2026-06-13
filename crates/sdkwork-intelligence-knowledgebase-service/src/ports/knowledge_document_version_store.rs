use async_trait::async_trait;
use sdkwork_knowledgebase_contract::document::KnowledgeDocumentVersion;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeDocumentVersionStore: Send + Sync {
    async fn create_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError>;

    async fn create_or_get_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        self.create_document_version(record).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeDocumentVersionRecord {
    pub document_id: u64,
    pub version_no: u64,
    pub original_object_ref_id: u64,
    pub checksum_sha256_hex: Option<String>,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeDocumentVersionStoreError {
    #[error("knowledge document version store internal error: {0}")]
    Internal(String),
}
