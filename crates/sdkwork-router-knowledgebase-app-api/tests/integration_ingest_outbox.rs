use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_knowledgebase_worker::run_maintenance_tick;
use sdkwork_router_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn ingest_appends_outbox_event_and_worker_publishes_it() {
    let runtime = test_runtime().await;
    seed_space(runtime.pool()).await;

    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(1));
    let body = serde_json::json!({
        "spaceId": 1,
        "title": "integration-ingest",
        "idempotencyKey": format!("integration-{}", unique_suffix()),
        "payloadMarkdown": "# Hello\n\nIntegration ingest body."
    });
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::INGESTS)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let job: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(job["state"], "succeeded");

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0 AND event_type = 'knowledge.ingest.succeeded'",
    )
    .fetch_one(runtime.pool())
    .await
    .expect("count pending outbox");
    assert_eq!(pending, 1);

    let tick = run_maintenance_tick(&runtime, 10, 10).await;
    assert_eq!(tick.outbox_published, 1);

    let still_pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0",
    )
    .fetch_one(runtime.pool())
    .await
    .expect("count pending after publish");
    assert_eq!(still_pending, 0);
}

async fn seed_space(pool: &sqlx::AnyPool) {
    sqlx::query(
        r#"
        INSERT INTO kb_space (
            id, uuid, tenant_id, organization_id, name, status,
            llm_wiki_initialized, wiki_log_sequence_counter, created_at, updated_at, version
        )
        VALUES (1, '00000000-0000-4000-8000-000000000001', 1, 1, 'integration-space', 1, 0, 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0)
        "#,
    )
    .execute(pool)
    .await
    .expect("seed kb_space");
}

async fn test_runtime() -> KnowledgebaseRuntime {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("cwd");
    let test_root = work_dir
        .join("target")
        .join("integration-ingest-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("mkdir test root");

    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("mkdir drive root");
    std::env::set_var(
        "SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT",
        drive_root.to_string_lossy().as_ref(),
    );

    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    let database_url = format!("sqlite://{relative_database_path}?mode=rwc");

    KnowledgebaseRuntime::connect(&database_url, 1)
        .await
        .expect("connect runtime")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}
