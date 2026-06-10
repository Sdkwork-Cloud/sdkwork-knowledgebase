use async_trait::async_trait;
use sdkwork_knowledgebase_contract::ingest::{
    IngestionJob, IngestionJobState, KnowledgeIngestRequest,
};
use sdkwork_knowledgebase_product::ingest::KnowledgeApiPayloadIngestService;
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, CreateOrGetIngestionJobResult, IngestionJobStore,
    IngestionJobStoreError,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[tokio::test]
async fn api_markdown_ingest_writes_payload_through_drive_and_creates_idempotent_job() {
    let drive = RecordingDrive::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeApiPayloadIngestService::new(&drive, &jobs);

    let request = KnowledgeIngestRequest {
        space_id: 7,
        title: "API payload note".to_string(),
        payload_markdown: "# API Note\n\nImportant source text.".to_string(),
        idempotency_key: "api-note-1".to_string(),
    };

    let first = service
        .ingest_markdown_payload(request.clone())
        .await
        .unwrap();
    let second = service.ingest_markdown_payload(request).await.unwrap();

    assert_eq!(first.job.id, second.job.id);
    assert_eq!(first.job.source_type, "api");
    assert_eq!(
        first.payload_object_ref.logical_path,
        "inbox/api/1/payload.md"
    );
    assert_eq!(first.payload_object_ref.object_role, "api_payload");
    assert_eq!(
        first.payload_object_ref.logical_path,
        second.payload_object_ref.logical_path
    );
    assert_eq!(
        drive.body_at("inbox/api/1/payload.md"),
        Some("# API Note\n\nImportant source text.".to_string())
    );
}

#[tokio::test]
async fn api_markdown_ingest_does_not_overwrite_existing_payload_for_same_idempotency_key() {
    let drive = RecordingDrive::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeApiPayloadIngestService::new(&drive, &jobs);

    let first_request = KnowledgeIngestRequest {
        space_id: 7,
        title: "API payload note".to_string(),
        payload_markdown: "# Original\n\nFirst payload.".to_string(),
        idempotency_key: "api-note-1".to_string(),
    };
    let replay_request = KnowledgeIngestRequest {
        payload_markdown: "# Replacement\n\nThis must not overwrite.".to_string(),
        ..first_request.clone()
    };

    let first = service
        .ingest_markdown_payload(first_request)
        .await
        .unwrap();
    let replay = service
        .ingest_markdown_payload(replay_request)
        .await
        .unwrap();

    assert_eq!(first.job.id, replay.job.id);
    assert_eq!(drive.write_count(), 1);
    assert_eq!(
        drive.body_at("inbox/api/1/payload.md"),
        Some("# Original\n\nFirst payload.".to_string())
    );
}

#[tokio::test]
async fn api_markdown_ingest_trims_idempotency_key_before_lookup() {
    let drive = RecordingDrive::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeApiPayloadIngestService::new(&drive, &jobs);

    let request = KnowledgeIngestRequest {
        space_id: 7,
        title: "API payload note".to_string(),
        payload_markdown: "# Original\n\nFirst payload.".to_string(),
        idempotency_key: "api-note-1".to_string(),
    };
    let replay_request = KnowledgeIngestRequest {
        idempotency_key: " api-note-1 ".to_string(),
        payload_markdown: "# Replacement\n\nThis must not overwrite.".to_string(),
        ..request.clone()
    };

    let first = service.ingest_markdown_payload(request).await.unwrap();
    let replay = service
        .ingest_markdown_payload(replay_request)
        .await
        .unwrap();

    assert_eq!(first.job.id, replay.job.id);
    assert_eq!(replay.job.idempotency_key, "api-note-1");
    assert_eq!(drive.write_count(), 1);
    assert_eq!(
        drive.body_at("inbox/api/1/payload.md"),
        Some("# Original\n\nFirst payload.".to_string())
    );
}

#[tokio::test]
async fn api_markdown_ingest_rejects_empty_payload_and_unsafe_idempotency_key() {
    let drive = RecordingDrive::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeApiPayloadIngestService::new(&drive, &jobs);

    let empty_payload = KnowledgeIngestRequest {
        space_id: 7,
        title: "Empty".to_string(),
        payload_markdown: "   ".to_string(),
        idempotency_key: "empty-1".to_string(),
    };
    let unsafe_key = KnowledgeIngestRequest {
        space_id: 7,
        title: "Unsafe".to_string(),
        payload_markdown: "# Unsafe".to_string(),
        idempotency_key: "../escape".to_string(),
    };

    assert!(service
        .ingest_markdown_payload(empty_payload)
        .await
        .is_err());
    assert!(service.ingest_markdown_payload(unsafe_key).await.is_err());
}

#[derive(Default)]
struct RecordingDrive {
    objects: Mutex<HashMap<String, Vec<u8>>>,
    write_count: Mutex<usize>,
}

impl RecordingDrive {
    fn body_at(&self, logical_path: &str) -> Option<String> {
        self.objects
            .lock()
            .unwrap()
            .get(logical_path)
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
    }

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
        self.objects
            .lock()
            .unwrap()
            .insert(request.logical_path.clone(), request.body.clone());
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
        let objects = self.objects.lock().unwrap();
        let body = objects
            .get(&request.object_key)
            .ok_or_else(|| KnowledgeStorageError::NotFound(request.object_key.clone()))?;
        Ok(KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "test".to_string(),
            object_key: request.object_key.clone(),
            logical_path: request
                .logical_path
                .unwrap_or_else(|| request.object_key.clone()),
            object_role: request.object_role,
            content_type: "text/markdown; charset=utf-8".to_string(),
            size_bytes: body.len() as u64,
            checksum_sha256_hex: None,
            etag: None,
            version_id: None,
        })
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.body_at(&object_ref.logical_path)
            .ok_or_else(|| KnowledgeStorageError::NotFound(object_ref.logical_path.clone()))
    }
}

#[derive(Default)]
struct MemoryIngestionJobStore {
    next_id: Mutex<u64>,
    by_id: Mutex<HashMap<u64, IngestionJob>>,
    by_key: Mutex<HashMap<(u64, String), u64>>,
}

#[async_trait]
impl IngestionJobStore for MemoryIngestionJobStore {
    async fn create_or_get_job(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<CreateOrGetIngestionJobResult, IngestionJobStoreError> {
        let key = (record.space_id, record.idempotency_key.clone());
        if let Some(existing_id) = self.by_key.lock().unwrap().get(&key).copied() {
            let job = self
                .by_id
                .lock()
                .unwrap()
                .get(&existing_id)
                .cloned()
                .ok_or_else(|| IngestionJobStoreError::Internal("missing job".to_string()))?;
            return Ok(CreateOrGetIngestionJobResult {
                job,
                created: false,
            });
        }

        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let job = IngestionJob {
            id: *next_id,
            space_id: record.space_id,
            source_type: record.source_type,
            idempotency_key: record.idempotency_key,
            state: IngestionJobState::Queued,
            error_message: None,
        };
        self.by_key.lock().unwrap().insert(key, job.id);
        self.by_id.lock().unwrap().insert(job.id, job.clone());
        Ok(CreateOrGetIngestionJobResult { job, created: true })
    }

    async fn get_job(&self, job_id: u64) -> Result<IngestionJob, IngestionJobStoreError> {
        self.by_id
            .lock()
            .unwrap()
            .get(&job_id)
            .cloned()
            .ok_or(IngestionJobStoreError::NotFound(job_id))
    }

    async fn update_job_state(
        &self,
        job_id: u64,
        state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, IngestionJobStoreError> {
        let mut jobs = self.by_id.lock().unwrap();
        let job = jobs
            .get_mut(&job_id)
            .ok_or(IngestionJobStoreError::NotFound(job_id))?;
        job.state = state;
        job.error_message = error_message;
        Ok(job.clone())
    }
}
