use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::SQLITE_CORE_MIGRATION;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeSpaceStore, SqliteKnowledgeWikiFileEntryStore,
};
use sdkwork_intelligence_knowledgebase_service::space::KnowledgeSpaceService;
use sdkwork_intelligence_knowledgebase_service::wiki::{
    KnowledgeWikiFileRegistryService, KnowledgeWikiInitializerService,
};
use sdkwork_knowledgebase_contract::space::CreateKnowledgeSpaceRequest;
use sdkwork_knowledgebase_test_support::fake_drive::FakeKnowledgeDriveStorage;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};

#[tokio::test]
async fn sqlite_space_repository_initializes_llm_wiki_standard_files() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001_u64;
    let organization_id = 7001_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let wiki_file_entries = SqliteKnowledgeWikiFileEntryStore::new(pool.clone(), tenant_id);
    let drive = FakeKnowledgeDriveStorage::default();
    let registry = KnowledgeWikiFileRegistryService::new(&wiki_file_entries);
    let wiki_initializer = KnowledgeWikiInitializerService::new(&drive).with_registry(&registry);
    let service = KnowledgeSpaceService::new(&spaces, &wiki_initializer);

    let created = service
        .create_space(CreateKnowledgeSpaceRequest {
                name: "Research Space".to_string(),
                description: Some("LLM Wiki research".to_string()),
                owner_subject_type: Some("user".to_string()),
                owner_subject_id: Some("test-owner".to_string()),
            })
        .await
        .unwrap();

    assert_ne!(created.id, 0);
    assert!(created.llm_wiki_initialized);

    let space_row = sqlx::query(
        r#"
        SELECT tenant_id, organization_id, name, description, llm_wiki_initialized, status
        FROM kb_space
        WHERE id = ?
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
        Some("LLM Wiki research")
    );
    assert_eq!(space_row.get::<i64, _>("llm_wiki_initialized"), 1);
    assert_eq!(space_row.get::<i64, _>("status"), 1);

    let rows = sqlx::query(
        r#"
        SELECT logical_path, entry_type, artifact_role, drive_bucket, drive_object_key,
               checksum_sha256_hex
        FROM kb_wiki_file_entry
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
            "wiki/schema/AGENTS.md",
            "wiki/schema/wiki_schema.yaml",
            "wiki/index.md",
            "wiki/log.md"
        ]
    );

    let entry_types = rows
        .iter()
        .map(|row| row.get::<String, _>("entry_type"))
        .collect::<Vec<_>>();
    assert_eq!(
        entry_types,
        vec!["wiki_schema", "wiki_schema", "wiki_index", "wiki_log"]
    );

    let artifact_roles = rows
        .iter()
        .map(|row| row.get::<String, _>("artifact_role"))
        .collect::<Vec<_>>();
    assert_eq!(
        artifact_roles,
        vec!["wiki_schema", "wiki_schema", "wiki_index", "wiki_log"]
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
