use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_router_knowledgebase_app_api::{dev_auth, KnowledgebaseSqliteRuntime};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn hosted_app_router_lists_documents() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/knowledge/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted app api must not return operation_not_implemented for documents.list"
    );

    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn hosted_backend_router_serves_provider_health() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/knowledge/provider_health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted backend must not return operation_not_implemented for providerHealth.retrieve"
    );

    let body = response_body_json(response).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["providerId"], "sdkwork-knowledgebase-sqlite");
}

#[tokio::test]
async fn hosted_backend_router_lists_sources() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/knowledge/sources")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn hosted_open_router_lists_documents() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_open_auth(runtime.build_open_api_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/knowledge/v3/api/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted open api must not return operation_not_implemented for documents.list"
    );

    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

async fn test_runtime() -> KnowledgebaseSqliteRuntime {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("hosted-runtime-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("create hosted runtime test directory");

    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create hosted runtime drive storage root");
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
    KnowledgebaseSqliteRuntime::connect(&database_url, 1)
        .await
        .expect("initialize hosted runtime")
}

async fn response_body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("parse response json")
}
