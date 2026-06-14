use axum::routing::{get, patch, post};
use axum::Router;
use std::sync::Arc;

use crate::handlers;
use crate::paths;
use crate::ports::KnowledgeBackendApi;

#[derive(Clone)]
pub(crate) struct BackendState {
    pub(crate) api: Arc<dyn KnowledgeBackendApi>,
}

pub fn build_router_with_backend_api<A>(api: A) -> Router
where
    A: KnowledgeBackendApi,
{
    build_router_with_shared_backend_api(Arc::new(api))
}

pub fn build_router_with_shared_backend_api(api: Arc<dyn KnowledgeBackendApi>) -> Router {
    Router::new()
        .route(paths::HEALTHZ, get(handlers::health))
        .route(
            paths::SOURCES,
            get(handlers::list_sources).post(handlers::create_source),
        )
        .route(
            paths::WIKI_COMPILE_JOBS,
            post(handlers::create_wiki_compile_job),
        )
        .route(paths::WIKI_CANDIDATES, get(handlers::list_wiki_candidates))
        .route(
            paths::WIKI_CANDIDATE_APPROVE,
            post(handlers::approve_wiki_candidate),
        )
        .route(
            paths::WIKI_CANDIDATE_REJECT,
            post(handlers::reject_wiki_candidate),
        )
        .route(paths::WIKI_PAGE_PUBLISH, post(handlers::publish_wiki_page))
        .route(
            paths::WIKI_SCHEMA_PROFILES,
            post(handlers::create_wiki_schema_profile),
        )
        .route(
            paths::WIKI_SCHEMA_PROFILE,
            patch(handlers::update_wiki_schema_profile),
        )
        .route(
            paths::WIKI_INDEX_REBUILD,
            post(handlers::rebuild_wiki_index),
        )
        .route(
            paths::WIKI_LOG_ENTRIES,
            post(handlers::create_wiki_log_entry),
        )
        .route(paths::WIKI_EXPORTS, post(handlers::create_wiki_export))
        .route(paths::WIKI_EXPORT, get(handlers::retrieve_wiki_export))
        .route(
            paths::WIKI_FILE_ENTRIES,
            get(handlers::list_wiki_file_entries),
        )
        .route(paths::WIKI_LINT_RUNS, post(handlers::create_wiki_lint_run))
        .route(paths::WIKI_EVAL_RUNS, post(handlers::create_wiki_eval_run))
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
        .with_state(BackendState { api })
}
