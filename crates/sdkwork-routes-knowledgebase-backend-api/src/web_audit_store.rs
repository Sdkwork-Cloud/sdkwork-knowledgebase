//! Framework HTTP audit persistence for Knowledgebase web surfaces.

use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_repository_sqlx::connect_postgres_pool;
use sdkwork_knowledgebase_observability::environment::is_production_like_environment;
use sdkwork_utils_rust::is_blank;
use sdkwork_web_axum::WebFrameworkLayer;
use sdkwork_web_core::{AuditEmitter, AuditFact, WebFrameworkError, WebRequestContextResolver};
use sdkwork_web_store_sqlx::{
    connect_and_bootstrap_webstore_database_from_env, shared_audit_emitter,
};
use sqlx::postgres::PgPool;
use tokio::sync::OnceCell;

static SHARED_AUDIT_EMITTER: OnceCell<Option<Arc<dyn AuditEmitter>>> = OnceCell::const_new();

struct PostgresWebAuditEmitter {
    pool: PgPool,
}

impl PostgresWebAuditEmitter {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[async_trait]
impl AuditEmitter for PostgresWebAuditEmitter {
    async fn emit(&self, fact: AuditFact) -> Result<(), WebFrameworkError> {
        sqlx::query(
            "INSERT INTO web_audit_event \
             (request_id, tenant_id, user_id, api_surface, path, method, operation_id, status_code, duration_ms, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&fact.request_id)
        .bind(&fact.tenant_id)
        .bind(&fact.user_id)
        .bind(format!("{:?}", fact.api_surface))
        .bind(&fact.path)
        .bind(&fact.method)
        .bind(&fact.operation_id)
        .bind(fact.status_code.map(i64::from))
        .bind(fact.duration_ms.map(|value| value as i64))
        .bind(now_epoch_secs())
        .execute(&self.pool)
        .await
        .map_err(|error| {
            WebFrameworkError::dependency_unavailable(format!("postgres web audit store error: {error}"))
        })?;
        Ok(())
    }
}

fn knowledgebase_database_url() -> Option<String> {
    for key in ["SDKWORK_KNOWLEDGEBASE_DATABASE_URL", "DATABASE_URL"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !is_blank(Some(trimmed)) && trimmed.starts_with("postgres") {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

async fn build_knowledgebase_audit_emitter() -> Option<Arc<dyn AuditEmitter>> {
    if let Ok(host) = connect_and_bootstrap_webstore_database_from_env().await {
        if let Some(sqlite) = host.pool().as_sqlite().cloned() {
            return Some(shared_audit_emitter(sqlite));
        }
    }

    let database_url = knowledgebase_database_url()?;
    let pool = connect_postgres_pool(&database_url).await.ok()?;
    Some(Arc::new(PostgresWebAuditEmitter::new(pool)))
}

pub async fn shared_knowledgebase_audit_emitter() -> Option<Arc<dyn AuditEmitter>> {
    SHARED_AUDIT_EMITTER
        .get_or_init(|| async { build_knowledgebase_audit_emitter().await })
        .await
        .clone()
}

pub async fn attach_knowledgebase_audit_emitter<R>(
    layer: WebFrameworkLayer<R>,
) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    match shared_knowledgebase_audit_emitter().await {
        Some(emitter) => layer.with_audit_emitter(emitter),
        None if is_production_like_environment() => {
            eprintln!(
                "Web audit emitter is required for production-like environments; configure WEB_STORE sqlite or SDKWORK_KNOWLEDGEBASE_DATABASE_URL with web_audit_event migration applied"
            );
            std::process::exit(1);
        }
        None => layer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_web_audit_emitter_is_constructible() {
        let _ = PostgresWebAuditEmitter::new;
    }
}
