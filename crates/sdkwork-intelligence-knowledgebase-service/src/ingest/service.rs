use crate::ports::{
    knowledge_drive_storage::{
        HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef,
        KnowledgeStorageError, PutKnowledgeObjectRequest,
    },
    knowledge_ingestion_job_store::{
        CreateIngestionJobRecord, IngestionJobStore, IngestionJobStoreError,
    },
};
use sdkwork_knowledgebase_contract::ingest::{
    CreateIngestionJobRequest, IngestionJob, IngestionJobState, KnowledgeIngestRequest,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

pub struct KnowledgeIngestionService<'a> {
    store: &'a dyn IngestionJobStore,
}

impl<'a> KnowledgeIngestionService<'a> {
    pub fn new(store: &'a dyn IngestionJobStore) -> Self {
        Self { store }
    }

    pub async fn create_job(
        &self,
        request: CreateIngestionJobRequest,
    ) -> Result<IngestionJob, KnowledgeIngestionServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeIngestionServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        let idempotency_key = normalize_idempotency_key(&request.idempotency_key)
            .map_err(KnowledgeIngestionServiceError::InvalidRequest)?;

        self.store
            .create_or_get_job(CreateIngestionJobRecord {
                space_id: request.space_id,
                source_type: request.source_type,
                idempotency_key,
                idempotency_fingerprint_sha256_hex: None,
            })
            .await
            .map(|result| result.job)
            .map_err(KnowledgeIngestionServiceError::Store)
    }

    pub async fn mark_running(
        &self,
        job_id: u64,
    ) -> Result<IngestionJob, KnowledgeIngestionServiceError> {
        self.transition(job_id, IngestionJobState::Running, None)
            .await
    }

    pub async fn mark_succeeded(
        &self,
        job_id: u64,
    ) -> Result<IngestionJob, KnowledgeIngestionServiceError> {
        self.transition(job_id, IngestionJobState::Succeeded, None)
            .await
    }

    pub async fn mark_failed(
        &self,
        job_id: u64,
        error_message: impl Into<String>,
    ) -> Result<IngestionJob, KnowledgeIngestionServiceError> {
        self.transition(
            job_id,
            IngestionJobState::Failed,
            Some(error_message.into()),
        )
        .await
    }

    async fn transition(
        &self,
        job_id: u64,
        next_state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, KnowledgeIngestionServiceError> {
        let current = self.store.get_job(job_id).await?;
        if !is_valid_transition(current.state, next_state) {
            return Err(KnowledgeIngestionServiceError::InvalidTransition {
                from: current.state,
                to: next_state,
            });
        }

        self.store
            .update_job_state(job_id, next_state, error_message)
            .await
            .map_err(KnowledgeIngestionServiceError::Store)
    }
}

fn is_valid_transition(from: IngestionJobState, to: IngestionJobState) -> bool {
    matches!(
        (from, to),
        (IngestionJobState::Queued, IngestionJobState::Running)
            | (IngestionJobState::Queued, IngestionJobState::Failed)
            | (IngestionJobState::Running, IngestionJobState::Succeeded)
            | (IngestionJobState::Running, IngestionJobState::Failed)
    )
}

fn is_safe_idempotency_key(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
}

fn normalize_idempotency_key(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("idempotency_key is required".to_string());
    }
    if !is_safe_idempotency_key(value) {
        return Err("idempotency_key contains unsafe characters".to_string());
    }
    Ok(value.to_string())
}

#[derive(Debug, Error)]
pub enum KnowledgeIngestionServiceError {
    #[error("invalid ingestion request: {0}")]
    InvalidRequest(String),
    #[error("invalid ingestion job transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: IngestionJobState,
        to: IngestionJobState,
    },
    #[error(transparent)]
    Store(#[from] IngestionJobStoreError),
}

pub struct KnowledgeApiPayloadIngestService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    jobs: &'a dyn IngestionJobStore,
}

impl<'a> KnowledgeApiPayloadIngestService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage, jobs: &'a dyn IngestionJobStore) -> Self {
        Self { drive, jobs }
    }

    pub async fn ingest_markdown_payload(
        &self,
        request: KnowledgeIngestRequest,
    ) -> Result<ApiPayloadIngestResult, KnowledgeApiPayloadIngestServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeApiPayloadIngestServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if is_blank(Some(request.title.as_str())) {
            return Err(KnowledgeApiPayloadIngestServiceError::InvalidRequest(
                "title is required".to_string(),
            ));
        }
        if is_blank(Some(request.payload_markdown.as_str())) {
            return Err(KnowledgeApiPayloadIngestServiceError::InvalidRequest(
                "payload_markdown is required".to_string(),
            ));
        }
        let idempotency_key = normalize_idempotency_key(&request.idempotency_key)
            .map_err(KnowledgeApiPayloadIngestServiceError::InvalidRequest)?;

        let payload_markdown = request.payload_markdown;
        let job_result = self
            .jobs
            .create_or_get_job(CreateIngestionJobRecord {
                space_id: request.space_id,
                source_type: "api".to_string(),
                idempotency_key,
                idempotency_fingerprint_sha256_hex: None,
            })
            .await
            .map_err(KnowledgeApiPayloadIngestServiceError::Store)?;
        let job = job_result.job;
        let payload_path = format!("inbox/api/{}/payload.md", job.id);

        if !job_result.created {
            match self
                .drive
                .head_object(HeadKnowledgeObjectRequest::managed_artifact(
                    payload_path.clone(),
                    "api_payload",
                ))
                .await
            {
                Ok(existing_payload) => {
                    return Ok(ApiPayloadIngestResult {
                        payload_object_ref: existing_payload,
                        job,
                    });
                }
                Err(KnowledgeStorageError::NotFound(_)) => {}
                Err(error) => return Err(KnowledgeApiPayloadIngestServiceError::Storage(error)),
            }
        }

        let payload_object_ref = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                payload_path,
                "api_payload",
                payload_markdown,
                None,
            ))
            .await?;

        Ok(ApiPayloadIngestResult {
            payload_object_ref,
            job,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiPayloadIngestResult {
    pub payload_object_ref: KnowledgeObjectRef,
    pub job: IngestionJob,
}

#[derive(Debug, Error)]
pub enum KnowledgeApiPayloadIngestServiceError {
    #[error("invalid api payload ingest request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    Store(#[from] IngestionJobStoreError),
}
