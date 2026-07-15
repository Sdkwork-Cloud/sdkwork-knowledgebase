//! Infrastructure routes and readiness probes for Knowledgebase HTTP surfaces.

use std::sync::Arc;

use axum::Router;
use sdkwork_intelligence_knowledgebase_repository_sqlx::knowledgebase_health_check;
use sdkwork_web_bootstrap::{
    infra_public_path_prefixes, mount_infra_routes, ReadinessCheck, ReadinessFuture,
    ServiceRouterConfig,
};
use sqlx::AnyPool;

pub const LIVEZ: &str = "/livez";
pub const READYZ: &str = "/readyz";
pub const HEALTHZ: &str = "/healthz";

/// Shared readiness probe used by every Knowledgebase HTTP surface. Hosts inject their complete
/// runtime dependency check here; the default database-only implementation remains useful for
/// isolated workers that do not own Drive dependencies.
pub type KnowledgebaseReadinessCheck = Arc<dyn ReadinessCheck>;

/// Database readiness probe for Knowledgebase `sqlx::AnyPool` connections.
#[derive(Clone)]
pub struct DbReadinessCheck {
    pool: AnyPool,
}

impl DbReadinessCheck {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

impl ReadinessCheck for DbReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let pool = self.pool.clone();
        Box::pin(async move {
            knowledgebase_health_check(&pool).await.map_err(|error| {
                sdkwork_knowledgebase_observability::set_readiness_status(false);
                error.to_string()
            })?;
            sdkwork_knowledgebase_observability::set_readiness_status(true);
            Ok(())
        })
    }
}

pub fn knowledgebase_service_router_config(
    readiness: Option<KnowledgebaseReadinessCheck>,
) -> ServiceRouterConfig {
    // The knowledgebase observability layer (`wrap_router_with_metrics`) mounts a
    // richer `/metrics` handler that includes OKF, audit, and billing metrics in
    // addition to the generic HTTP metrics. Skip the generic `/metrics` route here
    // so it does not overlap when `wrap_router_with_metrics` merges its own.
    let base = ServiceRouterConfig::default().skip_metrics();
    match readiness {
        Some(check) => base.with_readiness_check(check),
        None => base.with_always_ready(),
    }
}

pub fn mount_knowledgebase_infra_routes(router: Router, config: ServiceRouterConfig) -> Router {
    mount_infra_routes(router, config)
}

pub fn knowledgebase_infra_public_path_prefixes() -> Vec<String> {
    infra_public_path_prefixes()
}
