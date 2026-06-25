use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    document::{KnowledgeDocument, KnowledgeDocumentVersion},
    drive::KnowledgeDriveObjectRef,
    ingest::IngestionJob,
    source::KnowledgeSource,
};
use thiserror::Error;

use super::{
    knowledge_document_store::CreateKnowledgeDocumentRecord,
    knowledge_document_version_store::CreateKnowledgeDocumentVersionRecord,
    knowledge_drive_object_ref_store::CreateKnowledgeDriveObjectRefRecord,
    knowledge_ingestion_job_store::CreateIngestionJobRecord,
    knowledge_source_store::CreateKnowledgeSourceRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareDriveImportMetadataRecord {
    pub job: CreateIngestionJobRecord,
    pub object_ref: CreateKnowledgeDriveObjectRefRecord,
    pub source: CreateKnowledgeSourceRecord,
    pub document: CreateKnowledgeDocumentRecord,
    pub version: CreateKnowledgeDocumentVersionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedDriveImportMetadata {
    pub job: IngestionJob,
    pub source: KnowledgeSource,
    pub document: KnowledgeDocument,
    pub version: KnowledgeDocumentVersion,
    pub original_object_ref: KnowledgeDriveObjectRef,
}

#[async_trait]
pub trait DriveImportMetadataStore: Send + Sync {
    async fn validate_drive_import_idempotency(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<(), DriveImportMetadataStoreError>;

    async fn create_or_prepare_drive_import_metadata(
        &self,
        record: PrepareDriveImportMetadataRecord,
    ) -> Result<PreparedDriveImportMetadata, DriveImportMetadataStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DriveImportMetadataStoreError {
    #[error("invalid drive import metadata request: {0}")]
    InvalidRequest(String),
    #[error("drive import metadata conflict: {0}")]
    Conflict(String),
    #[error("drive import metadata store internal error: {0}")]
    Internal(String),
}
