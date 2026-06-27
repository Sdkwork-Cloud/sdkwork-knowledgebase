use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;

use crate::{handlers, health, paths, ports::KnowledgeBackendApi, DbReadinessCheck};

#[derive(Clone)]
pub struct BackendState {
    pub(crate) api: Arc<dyn KnowledgeBackendApi>,
    pub(crate) runtime_tenant_id: u64,
}

pub fn build_router_with_backend_api<A>(api: A, runtime_tenant_id: u64) -> Router
where
    A: KnowledgeBackendApi,
{
    build_router_with_shared_backend_api(Arc::new(api), runtime_tenant_id)
}

pub fn build_router_with_shared_backend_api(
    api: Arc<dyn KnowledgeBackendApi>,
    runtime_tenant_id: u64,
) -> Router {
    build_router_with_shared_backend_api_and_readiness(api, runtime_tenant_id, None)
}

pub fn build_router_with_shared_backend_api_and_readiness(
    api: Arc<dyn KnowledgeBackendApi>,
    runtime_tenant_id: u64,
    readiness: Option<DbReadinessCheck>,
) -> Router {
    health::mount_knowledgebase_infra_routes(
        build_business_router_with_shared_backend_api(api, runtime_tenant_id),
        health::knowledgebase_service_router_config(readiness),
    )
}

pub fn build_business_router_with_shared_backend_api(
    api: Arc<dyn KnowledgeBackendApi>,
    runtime_tenant_id: u64,
) -> Router {
    let state = BackendState {
        api,
        runtime_tenant_id,
    };
    Router::new()
        .route(
            paths::SOURCES,
            get(handlers::list_sources).post(handlers::create_source),
        )
        .route(
            paths::OKF_COMPILE_JOBS,
            post(handlers::create_okf_compile_job),
        )
        .route(paths::OKF_CANDIDATES, get(handlers::list_okf_candidates))
        .route(
            paths::OKF_CANDIDATE_APPROVE,
            post(handlers::approve_okf_candidate),
        )
        .route(
            paths::OKF_CANDIDATE_REJECT,
            post(handlers::reject_okf_candidate),
        )
        .route(
            paths::OKF_CONCEPT_PUBLISH,
            post(handlers::publish_okf_concept),
        )
        .route(paths::OKF_PROFILES, post(handlers::create_okf_profile))
        .route(paths::OKF_PROFILE, patch(handlers::update_okf_profile))
        .route(paths::OKF_INDEX_REBUILD, post(handlers::rebuild_okf_index))
        .route(paths::OKF_LOG_ENTRIES, post(handlers::create_okf_log_entry))
        .route(paths::OKF_EXPORTS, post(handlers::create_okf_export))
        .route(paths::OKF_EXPORT, get(handlers::retrieve_okf_export))
        .route(paths::OKF_IMPORTS, post(handlers::create_okf_import))
        .route(
            paths::OKF_BUNDLE_FILES,
            get(handlers::list_okf_bundle_files),
        )
        .route(paths::OKF_LINT_RUNS, post(handlers::create_okf_lint_run))
        .route(paths::OKF_EVAL_RUNS, post(handlers::create_okf_eval_run))
        .route(paths::INDEXES, post(handlers::create_index))
        .route(paths::INDEX, get(handlers::retrieve_index))
        .route(paths::INDEX_REBUILD, post(handlers::rebuild_index))
        .route(
            paths::RETRIEVAL_PROFILES,
            post(handlers::create_retrieval_profile),
        )
        .route(
            paths::RETRIEVAL_PROFILE,
            get(handlers::retrieve_retrieval_profile).patch(handlers::update_retrieval_profile),
        )
        .route(
            paths::RETRIEVAL_TRACES,
            get(handlers::list_retrieval_traces),
        )
        .route(
            paths::RETRIEVAL_TRACE,
            get(handlers::retrieve_retrieval_trace),
        )
        .route(
            paths::PROVIDER_HEALTH,
            get(handlers::retrieve_provider_health),
        )
        .with_state(state)
}

pub fn gateway_mount_business(
    api: Arc<dyn KnowledgeBackendApi>,
    runtime_tenant_id: u64,
) -> Router {
    build_business_router_with_shared_backend_api(api, runtime_tenant_id)
}
