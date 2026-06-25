use axum::Router;
use sdkwork_utils_rust::is_blank;

pub use sdkwork_knowledgebase_observability::{
    is_development_environment, is_production_like_environment, knowledgebase_environment,
};

use crate::{dev_auth, KnowledgebaseRuntime};

pub fn dev_auth_bypass_enabled() -> bool {
    let bypass_flag = std::env::var("SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if bypass_flag {
        if !is_development_environment() {
            eprintln!(
                "SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS requires SDKWORK_KNOWLEDGEBASE_ENVIRONMENT=development"
            );
            std::process::exit(1);
        }
        return true;
    }

    false
}

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
    validate_dev_auth_policy();
    validate_snowflake_node_id_for_production();
    validate_secrets_encryption_for_production();
    validate_postgres_for_production();

    if dev_auth_bypass_enabled() {
        return;
    }

    let organization_id = std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if organization_id == 0 {
        eprintln!(
            "SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID must be set when SDKWORK_KNOWLEDGEBASE_ENVIRONMENT is not development"
        );
        std::process::exit(1);
    }
}

fn validate_dev_auth_policy() {
    let bypass_flag_set = std::env::var("SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if bypass_flag_set && is_production_like_environment() {
        eprintln!(
            "SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS is forbidden for production-like environments"
        );
        std::process::exit(1);
    }

    if bypass_flag_set && !is_development_environment() {
        eprintln!(
            "SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS requires SDKWORK_KNOWLEDGEBASE_ENVIRONMENT=development"
        );
        std::process::exit(1);
    }
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
    tenant_id: u64,
    actor_id: Option<u64>,
    operator_id: Option<u64>,
) -> Router {
    let app_router = build_served_app_router(runtime, tenant_id, actor_id).await;
    let backend_router = build_served_backend_router(runtime, tenant_id, operator_id).await;
    let open_router = build_served_open_router(runtime, tenant_id, actor_id).await;
    app_router.merge(backend_router).merge(open_router)
}

pub async fn build_served_app_router(
    runtime: &KnowledgebaseRuntime,
    tenant_id: u64,
    actor_id: Option<u64>,
) -> Router {
    let router = runtime.build_full_app_router_with_web_framework().await;
    if dev_auth_bypass_enabled() {
        dev_auth::with_dev_app_auth(router, tenant_id, actor_id)
    } else {
        router
    }
}

pub async fn build_served_backend_router(
    runtime: &KnowledgebaseRuntime,
    tenant_id: u64,
    operator_id: Option<u64>,
) -> Router {
    let router = runtime.build_backend_router_with_web_framework().await;
    if dev_auth_bypass_enabled() {
        dev_auth::with_dev_backend_auth(router, tenant_id, operator_id)
    } else {
        router
    }
}

pub async fn build_served_open_router(
    runtime: &KnowledgebaseRuntime,
    tenant_id: u64,
    actor_id: Option<u64>,
) -> Router {
    let router = runtime.build_open_api_router_with_web_framework().await;
    if dev_auth_bypass_enabled() {
        dev_auth::with_dev_open_auth(router, tenant_id, actor_id)
    } else {
        router
    }
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
    fn dev_auth_bypass_disabled_when_bypass_flag_unset() {
        if std::env::var("SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS").is_ok() {
            return;
        }
        assert!(!dev_auth_bypass_enabled());
    }

    #[test]
    fn production_like_environment_is_not_development_by_default() {
        if std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").is_ok() {
            return;
        }
        assert!(!is_development_environment());
    }
}
