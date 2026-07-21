use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_routes_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn integration_market_list_bootstraps_from_created_space() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime, "Market Bootstrap Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(paths::MARKET_LISTINGS)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "market list failed: {}",
        response_body_string(response).await
    );
    let body = response_body_json(response).await;
    let items = body["items"].as_array().expect("market items array");
    assert!(
        !items.is_empty(),
        "expected market catalog bootstrap from knowledge spaces"
    );
    assert!(
        items
            .iter()
            .any(|item| item["title"].as_str() == Some("Market Bootstrap Space")),
        "expected created space to appear in market catalog: {body}"
    );
    assert_ne!(items[0]["id"].as_str(), None);
    let _ = space_id;
}

#[tokio::test]
async fn integration_market_subscription_round_trip() {
    let runtime = test_runtime().await;
    let _space_id = create_space(&runtime, "Market Subscription Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(paths::MARKET_LISTINGS)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let listing_id = response_body_json(list_response).await["items"][0]["id"]
        .as_str()
        .expect("listing id")
        .parse::<u64>()
        .expect("numeric listing id");

    let subscribe_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::MARKET_SUBSCRIPTIONS)
                .header("content-type", "application/json")
                .body(Body::from(json!({ "listingId": listing_id }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        subscribe_response.status(),
        StatusCode::CREATED,
        "subscribe failed: {}",
        response_body_string(subscribe_response).await
    );
    let subscribe_body = response_body_json(subscribe_response).await;
    assert_eq!(subscribe_body["accepted"], true);
    assert_eq!(subscribe_body["status"], "completed");

    let subscribed_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(paths::MARKET_LISTINGS)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let subscribed_body = response_body_json(subscribed_list).await;
    let listing_id_wire = listing_id.to_string();
    let subscribed_item = subscribed_body["items"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item["id"].as_str() == Some(listing_id_wire.as_str()))
        })
        .expect("subscribed listing");
    assert_eq!(subscribed_item["isSubscribed"], true);

    let unsubscribe_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "/app/v3/api/knowledge/market/subscriptions/{listing_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unsubscribe_response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response_body_string(unsubscribe_response).await, "");
}

#[tokio::test]
async fn integration_site_upsert_creates_the_site_resource() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime, "Knowledge Site Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let site_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(paths::SPACE_SITE.replace("{space_id}", &space_id.to_string()))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "title": "Knowledge Site",
                        "visibility": "public",
                        "homepageConceptId": null,
                        "themeId": "default",
                        "publishMode": "manual",
                        "expectedVersion": null
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        site_response.status(),
        StatusCode::OK,
        "site upsert failed: {}",
        response_body_string(site_response).await
    );
    let body = response_body_json(site_response).await;
    assert_eq!(body["spaceId"].as_str(), Some(space_id.to_string().as_str()));
    assert_eq!(body["title"], "Knowledge Site");
    assert_eq!(body["visibility"], "public");
    assert_eq!(body["publishMode"], "manual");
}

#[tokio::test]
async fn integration_image_generation_fails_closed_without_a_media_provider() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime, "Media Task Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::MEDIA_TASKS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "taskType": "generate_image",
                        "prompt": "A calm mountain landscape"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_body_json(response).await;
    assert_eq!(body["code"], 50301);
    assert_eq!(body["status"], 503);
    assert!(body.get("detail").is_none());
    assert!(!body.to_string().contains("unsplash.com"));
}

#[tokio::test]
async fn integration_transcription_fails_closed_without_derived_text_or_provider() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime, "Transcription Task Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::MEDIA_TASKS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "taskType": "speech_to_text",
                        "sourceUrl": "https://media.example.invalid/audio.mp3"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_body_json(response).await;
    assert_eq!(body["code"], 50301);
    assert_eq!(body["status"], 503);
    assert!(body.get("detail").is_none());
    assert!(!body.to_string().contains("audio.mp3"));
}

#[tokio::test]
async fn integration_git_sync_rejects_invalid_request() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime, "Git Sync Space").await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::GIT_SYNCS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "spaceId": space_id,
                        "repoUrl": "",
                        "commitMessage": "sync",
                        "idempotencyKey": "integration-git-sync-invalid"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

async fn create_space(runtime: &KnowledgebaseRuntime, name: &str) -> u64 {
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::SPACES)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": name,
                        "description": "Commerce integration test space"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "create space failed: {}",
        response_body_string(response).await
    );
    let body = response_body_json(response).await;
    json_u64_field(&body, "id").expect("space id")
}

fn json_u64_field(body: &Value, field: &str) -> Option<u64> {
    body.get(field)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}

async fn test_runtime() -> KnowledgebaseRuntime {
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID", "42");
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("integration-commerce-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("create integration commerce test directory");
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
        .expect("initialize integration commerce runtime")
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
