use crate::ingest::{
    ingest_success_outbox_record, KnowledgeApiMarkdownIndexService,
    KnowledgeApiMarkdownIndexServiceError, KnowledgeApiPayloadIngestService,
    KnowledgeApiPayloadIngestServiceError, KnowledgeIngestionService,
    KnowledgeIngestionServiceError,
};
use crate::ports::{
    knowledge_drive_storage::{
        KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
    },
    knowledge_ingestion_job_store::{
        CompleteRunningIngestionRecord, IngestionJobStore, IngestionJobStoreError,
    },
    knowledge_source_store::CreateKnowledgeSourceRecord,
    markdown_index_metadata_store::{MarkdownIndexMetadataStore, MarkdownIndexSourceBinding},
};
use sdkwork_knowledgebase_contract::{
    ingest::{IngestionJob, IngestionJobState},
    source::KnowledgeSourceType,
    KnowledgeIngestRequest,
};
use std::future::Future;
use thiserror::Error;

pub struct ApiMarkdownIngestPipeline<'a> {
    drive: &'a dyn crate::ports::knowledge_drive_storage::KnowledgeDriveStorage,
    jobs: &'a dyn IngestionJobStore,
    markdown_metadata: &'a dyn MarkdownIndexMetadataStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingMarkdownIngestJobParams {
    pub space_id: u64,
    pub job_id: u64,
    pub title: String,
    pub payload_markdown: String,
    pub ingest_provider: String,
    pub source_drive_prefix: String,
    pub payload_logical_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiMarkdownIngestPipelineResult {
    pub job: IngestionJob,
    pub document_version_id: Option<u64>,
}

#[derive(Debug, Error)]
pub enum ApiMarkdownIngestPipelineError {
    #[error(transparent)]
    Payload(#[from] KnowledgeApiPayloadIngestServiceError),
    #[error(transparent)]
    Ingestion(#[from] KnowledgeIngestionServiceError),
    #[error(transparent)]
    Index(#[from] KnowledgeApiMarkdownIndexServiceError),
    #[error(transparent)]
    Store(#[from] IngestionJobStoreError),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
}

impl<'a> ApiMarkdownIngestPipeline<'a> {
    pub fn new(
        drive: &'a dyn crate::ports::knowledge_drive_storage::KnowledgeDriveStorage,
        jobs: &'a dyn IngestionJobStore,
        markdown_metadata: &'a dyn MarkdownIndexMetadataStore,
    ) -> Self {
        Self {
            drive,
            jobs,
            markdown_metadata,
        }
    }

    pub async fn run(
        &self,
        request: KnowledgeIngestRequest,
        drive_space_id: Option<&str>,
        ingest_provider: &str,
    ) -> Result<ApiMarkdownIngestPipelineResult, ApiMarkdownIngestPipelineError> {
        let space_id = request.space_id;
        let title = request.title.clone();

        let payload_service = KnowledgeApiPayloadIngestService::new(self.drive, self.jobs);
        let result = payload_service
            .ingest_markdown_payload(request, drive_space_id)
            .await?;
        let payload_markdown = result.resolved_payload_markdown.clone();
        let mut job = result.job;
        if let Some(replay) = self.replay_if_not_processable(job.clone()).await? {
            return Ok(replay);
        }

        let ingestion = KnowledgeIngestionService::new(self.jobs);
        job = ingestion.mark_running(job.id).await?;
        let source_drive_prefix = format!("inbox/api/{}", job.id);

        let job_id = job.id;
        self.with_running_job_failure_guard(
            job_id,
            self.index_and_complete_running_job(
                job,
                space_id,
                &title,
                &payload_markdown,
                &result.payload_object_ref,
                ingest_provider,
                &source_drive_prefix,
                drive_space_id,
            ),
        )
        .await
    }

    pub async fn run_existing_queued_job(
        &self,
        params: ExistingMarkdownIngestJobParams,
        drive_space_id: Option<&str>,
    ) -> Result<ApiMarkdownIngestPipelineResult, ApiMarkdownIngestPipelineError> {
        let ingestion = KnowledgeIngestionService::new(self.jobs);
        let job = ingestion.mark_running(params.job_id).await?;
        let job_id = job.id;

        self.with_running_job_failure_guard(job_id, async move {
            let payload_object_ref = self
                .drive
                .put_object(
                    PutKnowledgeObjectRequest::text(
                        params.payload_logical_path,
                        "api_payload",
                        params.payload_markdown.as_str(),
                        None,
                    )
                    .with_drive_space_id(drive_space_id),
                )
                .await?;

            self.index_and_complete_running_job(
                job,
                params.space_id,
                &params.title,
                &params.payload_markdown,
                &payload_object_ref,
                &params.ingest_provider,
                &params.source_drive_prefix,
                drive_space_id,
            )
            .await
        })
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn index_and_complete_running_job(
        &self,
        mut job: IngestionJob,
        space_id: u64,
        title: &str,
        payload_markdown: &str,
        payload_object_ref: &KnowledgeObjectRef,
        ingest_provider: &str,
        source_drive_prefix: &str,
        drive_space_id: Option<&str>,
    ) -> Result<ApiMarkdownIngestPipelineResult, ApiMarkdownIngestPipelineError> {
        let ingestion = KnowledgeIngestionService::new(self.jobs);

        let indexer = KnowledgeApiMarkdownIndexService::new(self.markdown_metadata);
        let index_result = indexer
            .prepare_payload_markdown_index(
                space_id,
                MarkdownIndexSourceBinding::Create(CreateKnowledgeSourceRecord {
                    space_id,
                    source_type: KnowledgeSourceType::Api,
                    provider: Some(ingest_provider.to_string()),
                    drive_bucket: None,
                    drive_prefix: Some(source_drive_prefix.to_string()),
                    connector_metadata_json: None,
                }),
                title,
                payload_markdown,
                payload_object_ref,
                drive_space_id,
            )
            .await?;

        let document_version_id = index_result.document_version_id;
        if let Some(linkage) = index_result.ingest_linkage {
            self.jobs
                .attach_drive_import_linkage(job.id, linkage)
                .await?;
        }

        let completed = ingestion
            .complete_with_chunks_and_outbox(CompleteRunningIngestionRecord {
                job_id: job.id,
                claim_token: None,
                document_version_id,
                chunks: index_result.chunk_records,
                outbox: ingest_success_outbox_record(&job),
            })
            .await?;
        job = completed.job;

        Ok(ApiMarkdownIngestPipelineResult {
            job,
            document_version_id: Some(document_version_id),
        })
    }

    async fn with_running_job_failure_guard<T, F>(
        &self,
        job_id: u64,
        operation: F,
    ) -> Result<T, ApiMarkdownIngestPipelineError>
    where
        F: Future<Output = Result<T, ApiMarkdownIngestPipelineError>>,
    {
        match operation.await {
            Ok(result) => Ok(result),
            Err(error) => {
                let ingestion = KnowledgeIngestionService::new(self.jobs);
                if let Err(mark_error) = ingestion.mark_failed(job_id, error.to_string()).await {
                    tracing::error!(
                        job_id,
                        ?mark_error,
                        "failed to mark ingestion job as failed after running work error"
                    );
                }
                Err(error)
            }
        }
    }

    async fn replay_if_not_processable(
        &self,
        job: IngestionJob,
    ) -> Result<Option<ApiMarkdownIngestPipelineResult>, ApiMarkdownIngestPipelineError> {
        match job.state {
            IngestionJobState::Queued | IngestionJobState::Failed => Ok(None),
            IngestionJobState::Running
            | IngestionJobState::Succeeded
            | IngestionJobState::Cancelled => {
                let linkage = self.jobs.get_drive_import_linkage(job.id).await?;
                Ok(Some(ApiMarkdownIngestPipelineResult {
                    job,
                    document_version_id: linkage.map(|value| value.document_version_id),
                }))
            }
        }
    }
}
