use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ingest::{
    KnowledgeUploadSessionService, KnowledgeUploadSessionServiceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::{
    CompleteRunningIngestionRecord, CompletedIngestionResult, CreateIngestionJobRecord,
    CreateOrGetIngestionJobResult, DriveImportJobLinkage, IngestionJobLifecycle, IngestionJobStore,
    IngestionJobStoreError, KNOWLEDGE_UPLOAD_SESSION_TTL,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::AppendOutboxEventRecord;
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use sdkwork_knowledgebase_contract::upload::{
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
    KnowledgeUploadSessionStatus,
};
use std::sync::Mutex;
use std::time::Duration;
use time::{format_description::well_known::Rfc3339, Duration as TimeDuration, OffsetDateTime};

#[tokio::test]
async fn upload_session_expiry_is_stable_for_the_same_persisted_job() {
    let drive = RecordingDrive::default();
    let jobs = FixedIngestionJobStore::default();
    let service = KnowledgeUploadSessionService::new(&drive, &jobs);
    let request = CreateKnowledgeUploadSessionRequest {
        space_id: 7,
        title: "Stable expiry".to_string(),
        content_type: Some("text/markdown".to_string()),
    };

    let first = service.create_session(request.clone()).await.unwrap();
    std::thread::sleep(Duration::from_millis(5));
    let reconstructed = service.create_session(request).await.unwrap();
    let expected_expires_at = (jobs.created_at() + KNOWLEDGE_UPLOAD_SESSION_TTL)
        .format(&Rfc3339)
        .unwrap();

    assert_eq!(first.id, reconstructed.id);
    assert_eq!(first.expires_at, reconstructed.expires_at);
    assert_eq!(first.expires_at, expected_expires_at);
}

#[tokio::test]
async fn expired_upload_session_rejects_payload_without_writing_drive() {
    let drive = RecordingDrive::default();
    let jobs = FixedIngestionJobStore::with_created_at(
        OffsetDateTime::now_utc() - TimeDuration::hours(25),
    );
    let service = KnowledgeUploadSessionService::new(&drive, &jobs);
    let session = service.load_session(41).await.unwrap();

    assert_eq!(session.status, KnowledgeUploadSessionStatus::Expired);

    let error = service
        .resolve_payload_markdown(
            &session,
            &CompleteKnowledgeUploadSessionRequest {
                space_id: 7,
                title: "Expired upload".to_string(),
                idempotency_key: "expired-upload-41".to_string(),
                payload_markdown: Some("# Must not be written".to_string()),
            },
            Some("drive-space-7"),
        )
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeUploadSessionServiceError::InvalidRequest(ref detail)
            if detail.contains("expired")
    ));
    assert_eq!(drive.write_count(), 0);
}

#[tokio::test]
async fn non_upload_job_is_reported_as_missing_upload_session() {
    let drive = RecordingDrive::default();
    let jobs = FixedIngestionJobStore::with_source_type("api");
    let service = KnowledgeUploadSessionService::new(&drive, &jobs);

    let error = service.load_session(41).await.unwrap_err();

    assert_eq!(error.to_string(), "upload session not found: 41");
}

#[derive(Default)]
struct RecordingDrive {
    write_count: Mutex<usize>,
}

impl RecordingDrive {
    fn write_count(&self) -> usize {
        *self.write_count.lock().unwrap()
    }
}

#[async_trait]
impl KnowledgeDriveStorage for RecordingDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        *self.write_count.lock().unwrap() += 1;
        Ok(KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "test".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path,
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: request.checksum_sha256_hex,
            etag: None,
            version_id: None,
        })
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::NotFound(request.object_key))
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        Err(KnowledgeStorageError::NotFound(
            object_ref.logical_path.clone(),
        ))
    }
}

struct FixedIngestionJobStore {
    job: Mutex<IngestionJob>,
    created_at: OffsetDateTime,
}

impl Default for FixedIngestionJobStore {
    fn default() -> Self {
        Self::with_created_at(OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap())
    }
}

impl FixedIngestionJobStore {
    fn with_created_at(created_at: OffsetDateTime) -> Self {
        Self {
            job: Mutex::new(IngestionJob {
                id: 41,
                space_id: 7,
                source_type: "upload_session".to_string(),
                idempotency_key: "upload-session-fixed".to_string(),
                state: IngestionJobState::Queued,
                error_message: None,
            }),
            created_at,
        }
    }

    fn with_source_type(source_type: &str) -> Self {
        let mut store = Self::default();
        store.job.get_mut().unwrap().source_type = source_type.to_string();
        store
    }

    fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[async_trait]
impl IngestionJobStore for FixedIngestionJobStore {
    async fn create_or_get_job(
        &self,
        _record: CreateIngestionJobRecord,
    ) -> Result<CreateOrGetIngestionJobResult, IngestionJobStoreError> {
        Ok(CreateOrGetIngestionJobResult {
            job: self.job.lock().unwrap().clone(),
            created: false,
        })
    }

    async fn get_job(&self, job_id: u64) -> Result<IngestionJob, IngestionJobStoreError> {
        let job = self.job.lock().unwrap().clone();
        if job.id == job_id {
            Ok(job)
        } else {
            Err(IngestionJobStoreError::NotFound(job_id))
        }
    }

    async fn get_job_lifecycle(
        &self,
        job_id: u64,
    ) -> Result<IngestionJobLifecycle, IngestionJobStoreError> {
        let job = self.get_job(job_id).await?;
        Ok(IngestionJobLifecycle {
            job,
            created_at: self.created_at,
            updated_at: self.created_at,
        })
    }

    async fn update_job_state(
        &self,
        job_id: u64,
        expected_state: IngestionJobState,
        state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, IngestionJobStoreError> {
        let mut job = self.job.lock().unwrap();
        if job.id != job_id {
            return Err(IngestionJobStoreError::NotFound(job_id));
        }
        if job.state != expected_state {
            return Err(IngestionJobStoreError::Conflict(format!(
                "unexpected state: {:?}",
                job.state
            )));
        }
        job.state = state;
        job.error_message = error_message;
        Ok(job.clone())
    }

    async fn list_jobs_by_state(
        &self,
        state: IngestionJobState,
        limit: u32,
    ) -> Result<Vec<IngestionJob>, IngestionJobStoreError> {
        let job = self.job.lock().unwrap().clone();
        Ok((limit > 0 && job.state == state)
            .then_some(job)
            .into_iter()
            .collect())
    }

    async fn attach_drive_import_linkage(
        &self,
        _job_id: u64,
        _linkage: DriveImportJobLinkage,
    ) -> Result<(), IngestionJobStoreError> {
        Ok(())
    }

    async fn get_drive_import_linkage(
        &self,
        _job_id: u64,
    ) -> Result<Option<DriveImportJobLinkage>, IngestionJobStoreError> {
        Ok(None)
    }

    async fn mark_running_job_succeeded_with_outbox(
        &self,
        job_id: u64,
        _outbox: AppendOutboxEventRecord,
    ) -> Result<IngestionJob, IngestionJobStoreError> {
        self.update_job_state(
            job_id,
            IngestionJobState::Running,
            IngestionJobState::Succeeded,
            None,
        )
        .await
    }

    async fn complete_running_ingestion_with_chunks_and_outbox(
        &self,
        record: CompleteRunningIngestionRecord,
    ) -> Result<CompletedIngestionResult, IngestionJobStoreError> {
        let chunk_count = record.chunks.len();
        let job = self
            .mark_running_job_succeeded_with_outbox(record.job_id, record.outbox)
            .await?;
        Ok(CompletedIngestionResult { job, chunk_count })
    }
}
