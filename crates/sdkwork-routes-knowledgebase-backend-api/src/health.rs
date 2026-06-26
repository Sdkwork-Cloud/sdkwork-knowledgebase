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
    readiness: Option<DbReadinessCheck>,
) -> ServiceRouterConfig {
    match readiness {
        Some(check) => ServiceRouterConfig::default().with_readiness_check(Arc::new(check)),
        None => ServiceRouterConfig::default().with_always_ready(),
    }
}

pub fn mount_knowledgebase_infra_routes(router: Router, config: ServiceRouterConfig) -> Router {
    mount_infra_routes(router, config)
}

pub fn knowledgebase_infra_public_path_prefixes() -> Vec<String> {
    infra_public_path_prefixes()
}
