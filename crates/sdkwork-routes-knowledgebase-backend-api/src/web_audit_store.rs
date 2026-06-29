//! Framework HTTP audit persistence for Knowledgebase web surfaces.

use std::sync::Arc;

use sdkwork_knowledgebase_observability::environment::is_production_like_environment;
use sdkwork_web_axum::WebFrameworkLayer;
use sdkwork_web_core::{AuditEmitter, WebRequestContextResolver};
use sdkwork_web_store_sqlx::{
    connect_and_bootstrap_webstore_database_from_env, shared_audit_emitter, shared_audit_emitter_pg,
};
use tokio::sync::OnceCell;

static SHARED_AUDIT_EMITTER: OnceCell<Option<Arc<dyn AuditEmitter>>> = OnceCell::const_new();

async fn build_knowledgebase_audit_emitter() -> Option<Arc<dyn AuditEmitter>> {
    let host = connect_and_bootstrap_webstore_database_from_env()
        .await
        .ok()?;
    if let Some(sqlite) = host.pool().as_sqlite().cloned() {
        return Some(shared_audit_emitter(sqlite));
    }
    if let Some(postgres) = host.pool().as_postgres().cloned() {
        return Some(shared_audit_emitter_pg(postgres));
    }
    None
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
                "Web audit emitter is required for production-like environments; configure SDKWORK_WEB_STORE_DATABASE_URL and bootstrap web_audit_event migrations"
            );
            std::process::exit(1);
        }
        None => layer,
    }
}
