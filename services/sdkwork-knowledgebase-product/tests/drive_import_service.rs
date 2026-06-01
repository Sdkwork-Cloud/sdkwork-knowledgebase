use async_trait::async_trait;
use sdkwork_knowledgebase_contract::document::{
    KnowledgeDocument, KnowledgeDocumentState, KnowledgeDocumentVersion,
    KnowledgeDocumentVersionState, KnowledgeDocumentVisibility,
};
use sdkwork_knowledgebase_contract::ingest::{
    IngestionJob, IngestionJobState, KnowledgeDriveImportRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_product::imports::KnowledgeDriveImportService;
use sdkwork_knowledgebase_product::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentStore, KnowledgeDocumentStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_document_version_store::{
    CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
    KnowledgeDocumentVersionStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, CreateOrGetIngestionJobResult, IngestionJobStore,
    IngestionJobStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn drive_import_heads_drive_object_then_creates_source_document_version_and_job() {
    let drive = RecordingDrive::with_object(
        "knowledgebase-source",
        "incoming/quarterly-report.md",
        "# Report",
    );
    let sources = MemorySourceStore::default();
    let documents = MemoryDocumentStore::default();
    let object_refs = MemoryDriveObjectRefStore::default();
    let versions = MemoryDocumentVersionStore::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeDriveImportService::new(
        &drive,
        &sources,
        &documents,
        &object_refs,
        &versions,
        &jobs,
    );

    let result = service
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_bucket: "knowledgebase-source".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report".to_string(),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(
        drive.heads(),
        vec![(
            "knowledgebase-source".to_string(),
            "incoming/quarterly-report.md".to_string()
        )]
    );
    assert_eq!(result.source.source_type, KnowledgeSourceType::DriveObject);
    assert_eq!(
        result.source.drive_bucket.as_deref(),
        Some("knowledgebase-source")
    );
    assert_eq!(
        result.source.drive_prefix.as_deref(),
        Some("incoming/quarterly-report.md")
    );
    assert_eq!(result.document.source_id, Some(result.source.id));
    assert_eq!(result.document.title, "Quarterly Report");
    assert_eq!(result.document.content_state, KnowledgeDocumentState::Ready);
    assert_eq!(result.version.version_no, 1);
    assert_eq!(result.original_object_ref.object_role, "original_document");
    assert_eq!(
        result.original_object_ref.drive_provider_kind,
        "sdkwork-drive"
    );
    assert_eq!(
        result.original_object_ref.drive_bucket,
        "knowledgebase-source"
    );
    assert_eq!(
        result.original_object_ref.drive_object_key,
        "incoming/quarterly-report.md"
    );
    assert_eq!(
        result.original_object_ref.drive_object_version.as_deref(),
        Some("v1")
    );
    assert_eq!(
        result.original_object_ref.drive_etag.as_deref(),
        Some("etag")
    );
    assert_eq!(
        result.original_object_ref.size_bytes,
        "# Report".len() as u64
    );
    assert_eq!(
        result.version.original_object_ref_id,
        result.original_object_ref.id
    );
    assert_eq!(
        object_refs.created_refs(),
        vec![result.original_object_ref.clone()]
    );
    assert_eq!(result.job.source_type, "drive_object");
}

#[tokio::test]
async fn drive_import_replay_reuses_metadata_for_same_idempotency_key() {
    let drive = RecordingDrive::with_object(
        "knowledgebase-source",
        "incoming/quarterly-report.md",
        "# Report",
    );
    let sources = MemorySourceStore::default();
    let documents = MemoryDocumentStore::default();
    let object_refs = MemoryDriveObjectRefStore::default();
    let versions = MemoryDocumentVersionStore::default();
    let jobs = MemoryIngestionJobStore::default();
    let service = KnowledgeDriveImportService::new(
        &drive,
        &sources,
        &documents,
        &object_refs,
        &versions,
        &jobs,
    );

    let request = KnowledgeDriveImportRequest {
        space_id: 7,
        title: "Quarterly Report".to_string(),
        drive_bucket: "knowledgebase-source".to_string(),
        drive_object_key: "incoming/quarterly-report.md".to_string(),
        idempotency_key: "drive-quarterly-report".to_string(),
        language: Some("en".to_string()),
    };

    let first = service.import_drive_object(request.clone()).await.unwrap();
    let replay = service.import_drive_object(request).await.unwrap();

    assert_eq!(first.job.id, replay.job.id);
    assert_eq!(first.source.id, replay.source.id);
    assert_eq!(first.document.id, replay.document.id);
    assert_eq!(first.version.id, replay.version.id);
    assert_eq!(first.original_object_ref.id, replay.original_object_ref.id);
    assert_eq!(sources.create_count(), 1);
    assert_eq!(documents.create_count(), 1);
    assert_eq!(versions.create_count(), 1);
    assert_eq!(object_refs.create_count(), 1);
}

#[derive(Clone)]
struct StoredObject {
    object_ref: KnowledgeObjectRef,
}

#[derive(Default)]
struct RecordingDrive {
    objects: Mutex<HashMap<String, StoredObject>>,
    heads: Arc<Mutex<Vec<(String, String)>>>,
}

impl RecordingDrive {
    fn with_object(bucket: &str, object_key: &str, body: &str) -> Self {
        let drive = Self::default();
        drive.objects.lock().unwrap().insert(
            object_key.to_string(),
            StoredObject {
                object_ref: KnowledgeObjectRef {
                    bucket: bucket.to_string(),
                    object_key: object_key.to_string(),
                    logical_path: object_key.to_string(),
                    object_role: "original_document".to_string(),
                    content_type: "text/markdown; charset=utf-8".to_string(),
                    size_bytes: body.len() as u64,
                    checksum_sha256_hex: None,
                    etag: Some("etag".to_string()),
                    version_id: Some("v1".to_string()),
                },
            },
        );
        drive
    }

    fn heads(&self) -> Vec<(String, String)> {
        self.heads.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveStorage for RecordingDrive {
    async fn put_object(
        &self,
        _request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal("not needed"))
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        self.heads
            .lock()
            .unwrap()
            .push((request.bucket.clone(), request.object_key.clone()));
        self.objects
            .lock()
            .unwrap()
            .get(&request.object_key)
            .map(|stored| stored.object_ref.clone())
            .ok_or_else(|| KnowledgeStorageError::NotFound(request.object_key))
    }

    async fn get_object_text(
        &self,
        _object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        Err(KnowledgeStorageError::internal("not needed"))
    }
}

#[derive(Default)]
struct MemorySourceStore {
    next_id: Mutex<u64>,
    by_locator: Mutex<HashMap<(u64, String, String), KnowledgeSource>>,
}

impl MemorySourceStore {
    fn create_count(&self) -> usize {
        self.by_locator.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeSourceStore for MemorySourceStore {
    async fn create_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        self.insert_source(record)
    }

    async fn create_or_get_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        let key = source_key(&record);
        if let Some(source) = self.by_locator.lock().unwrap().get(&key).cloned() {
            return Ok(source);
        }
        self.insert_source(record)
    }
}

impl MemorySourceStore {
    fn insert_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let source = KnowledgeSource {
            id: *next_id,
            space_id: record.space_id,
            source_type: record.source_type,
            provider: record.provider,
            drive_bucket: record.drive_bucket,
            drive_prefix: record.drive_prefix,
        };
        if let (Some(bucket), Some(prefix)) = (&source.drive_bucket, &source.drive_prefix) {
            self.by_locator.lock().unwrap().insert(
                (source.space_id, bucket.clone(), prefix.clone()),
                source.clone(),
            );
        }
        Ok(source)
    }
}

fn source_key(record: &CreateKnowledgeSourceRecord) -> (u64, String, String) {
    (
        record.space_id,
        record.drive_bucket.clone().unwrap_or_default(),
        record.drive_prefix.clone().unwrap_or_default(),
    )
}

#[derive(Default)]
struct MemoryDocumentStore {
    next_id: Mutex<u64>,
    by_source: Mutex<HashMap<(u64, u64), KnowledgeDocument>>,
}

impl MemoryDocumentStore {
    fn create_count(&self) -> usize {
        self.by_source.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDocumentStore for MemoryDocumentStore {
    async fn create_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        self.insert_document(record)
    }

    async fn create_or_get_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        let key = document_key(&record);
        if let Some(document) = self.by_source.lock().unwrap().get(&key).cloned() {
            return Ok(document);
        }
        self.insert_document(record)
    }
}

impl MemoryDocumentStore {
    fn insert_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let document = KnowledgeDocument {
            id: *next_id,
            space_id: record.space_id,
            collection_id: record.collection_id,
            source_id: record.source_id,
            title: record.title,
            mime_type: record.mime_type,
            language: record.language,
            current_version_id: None,
            visibility: KnowledgeDocumentVisibility::Space,
            content_state: KnowledgeDocumentState::Ready,
            index_state: KnowledgeDocumentVersionState::Pending,
        };
        if let Some(source_id) = document.source_id {
            self.by_source
                .lock()
                .unwrap()
                .insert((document.space_id, source_id), document.clone());
        }
        Ok(document)
    }
}

fn document_key(record: &CreateKnowledgeDocumentRecord) -> (u64, u64) {
    (record.space_id, record.source_id.unwrap_or_default())
}

#[derive(Default)]
struct MemoryDriveObjectRefStore {
    next_id: Mutex<u64>,
    created_refs: Mutex<Vec<KnowledgeDriveObjectRef>>,
}

impl MemoryDriveObjectRefStore {
    fn created_refs(&self) -> Vec<KnowledgeDriveObjectRef> {
        self.created_refs.lock().unwrap().clone()
    }

    fn create_count(&self) -> usize {
        self.created_refs.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDriveObjectRefStore for MemoryDriveObjectRefStore {
    async fn create_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        self.insert_object_ref(record)
    }

    async fn create_or_get_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        let refs = self.created_refs.lock().unwrap();
        if let Some(object_ref) = refs
            .iter()
            .find(|object_ref| {
                object_ref.space_id == record.space_id
                    && object_ref.drive_bucket == record.drive_bucket
                    && object_ref.drive_object_key == record.drive_object_key
                    && object_ref.drive_object_version == record.drive_object_version
                    && object_ref.object_role == record.object_role
            })
            .cloned()
        {
            return Ok(object_ref);
        }
        drop(refs);
        self.insert_object_ref(record)
    }
}

impl MemoryDriveObjectRefStore {
    fn insert_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let object_ref = KnowledgeDriveObjectRef {
            id: *next_id,
            space_id: record.space_id,
            drive_provider_kind: record.drive_provider_kind,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            drive_object_version: record.drive_object_version,
            drive_etag: record.drive_etag,
            content_type: record.content_type,
            size_bytes: record.size_bytes,
            checksum_sha256_hex: record.checksum_sha256_hex,
            object_role: record.object_role,
            access_mode: record.access_mode,
        };
        self.created_refs.lock().unwrap().push(object_ref.clone());
        Ok(object_ref)
    }
}

#[derive(Default)]
struct MemoryDocumentVersionStore {
    next_id: Mutex<u64>,
    by_document: Mutex<HashMap<(u64, u64), KnowledgeDocumentVersion>>,
}

impl MemoryDocumentVersionStore {
    fn create_count(&self) -> usize {
        self.by_document.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeDocumentVersionStore for MemoryDocumentVersionStore {
    async fn create_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        self.insert_document_version(record)
    }

    async fn create_or_get_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        let key = (record.document_id, record.version_no);
        if let Some(version) = self.by_document.lock().unwrap().get(&key).cloned() {
            return Ok(version);
        }
        self.insert_document_version(record)
    }
}

impl MemoryDocumentVersionStore {
    fn insert_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let version = KnowledgeDocumentVersion {
            id: *next_id,
            document_id: record.document_id,
            version_no: record.version_no,
            original_object_ref_id: record.original_object_ref_id,
            checksum_sha256_hex: record.checksum_sha256_hex,
            size_bytes: record.size_bytes,
            mime_type: record.mime_type,
            parse_state: KnowledgeDocumentVersionState::Pending,
            index_state: KnowledgeDocumentVersionState::Pending,
        };
        self.by_document
            .lock()
            .unwrap()
            .insert((version.document_id, version.version_no), version.clone());
        Ok(version)
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
