use axum::Json;
use serde_json::{json, Value};
use sqlx::AnyPool;

use crate::error::BackendApiProblem;

const DEPENDENCY_UNAVAILABLE_DETAIL: &str = "One or more dependencies are unavailable.";

#[derive(Clone)]
pub struct DbReadinessCheck {
    pool: AnyPool,
}

impl DbReadinessCheck {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }

    pub async fn check(&self) -> Result<(), sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
    }
}

pub async fn livez() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn readyz_probe(
    readiness: Option<DbReadinessCheck>,
) -> Result<Json<Value>, BackendApiProblem> {
    if let Some(readiness) = readiness {
        readiness.check().await.map_err(|error| {
            sdkwork_knowledgebase_observability::set_readiness_status(false);
            eprintln!("[knowledgebase] readiness check failed: {error}");
            BackendApiProblem::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "dependencies_unavailable",
                DEPENDENCY_UNAVAILABLE_DETAIL,
            )
        })?;
    }
    sdkwork_knowledgebase_observability::set_readiness_status(true);
    Ok(Json(json!({ "status": "ok" })))
}

pub async fn readyz_with_state(
    readiness: Option<DbReadinessCheck>,
) -> Result<Json<Value>, BackendApiProblem> {
    readyz_probe(readiness).await
}

pub async fn healthz_with_state(
    readiness: Option<DbReadinessCheck>,
) -> Result<Json<Value>, BackendApiProblem> {
    readyz_probe(readiness).await
}
