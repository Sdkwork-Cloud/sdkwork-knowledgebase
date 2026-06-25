use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteDriveImportMetadataStore, SqliteIngestionJobStore, SqliteKnowledgeDocumentStore,
    SqliteKnowledgeDocumentVersionStore, SqliteKnowledgeDriveObjectRefStore,
    SqliteKnowledgeSourceStore, SqliteMarkdownIndexMetadataStore,
};
use sdkwork_intelligence_knowledgebase_service::imports::KnowledgeDriveImportService;
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
    CompleteRunningIngestionRecord, CreateIngestionJobRecord, IngestionJobStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::MarkdownIndexSourceBinding;
use sdkwork_knowledgebase_contract::document::KnowledgeDocumentVersionState;
use sdkwork_knowledgebase_contract::ingest::{IngestionJobState, KnowledgeDriveImportRequest};
use sdkwork_knowledgebase_contract::source::KnowledgeSourceType;
use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;
use sqlx::{AnyPool, Row};

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
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_space_id: None,
            drive_node_id: None,
            drive_storage_provider_id: "provider-kb".to_string(),
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

    let request = KnowledgeDriveImportRequest {
        space_id: 7,
        title: "Quarterly Report".to_string(),
        drive_space_id: None,
        drive_node_id: None,
        drive_storage_provider_id: "provider-kb".to_string(),
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
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_space_id: None,
            drive_node_id: None,
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report".to_string(),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();

    let error = service
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Other Report".to_string(),
            drive_space_id: None,
            drive_node_id: None,
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/other-report.md".to_string(),
            idempotency_key: "drive-quarterly-report".to_string(),
            language: Some("en".to_string()),
        })
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
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-report".to_string()),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report".to_string(),
            language: Some("en".to_string()),
        })
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
async fn sqlite_drive_import_enriches_existing_metadata_with_late_drive_node_binding() {
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

    let first = service
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_space_id: None,
            drive_node_id: None,
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report-unbound".to_string(),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();
    let enriched = service
        .import_drive_object(KnowledgeDriveImportRequest {
            space_id: 7,
            title: "Quarterly Report".to_string(),
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-report".to_string()),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            idempotency_key: "drive-quarterly-report-bound".to_string(),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(first.source.id, enriched.source.id);
    assert_eq!(first.document.id, enriched.document.id);
    assert_eq!(first.version.id, enriched.version.id);
    assert_eq!(
        first.original_object_ref.id,
        enriched.original_object_ref.id
    );
    assert_eq!(
        enriched.document.original_file_drive_node_id.as_deref(),
        Some("node-report")
    );
    assert_eq!(
        enriched.original_object_ref.drive_space_id.as_deref(),
        Some("drv-kb-001")
    );
    assert_eq!(
        enriched.original_object_ref.drive_node_id.as_deref(),
        Some("node-report")
    );

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

    assert_eq!(source_count, 1);
    assert_eq!(document_count, 1);
    assert_eq!(version_count, 1);
    assert_eq!(object_ref_count, 1);
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
    let running = jobs
        .update_job_state(
            created.job.id,
            IngestionJobState::Queued,
            IngestionJobState::Running,
            None,
        )
        .await
        .unwrap();
    let chunks = split_markdown_chunks(
        7,
        document.id,
        version.id,
        "# Report\n\nIndexed atomically.",
    );
    let completed = jobs
        .complete_running_ingestion_with_chunks_and_outbox(CompleteRunningIngestionRecord {
            job_id: running.id,
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

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
