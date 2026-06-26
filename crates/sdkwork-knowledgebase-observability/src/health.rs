//! Kubernetes-style liveness/readiness path constants and web-bootstrap re-exports.

pub use sdkwork_web_bootstrap::{
    healthz_handler, infra_public_path_prefixes, livez_handler, readyz_handler,
    READINESS_DEPENDENCY_UNAVAILABLE,
};

pub const LIVEZ: &str = "/livez";
pub const READYZ: &str = "/readyz";
pub const HEALTHZ: &str = "/healthz";

pub const READINESS_UNAVAILABLE_CODE: &str = "dependencies_not_ready";
pub const READINESS_UNAVAILABLE_DETAIL: &str =
    "Service dependencies are not ready. Please try again later.";
