use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use sdkwork_utils_rust::is_blank;
use tracing_subscriber::{fmt, EnvFilter};
use uuid::Uuid;

use crate::request_correlation::request_id_scope;

/// Initialize structured tracing for Knowledgebase API/worker processes.
pub fn init_tracing(service_name: &str) {
    init_tracing_subscriber(service_name);
    tracing::info!(service = service_name, "knowledgebase tracing initialized");
}

/// Select OTLP export when configured, otherwise structured tracing.
pub fn init_tracing_from_env(service_name: &str) {
    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())));

    #[cfg(feature = "otel")]
    if otel_endpoint.is_some() {
        if crate::otel::init_otel_tracing(service_name).is_ok() {
            return;
        }
        eprintln!(
            "sdkwork-knowledgebase-observability: OTEL_EXPORTER_OTLP_ENDPOINT is set but OpenTelemetry init failed; falling back to structured tracing"
        );
    }

    #[cfg(not(feature = "otel"))]
    if otel_endpoint.is_some() {
        eprintln!(
            "sdkwork-knowledgebase-observability: OTEL_EXPORTER_OTLP_ENDPOINT is set but observability was built without the `otel` feature; using structured tracing"
        );
    }

    init_tracing(service_name);
}

fn init_tracing_subscriber(service_name: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("info,sdkwork_knowledgebase_{service_name}=debug"))
    });

    let json_logs = std::env::var("SDKWORK_KNOWLEDGEBASE_LOG_FORMAT")
        .map(|value| value.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if json_logs {
        fmt()
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .with_env_filter(filter)
            .init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !is_blank(Some(value)))
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(request_id.clone());

    request_id_scope(request_id.clone(), async move {
        let span = tracing::info_span!(
            "http_request",
            request_id = %request_id,
            method = %request.method(),
            path = %request.uri().path()
        );
        let _guard = span.enter();

        let mut response = next.run(request).await;
        if let Ok(value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert("x-request-id", value);
        }
        response
    })
    .await
}

pub fn wrap_router_with_request_id(router: axum::Router) -> axum::Router {
    router.layer(axum::middleware::from_fn(request_id_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_from_env_falls_back_without_otel_endpoint() {
        let previous_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        init_tracing_from_env("test-service");
        match previous_endpoint {
            Some(value) => std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", value),
            None => std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT"),
        }
    }
}
