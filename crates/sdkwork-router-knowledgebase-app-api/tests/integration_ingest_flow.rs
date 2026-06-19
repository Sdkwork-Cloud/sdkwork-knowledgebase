use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_router_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn integration_hosted_ingest_creates_job_and_chunks_markdown() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime).await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

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
                        "idempotencyKey": "integration-ingest-001",
                        "payloadMarkdown": "# Title\n\nFirst paragraph.\n\nSecond paragraph."
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
    let body: serde_json::Value = serde_json::from_str(&body_text).expect("parse ingest json");
    assert_eq!(body["state"], "succeeded");

    let chunk_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_chunk WHERE tenant_id = 1 AND space_id = $1")
            .bind(space_id as i64)
            .fetch_one(runtime.pool())
            .await
            .expect("count chunks");
    assert!(
        chunk_count >= 2,
        "expected markdown chunking to persist chunks"
    );
}

async fn create_space(runtime: &KnowledgebaseRuntime) -> u64 {
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::SPACES)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "integration-space",
                        "description": "integration test space"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_body_json(response).await;
    body["id"].as_u64().expect("space id")
}

async fn test_runtime() -> KnowledgebaseRuntime {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("integration-ingest-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("create integration test directory");
    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create drive storage root");
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
        .expect("initialize integration runtime")
}

async fn response_body_json(response: axum::response::Response) -> serde_json::Value {
    let text = response_body_string(response).await;
    serde_json::from_str(&text).expect("parse response json")
}

async fn response_body_string(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    String::from_utf8(bytes.to_vec()).expect("utf8 response body")
}
