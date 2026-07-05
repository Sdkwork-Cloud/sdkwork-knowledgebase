use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeOkfBundleFileStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::okf::{
    OkfBundleFileRegistryService, OkfBundleInitializerService,
};
use sdkwork_intelligence_knowledgebase_service::space::KnowledgeSpaceService;
use sdkwork_knowledgebase_contract::space::CreateKnowledgeSpaceRequest;
use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;
use sqlx::{AnyPool, Row};

#[tokio::test]
async fn sqlite_space_repository_initializes_okf_bundle_standard_files() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let organization_id = 7001_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let bundle_file_entries = SqliteKnowledgeOkfBundleFileStore::new(pool.clone(), tenant_id);
    let drive = FakeKnowledgeDriveStorage::default();
    let registry = OkfBundleFileRegistryService::new(&bundle_file_entries);
    let okf_initializer = OkfBundleInitializerService::new(&drive).with_registry(&registry);
    let service = KnowledgeSpaceService::new(&spaces, &okf_initializer);

    let created = service
        .create_space(CreateKnowledgeSpaceRequest {
            name: "Research Space".to_string(),
            description: Some("OKF research".to_string()),
            owner_subject_type: Some("user".to_string()),
            owner_subject_id: Some("test-owner".to_string()),
            knowledge_mode: Default::default(),
        })
        .await
        .unwrap();

    assert_ne!(created.id, 0);
    assert!(created.okf_bundle_initialized);

    let space_row = sqlx::query(
        r#"
        SELECT tenant_id, organization_id, name, description, okf_bundle_initialized, status
        FROM kb_space
        WHERE id = $1
        "#,
    )
    .bind(created.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(space_row.get::<i64, _>("tenant_id"), tenant_id as i64);
    assert_eq!(
        space_row.get::<i64, _>("organization_id"),
        organization_id as i64
    );
    assert_eq!(space_row.get::<String, _>("name"), "Research Space");
    assert_eq!(
        space_row.get::<Option<String>, _>("description").as_deref(),
        Some("OKF research")
    );
    assert_eq!(space_row.get::<i64, _>("okf_bundle_initialized"), 1);
    assert_eq!(space_row.get::<i64, _>("status"), 1);

    let summary = spaces.summarize_tenant_knowledgebase().await.unwrap();
    assert_eq!(summary.space_count, 1);
    assert_eq!(summary.document_count, 0);
    assert!(summary.created_at.is_some());

    let rows = sqlx::query(
        r#"
        SELECT logical_path, file_kind, artifact_role, drive_bucket, drive_object_key,
               checksum_sha256_hex
        FROM kb_okf_bundle_file
        WHERE tenant_id = ? AND space_id = ?
        ORDER BY id
        "#,
    )
    .bind(tenant_id as i64)
    .bind(created.id as i64)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(rows.len(), 4);

    let logical_paths = rows
        .iter()
        .map(|row| row.get::<String, _>("logical_path"))
        .collect::<Vec<_>>();
    assert_eq!(
        logical_paths,
        vec![
            "okf/schema/AGENTS.md",
            "okf/schema/okf_profile.yaml",
            "okf/index.md",
            "okf/log.md"
        ]
    );

    let file_kinds = rows
        .iter()
        .map(|row| row.get::<String, _>("file_kind"))
        .collect::<Vec<_>>();
    assert_eq!(
        file_kinds,
        vec![
            "bundle_agents",
            "bundle_profile",
            "bundle_index",
            "bundle_log"
        ]
    );

    let artifact_roles = rows
        .iter()
        .map(|row| row.get::<String, _>("artifact_role"))
        .collect::<Vec<_>>();
    assert_eq!(
        artifact_roles,
        vec![
            "bundle_profile",
            "bundle_profile",
            "bundle_index",
            "bundle_log"
        ]
    );

    for row in rows {
        let logical_path = row.get::<String, _>("logical_path");
        assert_eq!(row.get::<String, _>("drive_bucket"), "knowledgebase-test");
        assert_eq!(row.get::<String, _>("drive_object_key"), logical_path);
        assert!(row
            .get::<Option<String>, _>("checksum_sha256_hex")
            .is_some());
    }
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
