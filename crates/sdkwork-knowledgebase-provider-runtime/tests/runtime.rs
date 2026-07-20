use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::{Method, StatusCode};
use sdkwork_knowledgebase_provider_runtime::{
    ProviderErrorCategory, ProviderExecutionContext, ProviderHttpRequest, ProviderOperation,
    ProviderRuntime, ProviderRuntimeConfig, ProviderTargetPolicy, ProviderTelemetry,
    ProviderTelemetryEvent,
};
use sdkwork_utils_rust::SDKWORK_TRACE_ID_HEADER;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

fn test_config(server: &MockServer) -> ProviderRuntimeConfig {
    let mut config = ProviderRuntimeConfig::for_base_url_with_policy(
        &server.uri(),
        ProviderTargetPolicy::Development,
    )
    .expect("test runtime config");
    config.connect_timeout = Duration::from_millis(100);
    config.request_timeout = Duration::from_secs(1);
    config.retry_base_delay = Duration::from_millis(1);
    config.retry_max_delay = Duration::from_millis(5);
    config.max_attempts = 2;
    config
}

fn context() -> ProviderExecutionContext {
    ProviderExecutionContext::new("engine.knowledge.external.test", "trace-provider-001")
}

fn get_request(server: &MockServer, operation: ProviderOperation) -> ProviderHttpRequest {
    ProviderHttpRequest::new(operation, Method::GET, format!("{}/resource", server.uri()))
        .expect("request")
        .idempotent(true)
}

#[tokio::test]
async fn runtime_propagates_trace_and_decodes_bounded_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .and(header(SDKWORK_TRACE_ID_HEADER, "trace-provider-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "ok": true })))
        .expect(1)
        .mount(&server)
        .await;
    let runtime = ProviderRuntime::new(test_config(&server)).expect("runtime");

    let response = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Health))
        .await
        .expect("execute");
    let json: serde_json::Value = response.json().expect("json");

    assert_eq!(json["ok"], true);
}

#[derive(Clone)]
struct FailThenSucceed {
    calls: Arc<AtomicUsize>,
}

impl Respond for FailThenSucceed {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            ResponseTemplate::new(503)
        } else {
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "retried": true }))
        }
    }
}

#[tokio::test]
async fn runtime_retries_only_idempotent_retryable_operations() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(FailThenSucceed {
            calls: calls.clone(),
        })
        .mount(&server)
        .await;
    let runtime = ProviderRuntime::new(test_config(&server)).expect("runtime");

    let response = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Search))
        .await
        .expect("retry succeeds");

    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn runtime_preserves_retry_after_and_rate_limit_category() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "2"))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_attempts = 1;
    let runtime = ProviderRuntime::new(config).expect("runtime");

    let error = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Search))
        .await
        .expect_err("rate limit");

    assert_eq!(error.category, ProviderErrorCategory::RateLimited);
    assert_eq!(error.retry_after, Some(Duration::from_secs(2)));
    assert!(error.retryable);
}

#[tokio::test]
async fn runtime_rejects_oversized_streamed_responses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"12345"))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_response_bytes = 4;
    let runtime = ProviderRuntime::new(config).expect("runtime");

    let error = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Read))
        .await
        .expect_err("body limit");

    assert_eq!(error.category, ProviderErrorCategory::ResponseTooLarge);
    assert!(!error.retryable);
}

#[tokio::test]
async fn runtime_enforces_operation_deadline() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_attempts = 1;
    let runtime = ProviderRuntime::new(config).expect("runtime");
    let mut execution_context = context();
    execution_context.deadline = Some(Instant::now() + Duration::from_millis(10));

    let error = runtime
        .execute(
            &execution_context,
            get_request(&server, ProviderOperation::Read),
        )
        .await
        .expect_err("deadline");

    assert_eq!(error.category, ProviderErrorCategory::Timeout);
}

#[tokio::test]
async fn runtime_opens_circuit_after_bounded_failures() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_attempts = 1;
    config.circuit_failure_threshold = 2;
    let runtime = ProviderRuntime::new(config).expect("runtime");

    for _ in 0..2 {
        let error = runtime
            .execute(&context(), get_request(&server, ProviderOperation::Health))
            .await
            .expect_err("unavailable");
        assert_eq!(error.category, ProviderErrorCategory::Unavailable);
    }
    let error = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Health))
        .await
        .expect_err("circuit open");

    assert_eq!(error.category, ProviderErrorCategory::CircuitOpen);
    assert_eq!(server.received_requests().await.expect("requests").len(), 2);
}

#[tokio::test]
async fn runtime_bulkhead_fails_fast_when_capacity_is_saturated() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_concurrency = 1;
    let runtime = ProviderRuntime::new(config).expect("runtime");
    let first_runtime = runtime.clone();
    let first_context = context();
    let first_request = get_request(&server, ProviderOperation::Read);
    let first =
        tokio::spawn(async move { first_runtime.execute(&first_context, first_request).await });
    tokio::time::sleep(Duration::from_millis(10)).await;

    let error = runtime
        .execute(&context(), get_request(&server, ProviderOperation::Read))
        .await
        .expect_err("bulkhead");

    assert_eq!(error.category, ProviderErrorCategory::BulkheadSaturated);
    first.await.expect("join").expect("first request");
}

#[tokio::test]
async fn runtime_enforces_https_and_exact_configured_origin() {
    let production_error = ProviderRuntimeConfig::for_base_url_with_policy(
        "http://provider.example",
        ProviderTargetPolicy::Production,
    )
    .expect_err("production requires https");
    assert_eq!(
        production_error.category,
        ProviderErrorCategory::InvalidTarget
    );

    let allowed = MockServer::start().await;
    let other = MockServer::start().await;
    let runtime = ProviderRuntime::new(test_config(&allowed)).expect("runtime");
    let error = runtime
        .execute(&context(), get_request(&other, ProviderOperation::Health))
        .await
        .expect_err("origin mismatch");

    assert_eq!(error.category, ProviderErrorCategory::InvalidTarget);
}

#[tokio::test]
async fn runtime_maps_auth_permission_and_not_found_without_retry() {
    for (status, category) in [
        (401, ProviderErrorCategory::Authentication),
        (403, ProviderErrorCategory::PermissionDenied),
        (404, ProviderErrorCategory::NotFound),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/resource"))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&server)
            .await;
        let runtime = ProviderRuntime::new(test_config(&server)).expect("runtime");

        let error = runtime
            .execute(&context(), get_request(&server, ProviderOperation::Read))
            .await
            .expect_err("status error");

        assert_eq!(error.category, category);
        assert!(!error.retryable);
    }
}

#[derive(Default)]
struct RecordingTelemetry {
    events: Mutex<Vec<ProviderTelemetryEvent>>,
}

impl ProviderTelemetry for RecordingTelemetry {
    fn record(&self, event: ProviderTelemetryEvent) {
        self.events.lock().expect("events mutex").push(event);
    }
}

#[tokio::test]
async fn runtime_records_bounded_operation_telemetry() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"ok"))
        .mount(&server)
        .await;
    let telemetry = Arc::new(RecordingTelemetry::default());
    let runtime = ProviderRuntime::new(test_config(&server))
        .expect("runtime")
        .with_telemetry(telemetry.clone());

    runtime
        .execute(&context(), get_request(&server, ProviderOperation::Health))
        .await
        .expect("execute");

    let events = telemetry.events.lock().expect("events mutex");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].operation, ProviderOperation::Health);
    assert_eq!(events[0].attempts, 1);
    assert_eq!(events[0].response_bytes, 2);
    assert!(events[0].error_category.is_none());
}

#[tokio::test]
async fn runtime_never_exposes_upstream_error_body_or_sensitive_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/resource"))
        .respond_with(ResponseTemplate::new(500).set_body_string("upstream-secret-body"))
        .mount(&server)
        .await;
    let mut config = test_config(&server);
    config.max_attempts = 1;
    let runtime = ProviderRuntime::new(config).expect("runtime");
    let request = get_request(&server, ProviderOperation::Read)
        .bearer_auth("credential-secret")
        .expect("auth header");

    let error = runtime
        .execute(&context(), request)
        .await
        .expect_err("upstream error");
    let rendered = error.to_string();

    assert!(!rendered.contains("upstream-secret-body"));
    assert!(!rendered.contains("credential-secret"));
}
