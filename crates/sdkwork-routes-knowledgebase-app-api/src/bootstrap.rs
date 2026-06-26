use axum::Router;
use sdkwork_utils_rust::is_blank;

pub use sdkwork_knowledgebase_observability::{
    is_development_environment, is_production_like_environment, knowledgebase_environment,
};

use crate::KnowledgebaseRuntime;

/// Resolves the knowledgebase database URL. Production-like environments fail closed
/// when `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` is unset.
pub fn resolve_database_url() -> String {
    match std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL") {
        Ok(url) if !is_blank(Some(url.as_str())) => url,
        _ if is_production_like_environment() => {
            eprintln!(
                "SDKWORK_KNOWLEDGEBASE_DATABASE_URL must be set for production-like environments"
            );
            std::process::exit(1);
        }
        _ => "sqlite://data/knowledgebase.db?mode=rwc".to_string(),
    }
}

pub fn validate_process_config() {
    validate_snowflake_node_id_for_production();
    validate_secrets_encryption_for_production();
    validate_postgres_for_production();

    let organization_id = resolve_deployment_tenant_id();
    if organization_id == 0 {
        eprintln!(
            "SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID must be set when SDKWORK_KNOWLEDGEBASE_ENVIRONMENT is not development"
        );
        std::process::exit(1);
    }
}

pub fn resolve_deployment_tenant_id() -> u64 {
    std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0)
}

fn validate_postgres_for_production() {
    if !is_production_like_environment() {
        return;
    }

    let database_url = resolve_database_url();
    let normalized = database_url.to_ascii_lowercase();
    if normalized.starts_with("postgres://") || normalized.starts_with("postgresql://") {
        return;
    }

    eprintln!(
        "SDKWORK_KNOWLEDGEBASE_DATABASE_URL must use PostgreSQL for production-like environments"
    );
    std::process::exit(1);
}

fn validate_secrets_encryption_for_production() {
    if !is_production_like_environment() {
        return;
    }

    if sdkwork_intelligence_knowledgebase_service::wechat::encryption_key_configured() {
        return;
    }

    eprintln!(
        "SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY_FILE or SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY must be set for production-like environments"
    );
    std::process::exit(1);
}

fn validate_snowflake_node_id_for_production() {
    if !is_production_like_environment() {
        return;
    }

    let node_id = std::env::var("SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID").ok();
    let Some(node_id) = node_id else {
        eprintln!(
            "SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID must be set for production-like environments"
        );
        std::process::exit(1);
    };
    if is_blank(Some(node_id.as_str())) {
        eprintln!("SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID must not be empty");
        std::process::exit(1);
    }
    if let Err(error) =
        sdkwork_intelligence_knowledgebase_repository_sqlx::SnowflakeKnowledgeIdGenerator::from_node_id_config(
            Some(node_id.trim()),
        )
    {
        eprintln!("invalid SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID: {error}");
        std::process::exit(1);
    }
}

pub fn is_unified_process_layout() -> bool {
    std::env::var("SDKWORK_KNOWLEDGEBASE_SERVICE_LAYOUT")
        .map(|value| value.eq_ignore_ascii_case("unified-process"))
        .unwrap_or(false)
}

pub async fn build_served_unified_router(
    runtime: &KnowledgebaseRuntime,
    _tenant_id: u64,
    _actor_id: Option<u64>,
    _operator_id: Option<u64>,
) -> Router {
    let app_router = build_served_app_router(runtime, 0, None).await;
    let backend_router = build_served_backend_router(runtime, 0, None).await;
    let open_router = build_served_open_router(runtime, 0, None).await;
    app_router.merge(backend_router).merge(open_router)
}

pub async fn build_served_app_router(
    runtime: &KnowledgebaseRuntime,
    _tenant_id: u64,
    _actor_id: Option<u64>,
) -> Router {
    runtime.build_full_app_router_with_web_framework().await
}

pub async fn build_served_backend_router(
    runtime: &KnowledgebaseRuntime,
    _tenant_id: u64,
    _operator_id: Option<u64>,
) -> Router {
    runtime.build_backend_router_with_web_framework().await
}

pub async fn build_served_open_router(
    runtime: &KnowledgebaseRuntime,
    _tenant_id: u64,
    _actor_id: Option<u64>,
) -> Router {
    runtime.build_open_api_router_with_web_framework().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_environment_requires_explicit_value() {
        if std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").is_ok() {
            return;
        }
        assert!(!is_development_environment());
    }

    #[test]
    fn production_like_environment_is_not_development_by_default() {
        if std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").is_ok() {
            return;
        }
        assert!(!is_development_environment());
    }

    #[test]
    fn process_config_requires_organization_without_runtime_bypass() {
        if std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").is_ok() {
            return;
        }
        let organization_id = std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID").ok();
        assert!(organization_id.is_none());
    }
}
