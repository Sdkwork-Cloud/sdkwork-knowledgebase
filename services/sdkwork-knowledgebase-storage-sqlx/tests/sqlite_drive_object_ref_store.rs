use sdkwork_knowledgebase_product::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_knowledgebase_storage_sqlx::migrations::SQLITE_CORE_MIGRATION;
use sdkwork_knowledgebase_storage_sqlx::SqliteKnowledgeDriveObjectRefStore;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

#[tokio::test]
async fn sqlite_drive_object_ref_store_persists_stable_locator_without_delivery_secrets() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), 9001);

    let object_ref = store
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-001".to_string()),
            logical_path: Some("raw/documents/report.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_bucket: "knowledgebase-source".to_string(),
            drive_object_key: "incoming/quarterly-report.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: Some("etag".to_string()),
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("abc123".to_string()),
            object_role: "original_document".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    assert_ne!(object_ref.id, 0);
    assert_eq!(object_ref.space_id, 7);
    assert_eq!(object_ref.drive_space_id.as_deref(), Some("drv-kb-001"));
    assert_eq!(object_ref.drive_node_id.as_deref(), Some("node-001"));
    assert_eq!(
        object_ref.logical_path.as_deref(),
        Some("raw/documents/report.md")
    );
    assert_eq!(object_ref.drive_provider_kind, SDKWORK_DRIVE_PROVIDER_KIND);
    assert_eq!(object_ref.drive_object_key, "incoming/quarterly-report.md");
    assert_eq!(object_ref.drive_object_version.as_deref(), Some("v1"));
    assert_eq!(object_ref.drive_etag.as_deref(), Some("etag"));
    assert_eq!(object_ref.access_mode, MANAGED_DRIVE_ACCESS_MODE);

    let row = sqlx::query(
        r#"
        SELECT tenant_id, drive_space_id, drive_node_id, logical_path, drive_bucket,
               drive_object_key, drive_object_version, drive_etag, drive_metadata, status
        FROM kb_drive_object_ref
        WHERE id = ?
        "#,
    )
    .bind(object_ref.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("tenant_id"), 9001);
    assert_eq!(
        row.get::<Option<String>, _>("drive_space_id").as_deref(),
        Some("drv-kb-001")
    );
    assert_eq!(
        row.get::<Option<String>, _>("drive_node_id").as_deref(),
        Some("node-001")
    );
    assert_eq!(
        row.get::<Option<String>, _>("logical_path").as_deref(),
        Some("raw/documents/report.md")
    );
    assert_eq!(row.get::<String, _>("drive_bucket"), "knowledgebase-source");
    assert_eq!(
        row.get::<String, _>("drive_object_key"),
        "incoming/quarterly-report.md"
    );
    assert_eq!(
        row.get::<Option<String>, _>("drive_object_version")
            .as_deref(),
        Some("v1")
    );
    assert_eq!(
        row.get::<Option<String>, _>("drive_etag").as_deref(),
        Some("etag")
    );
    assert_eq!(row.get::<Option<String>, _>("drive_metadata"), None);
    assert_eq!(row.get::<i64, _>("status"), 1);

    let unsafe_column_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM pragma_table_info('kb_drive_object_ref')
        WHERE name IN ('presigned_url', 'provider_credentials', 'payload_bytes')
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(unsafe_column_count, 0);
}

#[tokio::test]
async fn sqlite_drive_object_ref_store_keeps_content_versions_for_stable_wiki_paths() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), 9001);

    let first = store
        .create_or_get_object_ref(stable_wiki_object_ref_record(
            "sha256:index-v1",
            "checksum-index-v1",
            128,
        ))
        .await
        .unwrap();
    let replay = store
        .create_or_get_object_ref(stable_wiki_object_ref_record(
            "sha256:index-v1",
            "checksum-index-v1",
            128,
        ))
        .await
        .unwrap();
    let second = store
        .create_or_get_object_ref(stable_wiki_object_ref_record(
            "sha256:index-v2",
            "checksum-index-v2",
            256,
        ))
        .await
        .unwrap();

    assert_eq!(replay.id, first.id);
    assert_ne!(second.id, first.id);
    assert_eq!(
        first.drive_object_version.as_deref(),
        Some("sha256:index-v1")
    );
    assert_eq!(
        second.drive_object_version.as_deref(),
        Some("sha256:index-v2")
    );

    let active_ref_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM kb_drive_object_ref
        WHERE tenant_id = 9001
          AND space_id = 7
          AND drive_bucket = 'knowledgebase-test'
          AND drive_object_key = 'knowledge/tenant/space/wiki/index.md'
          AND object_role = 'wiki_index'
          AND status = 1
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_ref_count, 2);
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

fn stable_wiki_object_ref_record(
    drive_object_version: &str,
    checksum_sha256_hex: &str,
    size_bytes: u64,
) -> CreateKnowledgeDriveObjectRefRecord {
    CreateKnowledgeDriveObjectRefRecord {
        space_id: 7,
        drive_space_id: Some("drv-kb-001".to_string()),
        drive_node_id: Some("node-index".to_string()),
        logical_path: Some("wiki/index.md".to_string()),
        drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
        drive_bucket: "knowledgebase-test".to_string(),
        drive_object_key: "knowledge/tenant/space/wiki/index.md".to_string(),
        drive_object_version: Some(drive_object_version.to_string()),
        drive_etag: None,
        content_type: Some("text/markdown; charset=utf-8".to_string()),
        size_bytes,
        checksum_sha256_hex: Some(checksum_sha256_hex.to_string()),
        object_role: "wiki_index".to_string(),
        access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
    }
}
