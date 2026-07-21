use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteDriveImportMetadataStore, SqliteIngestionJobStore, SqliteKnowledgeDocumentStore,
    SqliteKnowledgeDocumentVersionStore, SqliteKnowledgeDriveObjectRefStore,
    SqliteKnowledgeSourceStore, SqliteMarkdownIndexMetadataStore,
};
use sdkwork_intelligence_knowledgebase_service::imports::{
    KnowledgeDriveImportService, ResolvedKnowledgeDriveImportRequest,
};
use sdkwork_intelligence_knowledgebase_service::ingest::{
    ingest_success_outbox_record, split_markdown_chunks, KnowledgeApiMarkdownIndexService,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::{
    CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::{
    ClaimIngestionJobsRequest, CompleteRunningIngestionRecord, CreateIngestionJobRecord,
    IngestionJobStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::MarkdownIndexSourceBinding;
use sdkwork_knowledgebase_contract::document::KnowledgeDocumentVersionState;
use sdkwork_knowledgebase_contract::ingest::{IngestionJobState, KnowledgeDriveImportRequest};
use sdkwork_knowledgebase_contract::source::KnowledgeSourceType;
use sdkwork_knowledgebase_observability::KnowledgebaseTenantQuotaLimits;
use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use time::Duration;
use tokio::sync::Barrier;

fn resolved_drive_import_request(
    title: &str,
    drive_node_id: &str,
    drive_object_key: &str,
    idempotency_key: &str,
) -> ResolvedKnowledgeDriveImportRequest {
    ResolvedKnowledgeDriveImportRequest {
        request: KnowledgeDriveImportRequest {
            space_id: 7,
            title: title.to_string(),
            drive_space_id: "drv-kb-001".to_string(),
            drive_node_id: drive_node_id.to_string(),
            idempotency_key: idempotency_key.to_string(),
            language: Some("en".to_string()),
        },
        drive_storage_provider_id: "provider-kb".to_string(),
        drive_bucket: "knowledgebase-test".to_string(),
        drive_object_key: drive_object_key.to_string(),
    }
}

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

    let metadata = SqliteDriveImportMetadataStore::new(pool.clone(), tenant_id);
    let service = KnowledgeDriveImportService::new(&drive, &metadata);

    let result = service
        .import_drive_object(resolved_drive_import_request(
            "Quarterly Report",
            "node-quarterly-report",
            "incoming/quarterly-report.md",
            "drive-quarterly-report",
        ))
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
        FROM kb_document_version
        WHERE id = $1
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

    let current_version_id: Option<i64> =
        sqlx::query_scalar("SELECT current_version_id FROM kb_document WHERE id = $1")
            .bind(result.document.id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(current_version_id, Some(result.version.id as i64));

    let job_state: i64 = sqlx::query_scalar("SELECT state FROM kb_ingestion_job WHERE id = $1")
        .bind(result.job.id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_state, 0);

    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_source")
        .fetch_one(&pool)
        .await
        .unwrap();
    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_ref_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_drive_object_ref")
        .fetch_one(&pool)
        .await
        .unwrap();
    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_ingestion_job")
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
async fn sqlite_drive_import_quota_rolls_back_entire_metadata_chain() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9010_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    drive
        .put_text("incoming/quota-report.md", "original_document", "# Report")
        .await
        .unwrap();
    let limits = KnowledgebaseTenantQuotaLimits {
        max_documents: 0,
        ..KnowledgebaseTenantQuotaLimits::default()
    };
    let metadata =
        SqliteDriveImportMetadataStore::new(pool.clone(), tenant_id).with_quota_limits(limits);
    let service = KnowledgeDriveImportService::new(&drive, &metadata);

    let error = service
        .import_drive_object(resolved_drive_import_request(
            "Quota Report",
            "node-quota-report",
            "incoming/quota-report.md",
            "drive-quota-report",
        ))
        .await
        .expect_err("drive import must exceed document quota");
    assert!(error.to_string().contains("quota exceeded"));

    for table in [
        "kb_source",
        "kb_document",
        "kb_document_version",
        "kb_drive_object_ref",
        "kb_ingestion_job",
    ] {
        let query = format!("SELECT COUNT(*) FROM {table} WHERE tenant_id = $1");
        let count: i64 = sqlx::query_scalar(&query)
            .bind(tenant_id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "quota failure must roll back {table}");
    }
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

    let metadata = SqliteDriveImportMetadataStore::new(pool.clone(), tenant_id);
    let service = KnowledgeDriveImportService::new(&drive, &metadata);

    let request = resolved_drive_import_request(
        "Quarterly Report",
        "node-quarterly-report",
        "incoming/quarterly-report.md",
        "drive-quarterly-report",
    );

    let first = service.import_drive_object(request.clone()).await.unwrap();
    let replay = service.import_drive_object(request).await.unwrap();

    assert_eq!(first.job.id, replay.job.id);
    assert_eq!(first.source.id, replay.source.id);
    assert_eq!(first.document.id, replay.document.id);
    assert_eq!(first.version.id, replay.version.id);
    assert_eq!(first.original_object_ref.id, replay.original_object_ref.id);

    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_source")
        .fetch_one(&pool)
        .await
        .unwrap();
    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_ref_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_drive_object_ref")
        .fetch_one(&pool)
        .await
        .unwrap();
    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_ingestion_job")
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
async fn sqlite_drive_import_rejects_same_idempotency_key_for_different_drive_object() {
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
    drive
        .put_text(
            "incoming/other-report.md",
            "original_document",
            "# Other Report",
        )
        .await
        .unwrap();

    let metadata = SqliteDriveImportMetadataStore::new(pool.clone(), tenant_id);
    let service = KnowledgeDriveImportService::new(&drive, &metadata);

    service
        .import_drive_object(resolved_drive_import_request(
            "Quarterly Report",
            "node-quarterly-report",
            "incoming/quarterly-report.md",
            "drive-quarterly-report",
        ))
        .await
        .unwrap();

    let error = service
        .import_drive_object(resolved_drive_import_request(
            "Other Report",
            "node-other-report",
            "incoming/other-report.md",
            "drive-quarterly-report",
        ))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("idempotency_key"));

    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_source")
        .fetch_one(&pool)
        .await
        .unwrap();
    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    let object_ref_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_drive_object_ref")
        .fetch_one(&pool)
        .await
        .unwrap();
    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_ingestion_job")
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
async fn sqlite_drive_import_persists_drive_node_binding_for_browser_projection() {
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

    let metadata = SqliteDriveImportMetadataStore::new(pool.clone(), tenant_id);
    let service = KnowledgeDriveImportService::new(&drive, &metadata);

    let result = service
        .import_drive_object(resolved_drive_import_request(
            "Quarterly Report",
            "node-report",
            "incoming/quarterly-report.md",
            "drive-quarterly-report",
        ))
        .await
        .unwrap();

    assert_eq!(
        result.document.original_file_drive_node_id.as_deref(),
        Some("node-report")
    );
    assert_eq!(
        result.original_object_ref.drive_space_id.as_deref(),
        Some("drv-kb-001")
    );
    assert_eq!(
        result.original_object_ref.drive_node_id.as_deref(),
        Some("node-report")
    );

    let document_node_id: Option<String> =
        sqlx::query_scalar("SELECT original_file_drive_node_id FROM kb_document WHERE id = $1")
            .bind(result.document.id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(document_node_id.as_deref(), Some("node-report"));

    let object_ref_row = sqlx::query(
        r#"
        SELECT drive_space_id, drive_node_id
        FROM kb_drive_object_ref
        WHERE id = ?
        "#,
    )
    .bind(result.original_object_ref.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        object_ref_row
            .get::<Option<String>, _>("drive_space_id")
            .as_deref(),
        Some("drv-kb-001")
    );
    assert_eq!(
        object_ref_row
            .get::<Option<String>, _>("drive_node_id")
            .as_deref(),
        Some("node-report")
    );
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
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .unwrap();
    let other_space = store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 8,
            source_type: "api".to_string(),
            idempotency_key: "shared-key".to_string(),
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .unwrap();
    let retry_first = store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 7,
            source_type: "api".to_string(),
            idempotency_key: "shared-key".to_string(),
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .unwrap();

    assert!(first.created);
    assert!(other_space.created);
    assert!(!retry_first.created);
    assert_ne!(first.job.id, other_space.job.id);
    assert_eq!(first.job.id, retry_first.job.id);

    let job_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_ingestion_job")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 2);
}

#[tokio::test]
async fn sqlite_workers_claim_each_queued_ingestion_job_once() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = Arc::new(SqliteIngestionJobStore::new(pool, 9001));
    let job = create_ingestion_job(&store, 7, "drive_object", "claim-once").await;
    let barrier = Arc::new(Barrier::new(3));

    let mut workers = Vec::new();
    for worker_number in 0..2 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        workers.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .claim_ingestion_jobs(ClaimIngestionJobsRequest {
                    claim_owner: format!("worker-{worker_number}"),
                    lease_duration: Duration::minutes(5),
                    limit: 20,
                })
                .await
                .unwrap()
        }));
    }
    barrier.wait().await;

    let mut claimed = Vec::new();
    for worker in workers {
        claimed.extend(worker.await.unwrap());
    }

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].job.id, job.id);
    assert_eq!(claimed[0].job.state, IngestionJobState::Running);
    assert_eq!(claimed[0].attempt_count, 1);
}

#[tokio::test]
async fn sqlite_expired_ingestion_lease_is_reclaimed_and_stale_worker_is_fenced() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteIngestionJobStore::new(pool.clone(), 9001);
    let job = create_ingestion_job(&store, 7, "drive_object", "lease-reclaim").await;

    let first = store
        .claim_ingestion_jobs(ClaimIngestionJobsRequest {
            claim_owner: "worker-a".to_string(),
            lease_duration: Duration::minutes(5),
            limit: 1,
        })
        .await
        .unwrap()
        .pop()
        .unwrap();
    let renewed_until = store
        .renew_ingestion_job_lease(job.id, &first.claim_token, Duration::minutes(5))
        .await
        .unwrap();
    assert!(renewed_until >= first.lease_expires_at);
    assert!(store
        .renew_ingestion_job_lease(job.id, "not-the-current-token", Duration::minutes(5))
        .await
        .is_err());
    sqlx::query(
        "UPDATE kb_ingestion_job SET lease_expires_at = '2026-01-01T00:00:00Z' WHERE id = $1",
    )
    .bind(job.id as i64)
    .execute(&pool)
    .await
    .unwrap();

    let second = store
        .claim_ingestion_jobs(ClaimIngestionJobsRequest {
            claim_owner: "worker-b".to_string(),
            lease_duration: Duration::minutes(5),
            limit: 1,
        })
        .await
        .unwrap()
        .pop()
        .unwrap();

    assert_ne!(first.claim_token, second.claim_token);
    assert_eq!(second.attempt_count, 2);
    assert!(store
        .fail_claimed_ingestion_job(job.id, &first.claim_token, "stale failure".to_string())
        .await
        .is_err());
    let failed = store
        .fail_claimed_ingestion_job(job.id, &second.claim_token, "current failure".to_string())
        .await
        .unwrap();
    assert_eq!(failed.state, IngestionJobState::Failed);

    let lease_fields: (Option<String>, Option<String>, Option<String>, i64) = sqlx::query_as(
        "SELECT claim_owner, claim_token, CAST(lease_expires_at AS TEXT), attempt_count FROM kb_ingestion_job WHERE id = $1",
    )
    .bind(job.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(lease_fields, (None, None, None, 2));
}

#[tokio::test]
async fn sqlite_inflight_count_recovers_stale_queued_and_running_upload_sessions() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let store = SqliteIngestionJobStore::new(pool.clone(), tenant_id);

    let stale_queued = create_ingestion_job(&store, 7, "upload_session", "stale-queued").await;
    let stale_running = create_ingestion_job(&store, 7, "upload_session", "stale-running").await;
    store
        .update_job_state(
            stale_running.id,
            IngestionJobState::Queued,
            IngestionJobState::Running,
            None,
        )
        .await
        .unwrap();
    let fresh_upload = create_ingestion_job(&store, 7, "upload_session", "fresh-upload").await;
    let stale_api = create_ingestion_job(&store, 7, "api", "stale-api").await;
    sqlx::query(
        r#"
        UPDATE kb_ingestion_job
        SET created_at = '2026-07-01T00:00:00Z', updated_at = '2026-07-01T00:00:00Z'
        WHERE tenant_id = $1 AND id IN ($2, $3, $4)
        "#,
    )
    .bind(tenant_id as i64)
    .bind(stale_queued.id as i64)
    .bind(stale_running.id as i64)
    .bind(stale_api.id as i64)
    .execute(&pool)
    .await
    .unwrap();

    let inflight = store.count_inflight_jobs().await.unwrap();

    assert_eq!(inflight, 2);
    for job_id in [stale_queued.id, stale_running.id] {
        let recovered = store.get_job(job_id).await.unwrap();
        assert_eq!(recovered.state, IngestionJobState::Failed);
        assert!(recovered
            .error_message
            .as_deref()
            .is_some_and(|detail| detail.contains("expired")));
    }
    assert_eq!(
        store.get_job(fresh_upload.id).await.unwrap().state,
        IngestionJobState::Queued
    );
    assert_eq!(
        store.get_job(stale_api.id).await.unwrap().state,
        IngestionJobState::Queued
    );
}

#[tokio::test]
async fn sqlite_stale_upload_session_recovery_is_bounded_without_consuming_quota() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let store = SqliteIngestionJobStore::new(pool.clone(), tenant_id);

    for job_number in 0..101_u64 {
        create_ingestion_job(
            &store,
            7,
            "upload_session",
            format!("stale-batch-{job_number}").as_str(),
        )
        .await;
    }
    sqlx::query(
        r#"
        UPDATE kb_ingestion_job
        SET created_at = '2026-07-01T00:00:00Z', updated_at = '2026-07-01T00:00:00Z'
        WHERE tenant_id = $1 AND job_type = 'upload_session'
        "#,
    )
    .bind(tenant_id as i64)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(store.count_inflight_jobs().await.unwrap(), 0);
    let remaining_stale: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM kb_ingestion_job
        WHERE tenant_id = $1
          AND job_type = 'upload_session'
          AND state IN (0, 1)
        "#,
    )
    .bind(tenant_id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        remaining_stale, 1,
        "cleanup must remain bounded to one batch"
    );
    assert_eq!(store.count_inflight_jobs().await.unwrap(), 0);
}

#[tokio::test]
async fn sqlite_stale_upload_session_scan_uses_existing_state_index() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let plan = sqlx::query(
        r#"
        EXPLAIN QUERY PLAN
        SELECT id
        FROM kb_ingestion_job
        WHERE tenant_id = $1
          AND state IN ($2, $3)
          AND status = $4
          AND job_type = 'upload_session'
          AND created_at <= $5
        ORDER BY id ASC
        LIMIT $6
        "#,
    )
    .bind(9001_i64)
    .bind(0_i64)
    .bind(1_i64)
    .bind(1_i64)
    .bind("2026-07-09T00:00:00Z")
    .bind(100_i64)
    .fetch_all(&pool)
    .await
    .unwrap();
    let details = plan
        .iter()
        .map(|row| row.try_get::<String, _>("detail").unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        details.contains("idx_kb_ingestion_job_tenant_state_status")
            || details.contains("idx_kb_ingestion_job_claimable"),
        "stale upload-session scan must use a bounded ingestion-job index: {details}"
    );
}

#[tokio::test]
async fn sqlite_ingestion_job_quota_is_atomic_under_concurrent_creates() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let limits = KnowledgebaseTenantQuotaLimits {
        max_concurrent_ingest_jobs: 2,
        ..KnowledgebaseTenantQuotaLimits::default()
    };
    let store = Arc::new(SqliteIngestionJobStore::new(pool, 9001).with_quota_limits(limits));
    let barrier = Arc::new(Barrier::new(11));
    let mut tasks = Vec::new();
    for index in 0..10_u64 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .create_or_get_job(CreateIngestionJobRecord {
                    space_id: 7,
                    source_type: "api".to_string(),
                    idempotency_key: format!("atomic-ingest-{index}"),
                    idempotency_fingerprint_sha256_hex: None,
                })
                .await
        }));
    }
    barrier.wait().await;

    let mut succeeded = 0;
    let mut rejected = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => succeeded += 1,
            Err(error) if error.to_string().contains("quota exceeded") => rejected += 1,
            Err(error) => panic!("unexpected create error: {error}"),
        }
    }

    assert_eq!(succeeded, 2);
    assert_eq!(rejected, 8);
}

#[tokio::test]
async fn sqlite_document_quota_is_atomic_under_concurrent_creates() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let limits = KnowledgebaseTenantQuotaLimits {
        max_documents: 3,
        ..KnowledgebaseTenantQuotaLimits::default()
    };
    let store = Arc::new(SqliteKnowledgeDocumentStore::new(pool, 9001).with_quota_limits(limits));
    let barrier = Arc::new(Barrier::new(11));
    let mut tasks = Vec::new();
    for index in 0..10_u64 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .create_document(CreateKnowledgeDocumentRecord {
                    space_id: 7,
                    collection_id: 0,
                    source_id: Some(10_000 + index),
                    identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
                    original_file_drive_node_id: Some(format!("atomic-node-{index}")),
                    title: format!("Atomic document {index}"),
                    mime_type: Some("text/markdown".to_string()),
                    language: Some("en".to_string()),
                })
                .await
        }));
    }
    barrier.wait().await;

    let mut succeeded = 0;
    let mut rejected = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => succeeded += 1,
            Err(error) if error.to_string().contains("quota exceeded") => rejected += 1,
            Err(error) => panic!("unexpected create error: {error}"),
        }
    }

    assert_eq!(succeeded, 3);
    assert_eq!(rejected, 7);
}

#[tokio::test]
async fn sqlite_storage_quota_is_atomic_under_concurrent_object_refs() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let limits = KnowledgebaseTenantQuotaLimits {
        max_storage_bytes: 100,
        ..KnowledgebaseTenantQuotaLimits::default()
    };
    let store =
        Arc::new(SqliteKnowledgeDriveObjectRefStore::new(pool, 9001).with_quota_limits(limits));
    let barrier = Arc::new(Barrier::new(11));
    let mut tasks = Vec::new();
    for index in 0..10_u64 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
                    space_id: 7,
                    drive_space_id: None,
                    drive_node_id: None,
                    logical_path: Some(format!("atomic/{index}.md")),
                    drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
                    drive_storage_provider_id: "provider-kb".to_string(),
                    drive_bucket: "knowledgebase-test".to_string(),
                    drive_object_key: format!("atomic/{index}.md"),
                    drive_object_version: None,
                    drive_etag: None,
                    content_type: Some("text/markdown".to_string()),
                    size_bytes: 30,
                    checksum_sha256_hex: None,
                    object_role: "original_document".to_string(),
                    access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
                })
                .await
        }));
    }
    barrier.wait().await;

    let mut succeeded = 0;
    let mut rejected = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(_) => succeeded += 1,
            Err(error) if error.to_string().contains("quota exceeded") => rejected += 1,
            Err(error) => panic!("unexpected create error: {error}"),
        }
    }

    assert_eq!(succeeded, 3);
    assert_eq!(rejected, 7);
}

#[tokio::test]
async fn sqlite_source_store_rejects_duplicate_source_identity() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeSourceStore::new(pool, 9001);
    let record = CreateKnowledgeSourceRecord {
        space_id: 7,
        source_type: KnowledgeSourceType::DriveObject,
        provider: Some("sdkwork-drive".to_string()),
        drive_bucket: Some("knowledgebase-test".to_string()),
        drive_prefix: Some("incoming/quarterly-report.md".to_string()),
        connector_metadata_json: None,
    };

    let first = store.create_source(record.clone()).await.unwrap();
    let error = store.create_source(record).await.unwrap_err();

    assert_ne!(first.id, 0);
    assert!(
        error.to_string().contains("UNIQUE") || error.to_string().contains("uk_kb_source_identity")
    );
}

#[tokio::test]
async fn sqlite_document_store_rejects_duplicate_document_identity() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeDocumentStore::new(pool.clone(), 9001);
    let record = CreateKnowledgeDocumentRecord {
        space_id: 7,
        collection_id: 0,
        source_id: Some(11),
        identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
        original_file_drive_node_id: Some("node-quarterly-report".to_string()),
        title: "Quarterly Report".to_string(),
        mime_type: Some("text/markdown; charset=utf-8".to_string()),
        language: Some("en".to_string()),
    };

    let first = store.create_document(record.clone()).await.unwrap();
    let error = store.create_document(record).await.unwrap_err();

    assert_ne!(first.id, 0);
    assert!(
        error.to_string().contains("UNIQUE")
            || error.to_string().contains("uk_kb_document_identity")
    );
}

#[tokio::test]
async fn sqlite_document_store_allows_same_source_with_different_drive_nodes() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeDocumentStore::new(pool.clone(), 9001);

    let first = store
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: Some(11),
            identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
            original_file_drive_node_id: Some("node-quarterly-report".to_string()),
            title: "Quarterly Report".to_string(),
            mime_type: Some("text/markdown; charset=utf-8".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();
    let second = store
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: Some(11),
            identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
            original_file_drive_node_id: Some("node-annual-report".to_string()),
            title: "Annual Report".to_string(),
            mime_type: Some("text/markdown; charset=utf-8".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();

    assert_ne!(first.id, second.id);

    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(document_count, 2);
}

#[tokio::test]
async fn sqlite_document_store_rejects_source_only_identity_without_source_id() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeDocumentStore::new(pool.clone(), 9001);

    let error = store
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: None,
            identity_scope: KnowledgeDocumentIdentityScope::SourceOnly,
            original_file_drive_node_id: Some("node-quarterly-report".to_string()),
            title: "Quarterly Report".to_string(),
            mime_type: Some("text/markdown; charset=utf-8".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("source_only document identity requires source_id"));

    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(document_count, 0);
}

#[tokio::test]
async fn sqlite_document_version_create_or_get_heals_missing_current_version_pointer() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let versions = SqliteKnowledgeDocumentVersionStore::new(pool.clone(), tenant_id);

    let object_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-report".to_string()),
            logical_path: Some("incoming/quarterly-report.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: Some("etag".to_string()),
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 42,
            checksum_sha256_hex: Some("checksum".to_string()),
            object_role: "original_document".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();
    let document = documents
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: Some(11),
            identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
            original_file_drive_node_id: Some("node-report".to_string()),
            title: "Quarterly Report".to_string(),
            mime_type: Some("text/markdown; charset=utf-8".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();
    let version_record = CreateKnowledgeDocumentVersionRecord {
        document_id: document.id,
        version_no: 1,
        original_object_ref_id: object_ref.id,
        checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
        size_bytes: object_ref.size_bytes,
        mime_type: object_ref.content_type.clone(),
    };

    let first = versions
        .create_or_get_document_version(version_record.clone())
        .await
        .unwrap();
    sqlx::query("UPDATE kb_document SET current_version_id = NULL WHERE id = $1")
        .bind(document.id as i64)
        .execute(&pool)
        .await
        .unwrap();

    let replay = versions
        .create_or_get_document_version(version_record)
        .await
        .unwrap();

    assert_eq!(replay.id, first.id);
    let current_version_id: Option<i64> =
        sqlx::query_scalar("SELECT current_version_id FROM kb_document WHERE id = $1")
            .bind(document.id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(current_version_id, Some(first.id as i64));
}

#[tokio::test]
async fn sqlite_ingestion_job_store_completes_chunks_job_and_outbox_atomically() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let jobs = SqliteIngestionJobStore::new(pool.clone(), tenant_id);
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let versions = SqliteKnowledgeDocumentVersionStore::new(pool.clone(), tenant_id);
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);

    let object_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: None,
            drive_node_id: Some("node-report".to_string()),
            logical_path: Some("incoming/report.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/report.md".to_string(),
            drive_object_version: None,
            drive_etag: None,
            content_type: Some("text/markdown".to_string()),
            size_bytes: 12,
            checksum_sha256_hex: None,
            object_role: "original_document".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();
    let document = documents
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: Some(11),
            identity_scope: KnowledgeDocumentIdentityScope::SourceOnly,
            original_file_drive_node_id: Some("node-report".to_string()),
            title: "Report".to_string(),
            mime_type: Some("text/markdown".to_string()),
            language: None,
        })
        .await
        .unwrap();
    let version = versions
        .create_or_get_document_version(CreateKnowledgeDocumentVersionRecord {
            document_id: document.id,
            version_no: 1,
            original_object_ref_id: object_ref.id,
            checksum_sha256_hex: None,
            size_bytes: 12,
            mime_type: Some("text/markdown".to_string()),
        })
        .await
        .unwrap();

    let created = jobs
        .create_or_get_job(CreateIngestionJobRecord {
            space_id: 7,
            source_type: "drive_object".to_string(),
            idempotency_key: "drive-atomic-1".to_string(),
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .unwrap();
    let claimed = jobs
        .claim_ingestion_jobs(ClaimIngestionJobsRequest {
            claim_owner: "completion-worker".to_string(),
            lease_duration: Duration::minutes(5),
            limit: 1,
        })
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(claimed.job.id, created.job.id);
    let running = claimed.job;
    let chunks = split_markdown_chunks(
        7,
        document.id,
        version.id,
        "# Report\n\nIndexed atomically.",
    );
    let completed = jobs
        .complete_running_ingestion_with_chunks_and_outbox(CompleteRunningIngestionRecord {
            job_id: running.id,
            claim_token: Some(claimed.claim_token),
            document_version_id: version.id,
            chunks,
            outbox: ingest_success_outbox_record(&running),
        })
        .await
        .unwrap();

    assert_eq!(completed.job.state, IngestionJobState::Succeeded);
    assert_eq!(completed.chunk_count, 2);

    let chunk_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_chunk WHERE tenant_id = $1 AND document_version_id = $2",
    )
    .bind(tenant_id as i64)
    .bind(version.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(chunk_count, 2);

    let outbox_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = $1 AND aggregate_id = $2 AND event_type = 'knowledge.ingest.succeeded'",
    )
    .bind(tenant_id as i64)
    .bind(running.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(outbox_count, 1);
}

#[tokio::test]
async fn sqlite_markdown_index_metadata_persists_object_ref_document_version_chain() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9002_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    let payload_object_ref = drive
        .put_text(
            "inbox/api/42/payload.md",
            "api_payload",
            "# API\n\nPayload.",
        )
        .await
        .unwrap();

    let metadata = SqliteMarkdownIndexMetadataStore::new(pool.clone(), tenant_id);
    let indexer = KnowledgeApiMarkdownIndexService::new(&metadata);
    let prepared = indexer
        .prepare_payload_markdown_index(
            7,
            MarkdownIndexSourceBinding::Create(CreateKnowledgeSourceRecord {
                space_id: 7,
                source_type: KnowledgeSourceType::Api,
                provider: Some("api-ingest".to_string()),
                drive_bucket: None,
                drive_prefix: Some("inbox/api/42".to_string()),
                connector_metadata_json: None,
            }),
            "API Payload",
            "# API\n\nPayload.",
            &payload_object_ref,
            Some("drive-space-uuid-7"),
        )
        .await
        .unwrap();

    assert_ne!(prepared.document_version_id, 0);
    assert_eq!(prepared.chunk_records.len(), 2);

    let version_row = sqlx::query(
        r#"
        SELECT tenant_id, document_id, original_object_ref_id, parse_state, index_state
        FROM kb_document_version
        WHERE id = $1
        "#,
    )
    .bind(prepared.document_version_id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(version_row.get::<i64, _>("tenant_id"), tenant_id as i64);
    assert_eq!(version_row.get::<i64, _>("parse_state"), 0);
    assert_eq!(version_row.get::<i64, _>("index_state"), 0);

    let document_id: i64 = version_row.get("document_id");
    let object_ref_id: i64 = version_row.get("original_object_ref_id");
    assert_ne!(document_id, 0);
    assert_ne!(object_ref_id, 0);

    let drive_space_id: Option<String> =
        sqlx::query_scalar("SELECT drive_space_id FROM kb_drive_object_ref WHERE id = $1")
            .bind(object_ref_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(drive_space_id.as_deref(), Some("drive-space-uuid-7"));

    let current_version_id: Option<i64> =
        sqlx::query_scalar("SELECT current_version_id FROM kb_document WHERE id = $1")
            .bind(document_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        current_version_id,
        Some(prepared.document_version_id as i64)
    );

    let object_ref_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_drive_object_ref")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(object_ref_count, 1);
}

#[tokio::test]
async fn sqlite_markdown_index_metadata_replay_reuses_document_version_chain() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9003_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    let payload_object_ref = drive
        .put_text(
            "inbox/api/99/payload.md",
            "api_payload",
            "# Replay\n\nTest.",
        )
        .await
        .unwrap();

    let source_binding = MarkdownIndexSourceBinding::Create(CreateKnowledgeSourceRecord {
        space_id: 7,
        source_type: KnowledgeSourceType::Api,
        provider: Some("api-ingest".to_string()),
        drive_bucket: None,
        drive_prefix: Some("inbox/api/99".to_string()),
        connector_metadata_json: None,
    });

    let metadata = SqliteMarkdownIndexMetadataStore::new(pool.clone(), tenant_id);
    let indexer = KnowledgeApiMarkdownIndexService::new(&metadata);
    let first = indexer
        .prepare_payload_markdown_index(
            7,
            source_binding.clone(),
            "Replay Payload",
            "# Replay\n\nTest.",
            &payload_object_ref,
            None,
        )
        .await
        .unwrap();
    let second = indexer
        .prepare_payload_markdown_index(
            7,
            source_binding,
            "Replay Payload",
            "# Replay\n\nTest.",
            &payload_object_ref,
            None,
        )
        .await
        .unwrap();

    assert_eq!(first.document_version_id, second.document_version_id);
    assert_eq!(first.chunk_records.len(), second.chunk_records.len());

    let document_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(document_count, 1);

    let version_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(version_count, 1);
}

#[tokio::test]
async fn sqlite_markdown_metadata_quota_allows_replay_and_rolls_back_new_document() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9004_u64;
    let drive = FakeKnowledgeDriveStorage::default();
    let first_object = drive
        .put_text("inbox/api/quota-1.md", "api_payload", "# First")
        .await
        .unwrap();
    let second_object = drive
        .put_text("inbox/api/quota-2.md", "api_payload", "# Second")
        .await
        .unwrap();
    let limits = KnowledgebaseTenantQuotaLimits {
        max_documents: 1,
        ..KnowledgebaseTenantQuotaLimits::default()
    };
    let metadata =
        SqliteMarkdownIndexMetadataStore::new(pool.clone(), tenant_id).with_quota_limits(limits);
    let indexer = KnowledgeApiMarkdownIndexService::new(&metadata);
    let first_source = MarkdownIndexSourceBinding::Create(CreateKnowledgeSourceRecord {
        space_id: 7,
        source_type: KnowledgeSourceType::Api,
        provider: Some("api-ingest".to_string()),
        drive_bucket: None,
        drive_prefix: Some("inbox/api/quota-1".to_string()),
        connector_metadata_json: None,
    });

    let first = indexer
        .prepare_payload_markdown_index(
            7,
            first_source.clone(),
            "First",
            "# First",
            &first_object,
            None,
        )
        .await
        .unwrap();
    let replay = indexer
        .prepare_payload_markdown_index(7, first_source, "First", "# First", &first_object, None)
        .await
        .unwrap();
    assert_eq!(first.document_version_id, replay.document_version_id);

    let error = indexer
        .prepare_payload_markdown_index(
            7,
            MarkdownIndexSourceBinding::Create(CreateKnowledgeSourceRecord {
                space_id: 7,
                source_type: KnowledgeSourceType::Api,
                provider: Some("api-ingest".to_string()),
                drive_bucket: None,
                drive_prefix: Some("inbox/api/quota-2".to_string()),
                connector_metadata_json: None,
            }),
            "Second",
            "# Second",
            &second_object,
            None,
        )
        .await
        .expect_err("new document must exceed quota");
    assert!(error.to_string().contains("quota exceeded"));

    let document_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_document WHERE tenant_id = $1 AND status = 1")
            .bind(tenant_id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    let object_ref_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_drive_object_ref WHERE tenant_id = $1 AND status = 1",
    )
    .bind(tenant_id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(document_count, 1);
    assert_eq!(
        object_ref_count, 1,
        "quota failure must roll back object ref"
    );
}

async fn create_ingestion_job(
    store: &SqliteIngestionJobStore,
    space_id: u64,
    source_type: &str,
    idempotency_key: &str,
) -> sdkwork_knowledgebase_contract::ingest::IngestionJob {
    store
        .create_or_get_job(CreateIngestionJobRecord {
            space_id,
            source_type: source_type.to_string(),
            idempotency_key: idempotency_key.to_string(),
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .unwrap()
        .job
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
