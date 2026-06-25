use async_trait::async_trait;
use sdkwork_knowledgebase_contract::document::{KnowledgeDocument, KnowledgeDocumentVersion};
use sdkwork_knowledgebase_contract::drive::KnowledgeDriveObjectRef;
use thiserror::Error;

use super::{
    knowledge_document_store::{CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope},
    knowledge_document_version_store::CreateKnowledgeDocumentVersionRecord,
    knowledge_drive_object_ref_store::CreateKnowledgeDriveObjectRefRecord,
    knowledge_source_store::CreateKnowledgeSourceRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownIndexSourceBinding {
    Existing { source_id: u64 },
    Create(CreateKnowledgeSourceRecord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareMarkdownIndexMetadataRecord {
    pub source: MarkdownIndexSourceBinding,
    pub object_ref: CreateKnowledgeDriveObjectRefRecord,
    pub document: CreateKnowledgeDocumentRecord,
    pub version: CreateKnowledgeDocumentVersionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMarkdownIndexMetadata {
    pub source_id: u64,
    pub object_ref: KnowledgeDriveObjectRef,
    pub document: KnowledgeDocument,
    pub version: KnowledgeDocumentVersion,
}

#[async_trait]
pub trait MarkdownIndexMetadataStore: Send + Sync {
    async fn create_or_prepare_markdown_index_metadata(
        &self,
        record: PrepareMarkdownIndexMetadataRecord,
    ) -> Result<PreparedMarkdownIndexMetadata, MarkdownIndexMetadataStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MarkdownIndexMetadataStoreError {
    #[error("invalid markdown index metadata request: {0}")]
    InvalidRequest(String),
    #[error("markdown index metadata conflict: {0}")]
    Conflict(String),
    #[error("markdown index metadata store internal error: {0}")]
    Internal(String),
}

impl MarkdownIndexMetadataStoreError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub fn validate_markdown_index_document_identity(
    record: &CreateKnowledgeDocumentRecord,
    source_binding: &MarkdownIndexSourceBinding,
) -> Result<(), MarkdownIndexMetadataStoreError> {
    if record.identity_scope == KnowledgeDocumentIdentityScope::SourceOnly {
        match source_binding {
            MarkdownIndexSourceBinding::Existing { source_id } if *source_id == 0 => {
                return Err(MarkdownIndexMetadataStoreError::invalid_request(
                    "source_id is required for source_only document identity",
                ));
            }
            MarkdownIndexSourceBinding::Create(source) if source.space_id != record.space_id => {
                return Err(MarkdownIndexMetadataStoreError::invalid_request(
                    "source.space_id must match document.space_id",
                ));
            }
            MarkdownIndexSourceBinding::Existing { source_id: _ }
            | MarkdownIndexSourceBinding::Create(_) => {}
        }
        if record.source_id.is_some() {
            return Err(MarkdownIndexMetadataStoreError::invalid_request(
                "document.source_id must be omitted; source binding resolves source_id in metadata transaction",
            ));
        }
    }
    Ok(())
}
