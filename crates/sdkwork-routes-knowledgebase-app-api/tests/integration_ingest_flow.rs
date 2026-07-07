use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_routes_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::{json, Value};
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
    assert_eq!(body["data"]["item"]["state"], "succeeded");

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

#[tokio::test]
async fn integration_hosted_ingest_retrieves_document_content_via_contract_route() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime).await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let payload_markdown = "# Title\n\nFirst paragraph.\n\nSecond paragraph.";

    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::INGESTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": "integration-content-retrieve",
                        "idempotencyKey": "integration-content-retrieve-001",
                        "payloadMarkdown": payload_markdown
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        ingest_response.status(),
        StatusCode::CREATED,
        "ingest failed: {}",
        response_body_string(ingest_response).await
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/documents?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = response_body_json(list_response).await;
    let document_id = list_body["items"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| json_u64_field(item, "id"))
        .expect("ingested document id");

    let content_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/documents/{document_id}/content"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        content_response.status(),
        StatusCode::OK,
        "content retrieve failed: {}",
        response_body_string(content_response).await
    );
    let content_body = response_body_json(content_response).await;
    assert_eq!(
        json_u64_field(&content_body, "documentId"),
        Some(document_id)
    );
    assert!(
        content_body["contentMarkdown"]
            .as_str()
            .unwrap_or_default()
            .contains("First paragraph."),
        "content body: {content_body}"
    );
    assert!(
        ["drive_object", "chunk_concat"]
            .contains(&content_body["contentSource"].as_str().unwrap_or_default()),
        "unexpected content source: {content_body}"
    );
}

#[tokio::test]
async fn integration_hosted_ingest_lists_document_versions_via_contract_route() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime).await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::INGESTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": "integration-version-history",
                        "idempotencyKey": "integration-version-history-001",
                        "payloadMarkdown": "# Versioned\n\nVersion body."
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        ingest_response.status(),
        StatusCode::CREATED,
        "ingest failed: {}",
        response_body_string(ingest_response).await
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/documents?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = response_body_json(list_response).await;
    let document_id = list_body["items"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| json_u64_field(item, "id"))
        .expect("ingested document id");

    let versions_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/documents/{document_id}/versions"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        versions_response.status(),
        StatusCode::OK,
        "versions list failed: {}",
        response_body_string(versions_response).await
    );
    let versions_body = response_body_json(versions_response).await;
    let items = versions_body["items"]
        .as_array()
        .expect("version items array");
    assert!(
        !items.is_empty(),
        "ingest should create at least one document version"
    );
    assert_eq!(json_u64_field(&items[0], "documentId"), Some(document_id));
}

#[tokio::test]
async fn integration_hosted_ingest_updates_document_visibility_via_contract_route() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime).await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let ingest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::INGESTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": "integration-visibility",
                        "idempotencyKey": "integration-visibility-001",
                        "payloadMarkdown": "# Visibility\n\nBody."
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        ingest_response.status(),
        StatusCode::CREATED,
        "ingest failed: {}",
        response_body_string(ingest_response).await
    );

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/documents?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = response_body_json(list_response).await;
    let document_id = list_body["items"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| json_u64_field(item, "id"))
        .expect("ingested document id");
    let title = list_body["items"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item["title"].as_str())
        .expect("ingested document title");

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/app/v3/api/knowledge/documents/{document_id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": title,
                        "visibility": "public"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        update_response.status(),
        StatusCode::OK,
        "update visibility failed: {}",
        response_body_string(update_response).await
    );
    let update_body = response_body_json(update_response).await;
    assert_eq!(update_body["visibility"], "public");

    let retrieve_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/app/v3/api/knowledge/documents/{document_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_body = response_body_json(retrieve_response).await;
    assert_eq!(retrieve_body["visibility"], "public");
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
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read create space response");
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create space failed: {}",
        String::from_utf8_lossy(&bytes)
    );
    let body: Value = serde_json::from_slice(&bytes).expect("parse create space response");
    json_item_u64_field(&body, "id").expect("space id")
}

fn json_item_u64_field(body: &Value, field: &str) -> Option<u64> {
    body.pointer("/data/item")
        .and_then(|item| json_u64_field(item, field))
}

fn json_u64_field(body: &Value, field: &str) -> Option<u64> {
    body.get(field)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
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
    let value: serde_json::Value = serde_json::from_str(&text).expect("parse response json");
    sdkwork_knowledgebase_test_support::api_envelope::unwrap_payload_or_envelope(&value)
}

async fn response_body_string(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    String::from_utf8(bytes.to_vec()).expect("utf8 response body")
}
