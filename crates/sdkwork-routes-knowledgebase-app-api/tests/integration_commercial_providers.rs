use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_routes_knowledgebase_app_api::{dev_auth, paths, KnowledgebaseRuntime};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn configured_sdk_media_providers_return_real_results() {
    let provider = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/images/generations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{
                "url": "https://cdn.example.test/generated/image.png",
                "revised_prompt": "A calm mountain landscape at dawn"
            }]
        })))
        .expect(1)
        .mount(&provider)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "text": "Verified provider transcription"
        })))
        .expect(1)
        .mount(&provider)
        .await;

    std::env::set_var(
        "SDKWORK_CLAW_ROUTER_APPLICATION_OPEN_HTTP_URL",
        provider.uri(),
    );
    std::env::set_var("SDKWORK_CLAW_ROUTER_API_KEY", "integration-provider-key");
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let space_id = create_space(&app).await;

    let ingest = request_json(
        &app,
        Method::POST,
        paths::INGESTS,
        json!({
            "spaceId": space_id,
            "title": "Commercial provider document",
            "idempotencyKey": format!("provider-ingest-{}", unique_suffix()),
            "payloadMarkdown": "# Published knowledge\n\nProvider-backed commercial content."
        }),
    )
    .await;
    assert_eq!(ingest.0, StatusCode::CREATED, "ingest failed: {}", ingest.1);

    let image = request_json(
        &app,
        Method::POST,
        paths::MEDIA_TASKS,
        json!({
            "spaceId": space_id,
            "taskType": "generate_image",
            "prompt": "A calm mountain landscape",
            "aspectMode": "landscape",
            "styleMode": "high"
        }),
    )
    .await;
    assert_eq!(
        image.0,
        StatusCode::CREATED,
        "image task failed: {}",
        image.1
    );
    let image_payload = payload(&image.1);
    assert_eq!(
        image_payload["item"]["url"],
        "https://cdn.example.test/generated/image.png"
    );
    assert_eq!(image_payload["item"]["resolution"], "1536x1024");

    let transcription = request_json(
        &app,
        Method::POST,
        paths::MEDIA_TASKS,
        json!({
            "spaceId": space_id,
            "taskType": "speech_to_text",
            "sourceUrl": "https://media.example.test/audio.mp3"
        }),
    )
    .await;
    assert_eq!(
        transcription.0,
        StatusCode::CREATED,
        "transcription failed: {}",
        transcription.1
    );
    assert_eq!(
        payload(&transcription.1)["item"]["text"],
        "Verified provider transcription"
    );

}

async fn create_space(app: &axum::Router) -> u64 {
    let response = request_json(
        app,
        Method::POST,
        paths::SPACES,
        json!({
            "name": "Commercial Provider Space",
            "description": "Commercial provider integration coverage"
        }),
    )
    .await;
    assert_eq!(
        response.0,
        StatusCode::CREATED,
        "create space failed: {}",
        response.1
    );
    payload(&response.1)["item"]["id"]
        .as_u64()
        .or_else(|| payload(&response.1)["item"]["id"].as_str()?.parse().ok())
        .expect("space ID")
}

async fn request_json(
    app: &axum::Router,
    method: Method,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("route response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("response body");
    let body = serde_json::from_slice(&bytes).expect("response JSON");
    (status, body)
}

fn payload(body: &Value) -> &Value {
    body.get("data").unwrap_or(body)
}

async fn test_runtime() -> KnowledgebaseRuntime {
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join(format!("commercial-providers-{}", unique_suffix()));
    std::fs::create_dir_all(&test_root).expect("create test root");
    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create drive root");
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID", "42");
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
    KnowledgebaseRuntime::connect(&format!("sqlite://{relative_database_path}?mode=rwc"), 1)
        .await
        .expect("initialize runtime")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}
