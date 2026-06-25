use axum::{http::StatusCode, routing::get, Json, Router};
use sdkwork_router_knowledgebase_app_api::ReadinessCheck;
use serde_json::{json, Value};

async fn livez() -> StatusCode {
    StatusCode::OK
}

async fn readyz_check(readiness: ReadinessCheck) -> Result<Json<Value>, StatusCode> {
    readiness.check().await.map_err(|error| {
        tracing::warn!(?error, "knowledgebase worker readiness check failed");
        sdkwork_knowledgebase_observability::set_readiness_status(false);
        StatusCode::SERVICE_UNAVAILABLE
    })?;
    sdkwork_knowledgebase_observability::set_readiness_status(true);
    Ok(Json(json!({ "status": "ok" })))
}

pub fn worker_health_router(readiness: ReadinessCheck) -> Router {
    let ready_probe = readiness.clone();
    let health_probe = readiness;
    Router::new()
        .route("/livez", get(livez))
        .route(
            "/readyz",
            get(move || {
                let readiness = ready_probe.clone();
                async move { readyz_check(readiness).await }
            }),
        )
        .route(
            "/healthz",
            get(move || {
                let readiness = health_probe.clone();
                async move { readyz_check(readiness).await }
            }),
        )
        .merge(sdkwork_knowledgebase_observability::metrics_route())
}

pub async fn serve_worker_health(listen_addr: &str, readiness: ReadinessCheck) {
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("bind knowledgebase worker health listener");
    tracing::info!(%listen_addr, "knowledgebase worker health endpoint listening");
    axum::serve(listener, worker_health_router(readiness))
        .await
        .expect("serve knowledgebase worker health");
}
