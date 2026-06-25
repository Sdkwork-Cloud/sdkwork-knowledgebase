use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_router_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[tokio::test]
async fn integration_wechat_official_accounts_replace_redacts_secrets_on_list() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let replace_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(paths::WECHAT_OFFICIAL_ACCOUNTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "accounts": [{
                            "id": "acct-1",
                            "name": "Test Account",
                            "type": "subscription",
                            "avatar": "TA",
                            "appId": "wx-test-app-id",
                            "appSecret": "super-secret-value"
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        replace_response.status(),
        StatusCode::OK,
        "replace failed: {}",
        response_body_string(replace_response).await
    );
    let replace_body = response_body_json(replace_response).await;
    assert_eq!(replace_body["accounts"][0]["id"], "acct-1");
    assert!(replace_body["accounts"][0]["appSecret"].is_null());

    let list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(paths::WECHAT_OFFICIAL_ACCOUNTS)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = response_body_json(list_response).await;
    assert_eq!(list_body["accounts"].as_array().map(Vec::len), Some(1));
    assert!(list_body["accounts"][0]["appSecret"].is_null());
}

#[tokio::test]
async fn integration_wechat_publish_rejects_missing_app_secret() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let replace_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(paths::WECHAT_OFFICIAL_ACCOUNTS)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "accounts": [{
                            "id": "acct-2",
                            "name": "No Secret",
                            "type": "subscription",
                            "avatar": "NS",
                            "appId": "wx-no-secret"
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replace_response.status(), StatusCode::OK);

    let publish_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(paths::WECHAT_ARTICLES_PUBLISH)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "accountIds": ["acct-2"],
                        "articles": [{
                            "id": "article-1",
                            "title": "Title",
                            "author": "Author",
                            "content": "<p>Hello</p>"
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish_response.status(), StatusCode::BAD_REQUEST);
    let publish_body = response_body_string(publish_response).await;
    assert!(
        publish_body.contains("appSecret"),
        "expected missing appSecret validation, got: {publish_body}"
    );
}

static WECHAT_INTEGRATION_ENV_LOCK: Mutex<()> = Mutex::new(());

async fn test_runtime() -> KnowledgebaseRuntime {
    let _env_guard = WECHAT_INTEGRATION_ENV_LOCK
        .lock()
        .expect("wechat integration env lock");
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("integration-wechat-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("create integration wechat test directory");
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
        .expect("initialize integration wechat runtime")
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
