use crate::ingest::{
    ingest_success_outbox_record, split_markdown_chunks, KnowledgeIngestionService,
    MarkdownIndexResult,
};
use crate::ports::knowledge_drive_storage::{KnowledgeDriveStorage, KnowledgeObjectRef};
use crate::ports::knowledge_ingestion_job_store::{
    CompleteRunningIngestionRecord, IngestionJobStore,
};
use sdkwork_knowledgebase_contract::drive::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, KnowledgeDriveImportResult};
use thiserror::Error;

pub struct KnowledgeDriveImportPipelineService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    jobs: &'a dyn IngestionJobStore,
}

impl<'a> KnowledgeDriveImportPipelineService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage, jobs: &'a dyn IngestionJobStore) -> Self {
        Self { drive, jobs }
    }

    pub async fn process_import_result(
        &self,
        import: &KnowledgeDriveImportResult,
    ) -> Result<DriveImportPipelineResult, KnowledgeDriveImportPipelineServiceError> {
        if import.job.state != sdkwork_knowledgebase_contract::ingest::IngestionJobState::Queued {
            return Ok(DriveImportPipelineResult {
                job: import.job.clone(),
                index_result: None,
            });
        }

        let ingestion = KnowledgeIngestionService::new(self.jobs);
        let job = ingestion
            .mark_running(import.job.id)
            .await
            .map_err(KnowledgeDriveImportPipelineServiceError::Ingestion)?;

        let object_ref = drive_object_ref_to_storage_ref(&import.original_object_ref);
        let payload = match self.drive.get_object_text(&object_ref).await {
            Ok(payload) => payload,
            Err(error) => {
                let _ = ingestion
                    .mark_failed(job.id, format!("drive storage read failed: {error:?}"))
                    .await;
                return Err(KnowledgeDriveImportPipelineServiceError::Storage(error));
            }
        };

        let chunk_records = split_markdown_chunks(
            import.document.space_id,
            import.document.id,
            import.version.id,
            &payload,
        );
        let completed = match ingestion
            .complete_with_chunks_and_outbox(CompleteRunningIngestionRecord {
                job_id: job.id,
                document_version_id: import.version.id,
                chunks: chunk_records,
                outbox: ingest_success_outbox_record(&job),
            })
            .await
        {
            Ok(completed) => completed,
            Err(error) => {
                if let Err(mark_error) = ingestion.mark_failed(job.id, format!("{error:?}")).await {
                    tracing::error!(
                        job_id = job.id,
                        ?mark_error,
                        "failed to mark ingestion job as failed after drive import completion error"
                    );
                }
                return Err(KnowledgeDriveImportPipelineServiceError::Ingestion(error));
            }
        };

        Ok(DriveImportPipelineResult {
            job: completed.job,
            index_result: Some(MarkdownIndexResult {
                document_version_id: import.version.id,
                chunk_count: completed.chunk_count,
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DriveImportPipelineResult {
    pub job: IngestionJob,
    pub index_result: Option<MarkdownIndexResult>,
}

fn drive_object_ref_to_storage_ref(value: &KnowledgeDriveObjectRef) -> KnowledgeObjectRef {
    KnowledgeObjectRef {
        storage_provider_id: value.drive_storage_provider_id.clone(),
        bucket: value.drive_bucket.clone(),
        object_key: value.drive_object_key.clone(),
        logical_path: value
            .logical_path
            .clone()
            .unwrap_or_else(|| value.drive_object_key.clone()),
        object_role: value.object_role.clone(),
        content_type: value
            .content_type
            .clone()
            .unwrap_or_else(|| "text/plain".to_string()),
        size_bytes: value.size_bytes,
        checksum_sha256_hex: value.checksum_sha256_hex.clone(),
        etag: value.drive_etag.clone(),
        version_id: value.drive_object_version.clone(),
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeDriveImportPipelineServiceError {
    #[error(transparent)]
    Ingestion(#[from] crate::ingest::KnowledgeIngestionServiceError),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
}
