use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ingest::{
    KnowledgeApiMarkdownIndexService, KnowledgeIngestionService, KnowledgeUploadSessionService,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::IngestionJobStore;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::KnowledgeSourceStore;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use sdkwork_knowledgebase_contract::upload::{
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
    KnowledgeUploadSession, KnowledgeUploadSessionStatus,
};

use crate::{
    runtime::KnowledgebaseSqliteRuntime, ApiError, ApiResult, KnowledgeUploadSessionAppService,
};

#[derive(Clone)]
pub(crate) struct SqliteHostedUploadSessionService {
    runtime: KnowledgebaseSqliteRuntime,
}

impl SqliteHostedUploadSessionService {
    pub fn new(runtime: KnowledgebaseSqliteRuntime) -> Self {
        Self { runtime }
    }

    async fn session_from_job_id(&self, session_id: u64) -> ApiResult<KnowledgeUploadSession> {
        let job = self
            .runtime
            .ingestion_job_store()
            .get_job(session_id)
            .await
            .map_err(ApiError::from)?;
        if job.source_type != "upload_session" {
            return Err(ApiError::not_found(
                "upload_session_not_found",
                format!("upload session was not found: {session_id}"),
            ));
        }
        let status = match job.state {
            IngestionJobState::Queued | IngestionJobState::Running => {
                KnowledgeUploadSessionStatus::Pending
            }
            IngestionJobState::Succeeded => KnowledgeUploadSessionStatus::Completed,
            IngestionJobState::Failed | IngestionJobState::Cancelled => {
                KnowledgeUploadSessionStatus::Expired
            }
        };
        Ok(KnowledgeUploadSession {
            id: job.id,
            space_id: job.space_id,
            title: format!("upload-session-{}", job.id),
            upload_logical_path: format!("upload_sessions/{}/payload", job.id),
            status,
            expires_at: String::new(),
        })
    }

    async fn run_ingest_pipeline(
        &self,
        space_id: u64,
        title: &str,
        payload_markdown: &str,
        job_id: u64,
    ) -> ApiResult<IngestionJob> {
        let ingestion = KnowledgeIngestionService::new(self.runtime.ingestion_job_store());
        let mut job = ingestion
            .mark_running(job_id)
            .await
            .map_err(ApiError::from)?;

        let payload_path = format!("inbox/api/{job_id}/payload.md");
        let payload_object_ref = self
            .runtime
            .drive_storage()
            .put_object(PutKnowledgeObjectRequest::text(
                payload_path,
                "api_payload",
                payload_markdown,
                None,
            ))
            .await
            .map_err(ApiError::from)?;

        let indexer = KnowledgeApiMarkdownIndexService::new(
            self.runtime.document_store(),
            self.runtime.version_store(),
            self.runtime.object_ref_store(),
            self.runtime.chunk_store(),
        );
        let source = self
            .runtime
            .source_store()
            .create_or_get_source(
                sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::CreateKnowledgeSourceRecord {
                    space_id,
                    source_type: sdkwork_knowledgebase_contract::source::KnowledgeSourceType::Api,
                    provider: Some("upload-session".to_string()),
                    drive_bucket: None,
                    drive_prefix: Some(format!("upload_sessions/{job_id}")),
                },
            )
            .await
            .map_err(ApiError::from)?;
        let index_result = match indexer
            .index_payload_markdown(
                space_id,
                source.id,
                title,
                payload_markdown,
                &payload_object_ref,
            )
            .await
        {
            Ok(index_result) => index_result,
            Err(error) => {
                let _ = ingestion.mark_failed(job.id, format!("{error:?}")).await;
                return Err(ApiError::from(error));
            }
        };

        let _ = self
            .runtime
            .try_embed_document_version(space_id, index_result.document_version_id)
            .await;

        job = ingestion
            .mark_succeeded(job.id)
            .await
            .map_err(ApiError::from)?;
        self.runtime.try_append_ingest_succeeded_outbox(&job).await;
        Ok(job)
    }
}

#[async_trait]
impl KnowledgeUploadSessionAppService for SqliteHostedUploadSessionService {
    async fn create_upload_session(
        &self,
        request: CreateKnowledgeUploadSessionRequest,
    ) -> ApiResult<KnowledgeUploadSession> {
        let service = KnowledgeUploadSessionService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        );
        service
            .create_session(request)
            .await
            .map_err(ApiError::from)
    }

    async fn complete_upload_session(
        &self,
        session_id: u64,
        request: CompleteKnowledgeUploadSessionRequest,
    ) -> ApiResult<IngestionJob> {
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_upload_session_request",
                "space_id is required",
            ));
        }
        if request.title.trim().is_empty() {
            return Err(ApiError::invalid_request(
                "invalid_upload_session_request",
                "title is required",
            ));
        }

        let session = self.session_from_job_id(session_id).await?;
        if session.space_id != request.space_id {
            return Err(ApiError::invalid_request(
                "space_id_mismatch",
                "spaceId in body must match the upload session space",
            ));
        }

        let upload_service = KnowledgeUploadSessionService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        );
        let payload_markdown = upload_service
            .resolve_payload_markdown(&session, &request)
            .await
            .map_err(ApiError::from)?;

        self.run_ingest_pipeline(
            request.space_id,
            &request.title,
            &payload_markdown,
            session_id,
        )
        .await
    }
}
