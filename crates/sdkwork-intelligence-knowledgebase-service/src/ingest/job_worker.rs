use crate::imports::{
    DriveImportPipelineResult, KnowledgeDriveImportPipelineService,
    KnowledgeDriveImportPipelineServiceError,
};
use crate::ports::knowledge_chunk_store::KnowledgeChunkStore;
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::ports::knowledge_ingestion_job_store::IngestionJobStore;
use sdkwork_knowledgebase_contract::ingest::IngestionJobState;
use sdkwork_knowledgebase_contract::source::KnowledgeSourceType;
use thiserror::Error;

pub struct KnowledgeIngestionJobWorkerService<'a> {
    jobs: &'a dyn IngestionJobStore,
    drive: &'a dyn KnowledgeDriveStorage,
    chunks: &'a dyn KnowledgeChunkStore,
}

impl<'a> KnowledgeIngestionJobWorkerService<'a> {
    pub fn new(
        jobs: &'a dyn IngestionJobStore,
        drive: &'a dyn KnowledgeDriveStorage,
        chunks: &'a dyn KnowledgeChunkStore,
    ) -> Self {
        Self {
            jobs,
            drive,
            chunks,
        }
    }

    pub async fn process_queued_jobs(
        &self,
        limit: u32,
    ) -> Result<IngestionJobWorkerBatchResult, KnowledgeIngestionJobWorkerServiceError> {
        let jobs = self
            .jobs
            .list_jobs_by_state(IngestionJobState::Queued, limit)
            .await
            .map_err(KnowledgeIngestionJobWorkerServiceError::Store)?;

        let mut skipped = 0usize;

        for job in jobs {
            if job.source_type != KnowledgeSourceType::DriveObject.as_str() {
                skipped += 1;
                continue;
            }
            skipped += 1;
            let _ = job;
        }

        Ok(IngestionJobWorkerBatchResult {
            processed: 0,
            skipped,
            failed: 0,
        })
    }

    pub async fn process_drive_import_result(
        &self,
        import: &sdkwork_knowledgebase_contract::ingest::KnowledgeDriveImportResult,
    ) -> Result<DriveImportPipelineResult, KnowledgeDriveImportPipelineServiceError> {
        KnowledgeDriveImportPipelineService::new(self.drive, self.jobs, self.chunks)
            .process_import_result(import)
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestionJobWorkerBatchResult {
    pub processed: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Debug, Error)]
pub enum KnowledgeIngestionJobWorkerServiceError {
    #[error(transparent)]
    Store(#[from] crate::ports::knowledge_ingestion_job_store::IngestionJobStoreError),
}
