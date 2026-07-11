use crate::ingest::payload_limits::{validate_markdown_payload, PayloadLimitError};
use crate::ports::{
    knowledge_drive_storage::{
        HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeStorageError,
        PutKnowledgeObjectRequest,
    },
    knowledge_ingestion_job_store::{
        CreateIngestionJobRecord, IngestionJobLifecycle, IngestionJobStore, IngestionJobStoreError,
        KNOWLEDGE_UPLOAD_SESSION_TTL,
    },
};
use sdkwork_knowledgebase_contract::ingest::IngestionJobState;
use sdkwork_knowledgebase_contract::upload::{
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
    KnowledgeUploadSession, KnowledgeUploadSessionStatus,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub struct KnowledgeUploadSessionService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    jobs: &'a dyn IngestionJobStore,
}

impl<'a> KnowledgeUploadSessionService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage, jobs: &'a dyn IngestionJobStore) -> Self {
        Self { drive, jobs }
    }

    pub async fn create_session(
        &self,
        request: CreateKnowledgeUploadSessionRequest,
    ) -> Result<KnowledgeUploadSession, KnowledgeUploadSessionServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeUploadSessionServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if is_blank(Some(request.title.as_str())) {
            return Err(KnowledgeUploadSessionServiceError::InvalidRequest(
                "title is required".to_string(),
            ));
        }

        let job = self
            .jobs
            .create_or_get_job(CreateIngestionJobRecord {
                space_id: request.space_id,
                source_type: "upload_session".to_string(),
                idempotency_key: format!("upload-session:{}", uuid::Uuid::new_v4()),
                idempotency_fingerprint_sha256_hex: None,
            })
            .await?
            .job;
        let mut session = self.load_session(job.id).await?;
        session.title = request.title;
        Ok(session)
    }

    pub async fn load_session(
        &self,
        session_id: u64,
    ) -> Result<KnowledgeUploadSession, KnowledgeUploadSessionServiceError> {
        let lifecycle = self.jobs.get_job_lifecycle(session_id).await?;
        if lifecycle.job.source_type != "upload_session" {
            return Err(KnowledgeUploadSessionServiceError::NotFound(session_id));
        }
        upload_session_from_lifecycle(lifecycle, OffsetDateTime::now_utc())
    }

    pub async fn resolve_payload_markdown(
        &self,
        session: &KnowledgeUploadSession,
        request: &CompleteKnowledgeUploadSessionRequest,
        drive_space_id: Option<&str>,
    ) -> Result<String, KnowledgeUploadSessionServiceError> {
        match session.status {
            KnowledgeUploadSessionStatus::Pending => {}
            KnowledgeUploadSessionStatus::Completed => {
                return Err(KnowledgeUploadSessionServiceError::InvalidRequest(
                    "upload session is already completed".to_string(),
                ));
            }
            KnowledgeUploadSessionStatus::Expired => {
                return Err(KnowledgeUploadSessionServiceError::InvalidRequest(
                    "upload session has expired".to_string(),
                ));
            }
        }

        if let Some(payload_markdown) = &request.payload_markdown {
            if is_blank(Some(payload_markdown.as_str())) {
                return Err(KnowledgeUploadSessionServiceError::InvalidRequest(
                    "payload_markdown must not be empty when provided".to_string(),
                ));
            }
            validate_markdown_payload(payload_markdown).map_err(|error| match error {
                PayloadLimitError::PayloadEmpty => {
                    KnowledgeUploadSessionServiceError::InvalidRequest(
                        "payload_markdown must not be empty when provided".to_string(),
                    )
                }
                PayloadLimitError::PayloadTooLarge { max_bytes } => {
                    KnowledgeUploadSessionServiceError::InvalidRequest(format!(
                        "payload_markdown exceeds maximum allowed size of {max_bytes} bytes"
                    ))
                }
            })?;
            self.drive
                .put_object(
                    PutKnowledgeObjectRequest::text(
                        session.upload_logical_path.clone(),
                        "upload_payload",
                        payload_markdown,
                        None,
                    )
                    .with_drive_space_id(drive_space_id),
                )
                .await?;
            return Ok(payload_markdown.clone());
        }

        let object_ref = self
            .drive
            .head_object(
                HeadKnowledgeObjectRequest::managed_artifact(
                    session.upload_logical_path.clone(),
                    "upload_payload",
                )
                .with_drive_space_id(drive_space_id),
            )
            .await
            .map_err(|error| match error {
                KnowledgeStorageError::NotFound(_) => {
                    KnowledgeUploadSessionServiceError::InvalidRequest(
                        "upload payload is not available at the session upload path".to_string(),
                    )
                }
                other => KnowledgeUploadSessionServiceError::Storage(other),
            })?;

        self.drive
            .get_object_text(&object_ref)
            .await
            .map_err(KnowledgeUploadSessionServiceError::Storage)
            .and_then(|payload| {
                validate_markdown_payload(&payload).map_err(|error| match error {
                    PayloadLimitError::PayloadEmpty => {
                        KnowledgeUploadSessionServiceError::InvalidRequest(
                            "upload payload must not be empty".to_string(),
                        )
                    }
                    PayloadLimitError::PayloadTooLarge { max_bytes } => {
                        KnowledgeUploadSessionServiceError::InvalidRequest(format!(
                            "upload payload exceeds maximum allowed size of {max_bytes} bytes"
                        ))
                    }
                })?;
                Ok(payload)
            })
    }
}

fn upload_session_from_lifecycle(
    lifecycle: IngestionJobLifecycle,
    now: OffsetDateTime,
) -> Result<KnowledgeUploadSession, KnowledgeUploadSessionServiceError> {
    let expires_at = lifecycle
        .created_at
        .checked_add(KNOWLEDGE_UPLOAD_SESSION_TTL)
        .ok_or_else(|| {
            KnowledgeUploadSessionServiceError::Internal(
                "upload session expiry is outside the supported timestamp range".to_string(),
            )
        })?;
    let status = match lifecycle.job.state {
        IngestionJobState::Queued | IngestionJobState::Running if now >= expires_at => {
            KnowledgeUploadSessionStatus::Expired
        }
        IngestionJobState::Queued | IngestionJobState::Running => {
            KnowledgeUploadSessionStatus::Pending
        }
        IngestionJobState::Succeeded => KnowledgeUploadSessionStatus::Completed,
        IngestionJobState::Failed | IngestionJobState::Cancelled => {
            KnowledgeUploadSessionStatus::Expired
        }
    };
    let expires_at = expires_at
        .format(&Rfc3339)
        .map_err(|error| KnowledgeUploadSessionServiceError::Internal(error.to_string()))?;

    Ok(KnowledgeUploadSession {
        id: lifecycle.job.id,
        space_id: lifecycle.job.space_id,
        title: format!("upload-session-{}", lifecycle.job.id),
        upload_logical_path: format!("upload_sessions/{}/payload", lifecycle.job.id),
        status,
        expires_at,
    })
}

#[derive(Debug, Error)]
pub enum KnowledgeUploadSessionServiceError {
    #[error("upload session not found: {0}")]
    NotFound(u64),
    #[error("invalid upload session request: {0}")]
    InvalidRequest(String),
    #[error("upload session internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Store(#[from] IngestionJobStoreError),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
}
