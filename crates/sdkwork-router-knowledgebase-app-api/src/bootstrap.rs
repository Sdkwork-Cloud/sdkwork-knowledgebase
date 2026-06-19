use axum::Router;

use crate::{dev_auth, KnowledgebaseRuntime};

pub fn dev_auth_bypass_enabled() -> bool {
    knowledgebase_environment()
        .map(|environment| environment.eq_ignore_ascii_case("development"))
        .unwrap_or(false)
        || std::env::var("SDKWORK_KNOWLEDGEBASE_DEV_AUTH_BYPASS")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
}

pub fn knowledgebase_environment() -> Option<String> {
    std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").ok()
}

pub fn validate_process_config() {
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
