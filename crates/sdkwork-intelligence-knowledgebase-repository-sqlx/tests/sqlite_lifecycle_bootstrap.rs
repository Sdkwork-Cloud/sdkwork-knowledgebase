use sdkwork_intelligence_knowledgebase_repository_sqlx::connect_knowledgebase_and_install_schema;
use sqlx::Row;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn sqlite_file_bootstrap_uses_application_database_lifecycle() {
    let database_url = sqlite_test_database_url("sqlite-lifecycle-bootstrap");
    let pool = connect_knowledgebase_and_install_schema(&database_url)
        .await
        .expect("connect sqlite through knowledgebase lifecycle");

    for table in [
        "ops_schema_migration_history",
        "ops_database_installation_state",
        "kb_space",
        "kb_chunk_fts",
    ] {
        assert!(
            sqlite_table_exists(&pool, table).await,
            "expected lifecycle bootstrap to create table {table}"
        );
    }

    for (table, column) in [
        ("kb_space", "knowledge_mode"),
        ("kb_embedding", "vector_json"),
        ("kb_agent_profile", "knowledge_mode"),
        ("kb_agent_profile", "agent_implementation_id"),
        ("kb_outbox_event", "last_error"),
        ("kb_outbox_event", "retry_count"),
        ("kb_outbox_event", "claimed_at"),
        ("web_audit_event", "expires_at"),
    ] {
        assert!(
            sqlite_column_exists(&pool, table, column).await,
            "expected lifecycle bootstrap to create column {table}.{column}"
        );
    }
}

async fn sqlite_table_exists(pool: &sqlx::AnyPool, table: &str) -> bool {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sqlite_master
        WHERE type = 'table' AND name = ?
        "#,
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .expect("query sqlite schema");
    count > 0
}

async fn sqlite_column_exists(pool: &sqlx::AnyPool, table: &str, column: &str) -> bool {
    let statement = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&statement)
        .fetch_all(pool)
        .await
        .expect("query sqlite table columns");
    rows.iter().any(|row| {
        let name: String = row.try_get("name").expect("sqlite pragma column name");
        name == column
    })
}

fn sqlite_test_database_url(test_name: &str) -> String {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("repository-sqlite-tests")
        .join(format!(
            "{}-{}-{}-{}",
            test_name,
            std::process::id(),
            nanos,
            sequence
        ));
    std::fs::create_dir_all(&test_root).expect("create sqlite lifecycle test directory");
    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    format!("sqlite://{relative_database_path}?mode=rwc")
}
