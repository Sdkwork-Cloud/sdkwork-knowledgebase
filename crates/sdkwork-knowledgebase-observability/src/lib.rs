//! HTTP metrics primitives for SDKWork Knowledgebase services.

pub mod audit;
pub use audit::{
    install_audit_persistence, record_backend_admin_operation, record_document_visibility_changed,
    record_space_member_granted, record_space_member_revoked, AuditPersistenceError,
    AuditPersistenceEvent,
};
pub mod billing_metrics;
pub use billing_metrics::{
    record_context_pack_completed, record_ingest_job_failed, record_ingest_job_succeeded,
    record_retrieval_completed,
};
pub mod environment;
pub use environment::{
    deployment_tenant_id, is_development_environment, is_production_like_environment,
    knowledgebase_environment,
};
pub mod tenant_quota;
pub use tenant_quota::KnowledgebaseTenantQuotaLimits;
pub mod health;
pub mod request_correlation;
pub mod tracing_support;

#[cfg(feature = "otel")]
mod otel;

mod okf_metrics;
mod provider_metrics;

use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use okf_metrics::render_okf_prometheus_metrics;
use provider_metrics::render_provider_prometheus_metrics;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub use okf_metrics::{
    record_okf_bundle_exported, record_okf_bundle_imported, record_okf_bundle_lint_completed,
    record_okf_concept_publish, record_okf_concept_upsert,
};
pub use provider_metrics::{install_provider_metrics, KnowledgebaseProviderMetrics};

static REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUEST_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUEST_AUTH_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUEST_DURATION_MS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HEALTH_STATUS: AtomicU64 = AtomicU64::new(1);

/// Update the exported readiness gauge (`1` = dependencies ready, `0` = not ready).
pub fn set_readiness_status(ready: bool) {
    HEALTH_STATUS.store(u64::from(ready), Ordering::Relaxed);
}

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let started = Instant::now();
    REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    let response = next.run(request).await;
    if response.status().is_server_error() {
        REQUEST_ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
    }
    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        REQUEST_AUTH_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
    }
    REQUEST_DURATION_MS_TOTAL.fetch_add(
        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
    response
}

pub async fn metrics_handler() -> impl IntoResponse {
    let body = format!(
        "# HELP knowledge_api_requests_total Total HTTP requests handled by knowledgebase API servers.\n\
         # TYPE knowledge_api_requests_total counter\n\
         knowledge_api_requests_total {}\n\
         # HELP knowledge_api_request_errors_total Total HTTP 5xx responses.\n\
         # TYPE knowledge_api_request_errors_total counter\n\
         knowledge_api_request_errors_total {}\n\
         # HELP knowledge_api_auth_failures_total Total HTTP 401/403 responses.\n\
         # TYPE knowledge_api_auth_failures_total counter\n\
         knowledge_api_auth_failures_total {}\n\
         # HELP knowledge_api_request_duration_ms_total Cumulative request duration in milliseconds.\n\
         # TYPE knowledge_api_request_duration_ms_total counter\n\
         knowledge_api_request_duration_ms_total {}\n\
         # HELP knowledgebase_health_status Service readiness gauge (1=ready, 0=not ready).\n\
         # TYPE knowledgebase_health_status gauge\n\
         knowledgebase_health_status {}\n\
         {}{}{}{}",
        REQUESTS_TOTAL.load(Ordering::Relaxed),
        REQUEST_ERRORS_TOTAL.load(Ordering::Relaxed),
        REQUEST_AUTH_FAILURES_TOTAL.load(Ordering::Relaxed),
        REQUEST_DURATION_MS_TOTAL.load(Ordering::Relaxed),
        HEALTH_STATUS.load(Ordering::Relaxed),
        render_okf_prometheus_metrics(),
        audit::render_audit_prometheus_metrics(),
        billing_metrics::render_billing_prometheus_metrics(),
        render_provider_prometheus_metrics(),
    );

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        body,
    )
}

pub fn metrics_route() -> axum::Router {
    install_provider_metrics();
    axum::Router::new().route("/metrics", axum::routing::get(metrics_handler))
}

pub fn wrap_router_with_metrics(router: axum::Router) -> axum::Router {
    tracing_support::wrap_router_with_request_id(
        router
            .merge(metrics_route())
            .layer(axum::middleware::from_fn(metrics_middleware)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn metrics_endpoint_exports_prometheus_counters() {
        let app =
            wrap_router_with_metrics(axum::Router::new().route("/healthz", get(|| async { "ok" })));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("knowledge_api_requests_total"));
        assert!(text.contains("knowledge_api_auth_failures_total"));
        assert!(text.contains("knowledgebase_health_status"));
        assert!(text.contains("knowledge_audit_document_visibility_changed_total"));
        assert!(text.contains("kb_okf_concept_publish_total"));
        assert!(text.contains("knowledge_retrievals_total"));
        assert!(text.contains("knowledge_context_packs_total"));
        assert!(text.contains("knowledge_provider_operations_total"));
        assert!(text.contains("knowledge_provider_operation_duration_seconds"));
    }
}
