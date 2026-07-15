use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_intelligence_knowledgebase_repository_sqlx::is_postgres_database_url;
use sdkwork_routes_knowledgebase_app_api::{
    paths, KnowledgeAppRequestContext, KnowledgebaseRuntime,
};
use serde_json::json;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, MutexGuard};
use tower::util::ServiceExt;

static POSTGRES_CREATE_SPACE_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn postgres_create_space_test_lock() -> MutexGuard<'static, ()> {
    POSTGRES_CREATE_SPACE_TEST_LOCK.lock().await
}

fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL")
        .ok()
        .filter(|url| is_postgres_database_url(url))
}

#[tokio::test]
async fn postgres_runtime_creates_space_through_app_router() {
    let _guard = postgres_create_space_test_lock().await;
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!(
            "skipping postgres create space integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL to a postgres URL"
        );
        return;
    };

    std::env::set_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID", "100001");
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID", "0");
    std::env::set_var("SDKWORK_CLAW_DATABASE_SCHEMA", "sdkwork_ai_dev");
    let drive_root = std::env::current_dir()
        .expect("current directory")
        .join("target")
        .join("postgres-create-space-integration")
        .join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create drive storage root");
    std::env::set_var(
        "SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT",
        drive_root.to_string_lossy().as_ref(),
    );

    let runtime = KnowledgebaseRuntime::connect(&database_url, 100_001)
        .await
        .expect("initialize postgres knowledgebase runtime");
    let app = runtime.build_full_app_router();

    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::SPACES)
                .header("content-type", "application/json")
                .extension(KnowledgeAppRequestContext {
                    tenant_id: 100_001,
                    actor_id: Some(42),
                    organization_id: Some(0),
                    session_id: None,
                    request_id: "test-request-postgres-create-one".to_string(),
                    trace_id: None,
                    idempotency_key: None,
                })
                .body(Body::from(
                    json!({
                        "name": format!("postgres-create-space-integration-{unique_suffix}"),
                        "description": "full hosted create_space path"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read create space response");
    let body = String::from_utf8(bytes.to_vec()).expect("utf8 response body");
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create space failed: status={status} body={body}"
    );
    let payload: serde_json::Value =
        serde_json::from_str(&body).expect("create space response json");
    assert_eq!(payload["code"], 0, "unexpected envelope code: {body}");
    let item = &payload["data"]["item"];
    assert!(
        json_u64_field(item, "id").is_some_and(|id| id > 0),
        "missing created space id: {body}"
    );
    assert!(
        item["driveSpaceId"]
            .as_str()
            .is_some_and(|value| value.starts_with("kb-")),
        "missing drive space binding: {body}"
    );
    assert_eq!(
        item["okfBundleInitialized"], true,
        "okf bundle should be initialized: {body}"
    );
    let first_drive_space_id = item["driveSpaceId"]
        .as_str()
        .expect("drive space id string")
        .to_string();

    let second = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::SPACES)
                .header("content-type", "application/json")
                .extension(KnowledgeAppRequestContext {
                    tenant_id: 100_001,
                    actor_id: Some(42),
                    organization_id: Some(0),
                    session_id: None,
                    request_id: "test-request-postgres-create-two".to_string(),
                    trace_id: None,
                    idempotency_key: None,
                })
                .body(Body::from(
                    json!({
                        "name": format!("postgres-create-space-integration-second-{unique_suffix}"),
                        "description": "same owner can create another space"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let second_status = second.status();
    let second_bytes = axum::body::to_bytes(second.into_body(), usize::MAX)
        .await
        .expect("read second create space response");
    let second_body = String::from_utf8(second_bytes.to_vec()).expect("utf8 second response body");
    assert_eq!(
        second_status,
        StatusCode::CREATED,
        "second create space failed: status={second_status} body={second_body}"
    );
    let second_payload: serde_json::Value =
        serde_json::from_str(&second_body).expect("second create space response json");
    let second_item = &second_payload["data"]["item"];
    let second_drive_space_id = second_item["driveSpaceId"]
        .as_str()
        .expect("second drive space id");
    assert_ne!(
        second_drive_space_id,
        first_drive_space_id.as_str(),
        "each knowledge space must bind a dedicated drive space"
    );
}

fn json_u64_field(body: &serde_json::Value, field: &str) -> Option<u64> {
    body.get(field)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}
