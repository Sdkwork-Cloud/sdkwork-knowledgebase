//! Kubernetes-style liveness/readiness helpers for Knowledgebase HTTP surfaces.

use axum::Json;
use serde_json::{json, Value};

pub const LIVEZ: &str = "/livez";
pub const READYZ: &str = "/readyz";
pub const HEALTHZ: &str = "/healthz";

pub const READINESS_UNAVAILABLE_CODE: &str = "dependencies_not_ready";
pub const READINESS_UNAVAILABLE_DETAIL: &str =
    "Service dependencies are not ready. Please try again later.";

pub fn livez_response() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub fn readyz_ok_response() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub fn healthz_ok_response() -> Json<Value> {
    readyz_ok_response()
}
