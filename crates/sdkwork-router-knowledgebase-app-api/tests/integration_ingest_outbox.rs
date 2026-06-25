use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_knowledgebase_worker::run_maintenance_tick;
use sdkwork_router_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn ingest_appends_outbox_event_and_worker_publishes_it() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let space_id = create_space(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::INGESTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": "integration-ingest",
                        "idempotencyKey": format!("integration-{}", unique_suffix()),
                        "payloadMarkdown": "# Hello\n\nIntegration ingest body."
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_text = response_body_string(response).await;
    assert_eq!(status, StatusCode::CREATED, "ingest failed: {body_text}");
    let job: Value = serde_json::from_str(&body_text).expect("parse ingest json");
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

async fn create_space(app: &axum::Router) -> u64 {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::SPACES)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "integration-outbox-space",
                        "description": "integration outbox test space"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body_text = response_body_string(response).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create space failed: {body_text}"
    );
    let body: Value = serde_json::from_str(&body_text).expect("parse create space response");
    body["id"].as_u64().expect("created space id")
}

async fn response_body_string(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    String::from_utf8_lossy(&bytes).into_owned()
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
        .join("integration-ingest-outbox-tests")
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

    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    std::env::remove_var("SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_URL");
    std::env::remove_var("SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_SECRET");

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
