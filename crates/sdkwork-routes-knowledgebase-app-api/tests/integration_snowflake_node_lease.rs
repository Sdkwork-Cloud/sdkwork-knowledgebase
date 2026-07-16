use sdkwork_intelligence_knowledgebase_repository_sqlx::default_knowledge_id_generator;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn runtime_uses_a_healthy_database_backed_snowflake_node_lease() {
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join(format!("snowflake-node-lease-{}", unique_suffix()));
    std::fs::create_dir_all(&test_root).expect("create test root");
    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create drive root");

    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    let database_url = format!("sqlite://{relative_database_path}?mode=rwc");

    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_DATABASE_NODE_LEASE_ENABLED", "true");
    std::env::set_var("SDKWORK_NODE_INSTANCE_ID", "integration-snowflake-lease");
    std::env::set_var(
        "SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT",
        drive_root.to_string_lossy().as_ref(),
    );
    std::env::remove_var("SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID");

    let runtime = KnowledgebaseRuntime::connect(&database_url, 1)
        .await
        .expect("connect runtime");
    runtime.readiness_check().await.expect("runtime readiness");

    let (node_id, instance_identity, expires_at_ms): (i64, String, i64) = sqlx::query_as(
        "SELECT node_id, instance_identity, expires_at_ms FROM sdkwork_node_registry",
    )
    .fetch_one(runtime.pool())
    .await
    .expect("read node registry");
    assert!((0..=1023).contains(&node_id));
    assert!(instance_identity.contains("integration-snowflake-lease"));
    assert!(expires_at_ms > current_epoch_millis());

    let generator = default_knowledge_id_generator();
    let first = generator.next_id().expect("first fenced ID");
    let second = generator.next_id().expect("second fenced ID");
    assert_ne!(first, second);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

fn current_epoch_millis() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis(),
    )
    .expect("current epoch millis fits i64")
}
