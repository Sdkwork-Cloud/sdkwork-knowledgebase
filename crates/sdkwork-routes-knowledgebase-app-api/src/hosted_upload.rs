use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ingest::{
    ApiMarkdownIngestPipeline, ExistingMarkdownIngestJobParams, KnowledgeUploadSessionService,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::IngestionJobStore;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::KnowledgeSpaceStore;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use sdkwork_knowledgebase_contract::upload::{
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
    KnowledgeUploadSession, KnowledgeUploadSessionStatus,
};
use sdkwork_utils_rust::is_blank;

use crate::{
    hosted_access::require_space_access,
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeUploadSessionAppService,
};

#[derive(Clone)]
pub(crate) struct HostedUploadSessionService {
    runtime: KnowledgebaseRuntime,
}

impl HostedUploadSessionService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
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
        let space = self.runtime.space_store().get_space(space_id).await?;
        let pipeline = ApiMarkdownIngestPipeline::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
            self.runtime.markdown_index_metadata_store(),
        );
        let result = pipeline
            .run_existing_queued_job(
                ExistingMarkdownIngestJobParams {
                    space_id,
                    job_id,
                    title: title.to_string(),
                    payload_markdown: payload_markdown.to_string(),
                    ingest_provider: "upload-session".to_string(),
                    source_drive_prefix: format!("upload_sessions/{job_id}"),
                    payload_logical_path: format!("inbox/api/{job_id}/payload.md"),
                },
                space.drive_space_id.as_deref(),
            )
            .await
            .map_err(ApiError::from)?;

        if let Some(document_version_id) = result.document_version_id {
            let _ = self
                .runtime
                .try_embed_document_version(space_id, document_version_id)
                .await;
        }

        Ok(result.job)
    }
}

#[async_trait]
impl KnowledgeUploadSessionAppService for HostedUploadSessionService {
    async fn create_upload_session(
        &self,
        context: KnowledgeAppRequestContext,
        request: CreateKnowledgeUploadSessionRequest,
    ) -> ApiResult<KnowledgeUploadSession> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        crate::tenant_quota_enforcement::ensure_tenant_can_start_ingest(&self.runtime).await?;
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
        context: KnowledgeAppRequestContext,
        session_id: u64,
        request: CompleteKnowledgeUploadSessionRequest,
    ) -> ApiResult<IngestionJob> {
        require_space_access(&self.runtime, &context, request.space_id).await?;
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_upload_session_request",
                "space_id is required",
            ));
        }
        if is_blank(Some(request.title.as_str())) {
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

        let space = self
            .runtime
            .space_store()
            .get_space(request.space_id)
            .await?;
        let upload_service = KnowledgeUploadSessionService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        );
        let payload_markdown = upload_service
            .resolve_payload_markdown(&session, &request, space.drive_space_id.as_deref())
            .await
            .map_err(ApiError::from)?;

        crate::tenant_quota_enforcement::ensure_tenant_can_add_storage(
            &self.runtime,
            u64::try_from(payload_markdown.len()).unwrap_or(u64::MAX),
        )
        .await?;

        self.run_ingest_pipeline(
            request.space_id,
            &request.title,
            &payload_markdown,
            session_id,
        )
        .await
    }
}
