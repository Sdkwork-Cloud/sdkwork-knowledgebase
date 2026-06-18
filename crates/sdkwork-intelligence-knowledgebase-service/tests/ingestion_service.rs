use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionService;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, CreateOrGetIngestionJobResult, IngestionJobStore,
    IngestionJobStoreError,
};
use sdkwork_knowledgebase_contract::ingest::{
    CreateIngestionJobRequest, IngestionJob, IngestionJobState,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[tokio::test]
async fn creating_ingest_job_is_idempotent_by_key() {
    let store = MemoryIngestionJobStore::default();
    let service = KnowledgeIngestionService::new(&store);

    let first = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: "upload-1".to_string(),
        })
        .await
        .unwrap();
    let second = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: "upload-1".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(first.state, IngestionJobState::Queued);
}

#[tokio::test]
async fn creating_ingest_job_trims_idempotency_key_before_lookup() {
    let store = MemoryIngestionJobStore::default();
    let service = KnowledgeIngestionService::new(&store);

    let first = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: "upload-1".to_string(),
        })
        .await
        .unwrap();
    let replay = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: " upload-1 ".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(first.id, replay.id);
    assert_eq!(replay.idempotency_key, "upload-1");
}

#[tokio::test]
async fn ingest_job_supports_valid_state_transitions() {
    let store = MemoryIngestionJobStore::default();
    let service = KnowledgeIngestionService::new(&store);
    let job = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: "upload-2".to_string(),
        })
        .await
        .unwrap();

    let running = service.mark_running(job.id).await.unwrap();
    assert_eq!(running.state, IngestionJobState::Running);

    let succeeded = service.mark_succeeded(job.id).await.unwrap();
    assert_eq!(succeeded.state, IngestionJobState::Succeeded);
}

#[tokio::test]
async fn ingest_job_rejects_invalid_state_transition() {
    let store = MemoryIngestionJobStore::default();
    let service = KnowledgeIngestionService::new(&store);
    let job = service
        .create_job(CreateIngestionJobRequest {
            space_id: 1,
            source_type: "upload".to_string(),
            idempotency_key: "upload-3".to_string(),
        })
        .await
        .unwrap();

    assert!(service.mark_succeeded(job.id).await.is_err());
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
            .ok_or_else(|| IngestionJobStoreError::NotFound(job_id))
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

    async fn list_jobs_by_state(
        &self,
        state: IngestionJobState,
        limit: u32,
    ) -> Result<Vec<IngestionJob>, IngestionJobStoreError> {
        let jobs = self.by_id.lock().unwrap();
        Ok(jobs
            .values()
            .filter(|job| job.state == state)
            .take(limit as usize)
            .cloned()
            .collect())
    }
}
