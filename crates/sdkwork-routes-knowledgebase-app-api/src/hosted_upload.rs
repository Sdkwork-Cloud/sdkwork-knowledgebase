use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ingest::{
    ApiMarkdownIngestPipeline, ExistingMarkdownIngestJobParams, KnowledgeUploadSessionService,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_access_control::KnowledgeAccessRole;
use sdkwork_knowledgebase_contract::ingest::IngestionJob;
use sdkwork_knowledgebase_contract::upload::{
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
    KnowledgeUploadSession,
};
use sdkwork_utils_rust::is_blank;

use crate::{
    hosted_access::require_space_access_with_role, runtime::KnowledgebaseRuntime, ApiError,
    ApiResult, KnowledgeAppRequestContext, KnowledgeUploadSessionAppService,
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
        KnowledgeUploadSessionService::new(
            self.runtime.drive_storage(),
            self.runtime.ingestion_job_store(),
        )
        .load_session(session_id)
        .await
        .map_err(ApiError::from)
    }

    async fn run_ingest_pipeline(
        &self,
        space_id: u64,
        drive_space_id: Option<&str>,
        title: &str,
        payload_markdown: &str,
        job_id: u64,
    ) -> ApiResult<IngestionJob> {
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
                drive_space_id,
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
        require_space_access_with_role(
            &self.runtime,
            &context,
            request.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
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
        let space = require_space_access_with_role(
            &self.runtime,
            &context,
            request.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
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
            space.drive_space_id.as_deref(),
            &request.title,
            &payload_markdown,
            session_id,
        )
        .await
    }
}
