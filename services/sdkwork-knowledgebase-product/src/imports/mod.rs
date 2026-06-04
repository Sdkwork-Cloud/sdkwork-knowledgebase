use crate::ingest::{KnowledgeIngestionService, KnowledgeIngestionServiceError};
use crate::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentStore, KnowledgeDocumentStoreError,
};
use crate::ports::knowledge_document_version_store::{
    CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
    KnowledgeDocumentVersionStoreError,
};
use crate::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError, MANAGED_DRIVE_ACCESS_MODE, SDKWORK_DRIVE_PROVIDER_KIND,
};
use crate::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeStorageError,
};
use crate::ports::knowledge_ingestion_job_store::IngestionJobStore;
use crate::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::document::{KnowledgeDocument, KnowledgeDocumentVersion};
use sdkwork_knowledgebase_contract::ingest::{
    CreateIngestionJobRequest, IngestionJob, KnowledgeDriveImportRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use thiserror::Error;

pub struct KnowledgeDriveImportService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    sources: &'a dyn KnowledgeSourceStore,
    documents: &'a dyn KnowledgeDocumentStore,
    object_refs: &'a dyn KnowledgeDriveObjectRefStore,
    versions: &'a dyn KnowledgeDocumentVersionStore,
    jobs: &'a dyn IngestionJobStore,
}

impl<'a> KnowledgeDriveImportService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        sources: &'a dyn KnowledgeSourceStore,
        documents: &'a dyn KnowledgeDocumentStore,
        object_refs: &'a dyn KnowledgeDriveObjectRefStore,
        versions: &'a dyn KnowledgeDocumentVersionStore,
        jobs: &'a dyn IngestionJobStore,
    ) -> Self {
        Self {
            drive,
            sources,
            documents,
            object_refs,
            versions,
            jobs,
        }
    }

    pub async fn import_drive_object(
        &self,
        request: KnowledgeDriveImportRequest,
    ) -> Result<KnowledgeDriveImportResult, KnowledgeDriveImportServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if request.title.trim().is_empty() {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "title is required".to_string(),
            ));
        }
        if request.drive_bucket.trim().is_empty() {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "drive_bucket is required".to_string(),
            ));
        }
        if request.drive_object_key.trim().is_empty() {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "drive_object_key is required".to_string(),
            ));
        }
        if request.idempotency_key.trim().is_empty() {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "idempotency_key is required".to_string(),
            ));
        }
        if !is_safe_idempotency_key(&request.idempotency_key) {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "idempotency_key contains unsafe characters".to_string(),
            ));
        }
        let drive_space_id = normalize_optional_drive_id(request.drive_space_id, "drive_space_id")?;
        let drive_node_id = normalize_optional_drive_id(request.drive_node_id, "drive_node_id")?;
        if drive_node_id.is_some() && drive_space_id.is_none() {
            return Err(KnowledgeDriveImportServiceError::InvalidRequest(
                "drive_space_id is required when drive_node_id is provided".to_string(),
            ));
        }

        let original_object_ref = self
            .drive
            .head_object(HeadKnowledgeObjectRequest::original_document(
                request.drive_bucket.clone(),
                request.drive_object_key.clone(),
            ))
            .await?;

        let original_drive_object_ref = self
            .object_refs
            .create_or_get_object_ref(CreateKnowledgeDriveObjectRefRecord {
                space_id: request.space_id,
                drive_space_id: drive_space_id.clone(),
                drive_node_id: drive_node_id.clone(),
                logical_path: Some(original_object_ref.logical_path.clone()),
                drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
                drive_bucket: original_object_ref.bucket.clone(),
                drive_object_key: original_object_ref.object_key.clone(),
                drive_object_version: original_object_ref.version_id.clone(),
                drive_etag: original_object_ref.etag.clone(),
                content_type: Some(original_object_ref.content_type.clone()),
                size_bytes: original_object_ref.size_bytes,
                checksum_sha256_hex: original_object_ref.checksum_sha256_hex.clone(),
                object_role: original_object_ref.object_role.clone(),
                access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
            })
            .await?;

        let source = self
            .sources
            .create_or_get_source(CreateKnowledgeSourceRecord {
                space_id: request.space_id,
                source_type: KnowledgeSourceType::DriveObject,
                provider: Some("sdkwork-drive".to_string()),
                drive_bucket: Some(request.drive_bucket),
                drive_prefix: Some(request.drive_object_key),
            })
            .await?;

        let mut document = self
            .documents
            .create_or_get_document(CreateKnowledgeDocumentRecord {
                space_id: request.space_id,
                collection_id: 0,
                source_id: Some(source.id),
                original_file_drive_node_id: drive_node_id,
                title: request.title,
                mime_type: Some(original_object_ref.content_type.clone()),
                language: request.language,
            })
            .await?;

        let version = self
            .versions
            .create_or_get_document_version(CreateKnowledgeDocumentVersionRecord {
                document_id: document.id,
                version_no: 1,
                original_object_ref_id: original_drive_object_ref.id,
                checksum_sha256_hex: original_drive_object_ref.checksum_sha256_hex.clone(),
                size_bytes: original_drive_object_ref.size_bytes,
                mime_type: original_drive_object_ref.content_type.clone(),
            })
            .await?;
        document.current_version_id = Some(version.id);

        let job = KnowledgeIngestionService::new(self.jobs)
            .create_job(CreateIngestionJobRequest {
                space_id: request.space_id,
                source_type: KnowledgeSourceType::DriveObject.as_str().to_string(),
                idempotency_key: request.idempotency_key,
            })
            .await?;

        Ok(KnowledgeDriveImportResult {
            source,
            document,
            version,
            original_object_ref: original_drive_object_ref,
            job,
        })
    }
}

fn is_safe_idempotency_key(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
}

fn normalize_optional_drive_id(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<String>, KnowledgeDriveImportServiceError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim().to_string();
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Err(KnowledgeDriveImportServiceError::InvalidRequest(format!(
            "{field_name} contains unsafe characters"
        )));
    }
    Ok(Some(value))
}

#[derive(Debug, Clone)]
pub struct KnowledgeDriveImportResult {
    pub source: KnowledgeSource,
    pub document: KnowledgeDocument,
    pub version: KnowledgeDocumentVersion,
    pub original_object_ref: KnowledgeDriveObjectRef,
    pub job: IngestionJob,
}

#[derive(Debug, Error)]
pub enum KnowledgeDriveImportServiceError {
    #[error("invalid drive import request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    SourceStore(#[from] KnowledgeSourceStoreError),
    #[error(transparent)]
    DocumentStore(#[from] KnowledgeDocumentStoreError),
    #[error(transparent)]
    DriveObjectRefStore(#[from] KnowledgeDriveObjectRefStoreError),
    #[error(transparent)]
    VersionStore(#[from] KnowledgeDocumentVersionStoreError),
    #[error(transparent)]
    Ingestion(#[from] KnowledgeIngestionServiceError),
}
