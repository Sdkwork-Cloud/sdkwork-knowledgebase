use sdkwork_knowledgebase_contract::document::KnowledgeDocumentVersionState;
use sdkwork_knowledgebase_contract::ingest::{IngestionJobState, KnowledgeDriveImportRequest};
use sdkwork_knowledgebase_product::imports::KnowledgeDriveImportService;
use sdkwork_knowledgebase_product::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, IngestionJobStore,
};
use sdkwork_knowledgebase_storage_sqlx::migrations::SQLITE_CORE_MIGRATION;
use sdkwork_knowledgebase_storage_sqlx::{
    SqliteIngestionJobStore, SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeSourceStore,
};
use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

#[tokio::test]
async fn sqlite_repositories_persist_drive_import_metadata_chain() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    drive
        .put_text(
            "incoming/quarterly-report.md",
            "original_document",
            "# Report",
        )
        .await
        .unwrap();

    let sources = SqliteKnowledgeSourceStore::new(pool.clone(), tenant_id);
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let versions = SqliteKnowledgeDocumentVersionStore::new(pool.clone(), tenant_id);
    let jobs = SqliteIngestionJobStore::new(pool.clone(), tenant_id);
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
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report".to_string(),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();

    assert_ne!(result.source.id, 0);
    assert_ne!(result.document.id, 0);
    assert_ne!(result.version.id, 0);
    assert_ne!(result.job.id, 0);
    assert_eq!(
        result.version.original_object_ref_id,
        result.original_object_ref.id
    );
    assert_eq!(
        result.version.parse_state,
        KnowledgeDocumentVersionState::Pending
    );
    assert_eq!(result.job.state, IngestionJobState::Queued);

    let version_row = sqlx::query(
        r#"
        SELECT tenant_id, document_id, original_object_ref_id, parse_state, index_state
        FROM knowledge_document_version
        WHERE id = ?
        "#,
    )
    .bind(result.version.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(version_row.get::<i64, _>("tenant_id"), tenant_id as i64);
    assert_eq!(
        version_row.get::<i64, _>("document_id"),
        result.document.id as i64
    );
    assert_eq!(
        version_row.get::<i64, _>("original_object_ref_id"),
        result.original_object_ref.id as i64
    );
    assert_eq!(version_row.get::<i64, _>("parse_state"), 0);
    assert_eq!(version_row.get::<i64, _>("index_state"), 0);

    let job_state: i64 =
        sqlx::query_scalar("SELECT state FROM knowledge_ingestion_job WHERE id = ?")
            .bind(result.job.id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(job_state, 0);

    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_source")
        .fetch_one(&pool)
        .await
        .unwrap();
    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_ref_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_drive_object_ref")
            .fetch_one(&pool)
            .await
            .unwrap();
    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_ingestion_job")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(source_count, 1);
    assert_eq!(document_count, 1);
    assert_eq!(version_count, 1);
    assert_eq!(object_ref_count, 1);
    assert_eq!(job_count, 1);
}

#[tokio::test]
async fn sqlite_drive_import_replay_reuses_metadata_chain() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    drive
        .put_text(
            "incoming/quarterly-report.md",
            "original_document",
            "# Report",
        )
        .await
        .unwrap();

    let sources = SqliteKnowledgeSourceStore::new(pool.clone(), tenant_id);
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let versions = SqliteKnowledgeDocumentVersionStore::new(pool.clone(), tenant_id);
    let jobs = SqliteIngestionJobStore::new(pool.clone(), tenant_id);
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
        drive_bucket: "knowledgebase-test".to_string(),
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

    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_source")
        .fetch_one(&pool)
        .await
        .unwrap();
    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_ref_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_drive_object_ref")
            .fetch_one(&pool)
            .await
            .unwrap();
    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_ingestion_job")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(source_count, 1);
    assert_eq!(document_count, 1);
    assert_eq!(version_count, 1);
    assert_eq!(object_ref_count, 1);
    assert_eq!(job_count, 1);
}

#[tokio::test]
async fn sqlite_ingestion_jobs_are_idempotent_per_space_not_whole_tenant() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteIngestionJobStore::new(pool.clone(), 9001);

    let first = store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 7,
            source_type: "api".to_string(),
            idempotency_key: "shared-key".to_string(),
        })
        .await
        .unwrap();
    let other_space = store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 8,
            source_type: "api".to_string(),
            idempotency_key: "shared-key".to_string(),
        })
        .await
        .unwrap();
    let retry_first = store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 7,
            source_type: "api".to_string(),
            idempotency_key: "shared-key".to_string(),
        })
        .await
        .unwrap();

    assert!(first.created);
    assert!(other_space.created);
    assert!(!retry_first.created);
    assert_ne!(first.job.id, other_space.job.id);
    assert_eq!(first.job.id, retry_first.job.id);

    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_ingestion_job")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 2);
}

async fn sqlite_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap()
}

async fn apply_sqlite_migration(pool: &SqlitePool) {
    for statement in SQLITE_CORE_MIGRATION.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await.unwrap();
        }
    }
}
