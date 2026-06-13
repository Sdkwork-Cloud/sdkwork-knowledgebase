use axum::routing::{get, patch, post};
use axum::Router;
use std::sync::Arc;

use crate::handlers;
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
        .route("/healthz", get(handlers::health))
        .route(
            "/backend/v3/api/knowledge/sources",
            get(handlers::list_sources).post(handlers::create_source),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_compile_jobs",
            post(handlers::create_wiki_compile_job),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates",
            get(handlers::list_wiki_candidates),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/approve",
            post(handlers::approve_wiki_candidate),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/reject",
            post(handlers::reject_wiki_candidate),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_pages/:page_id/publish",
            post(handlers::publish_wiki_page),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_schema_profiles",
            post(handlers::create_wiki_schema_profile),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_schema_profiles/:profile_id",
            patch(handlers::update_wiki_schema_profile),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_index/rebuild",
            post(handlers::rebuild_wiki_index),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_log_entries",
            post(handlers::create_wiki_log_entry),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_exports",
            post(handlers::create_wiki_export),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_exports/:export_id",
            get(handlers::retrieve_wiki_export),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_file_entries",
            get(handlers::list_wiki_file_entries),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_lint_runs",
            post(handlers::create_wiki_lint_run),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_eval_runs",
            post(handlers::create_wiki_eval_run),
        )
        .route(
            "/backend/v3/api/knowledge/indexes",
            post(handlers::create_index),
        )
        .route(
            "/backend/v3/api/knowledge/indexes/:index_id",
            get(handlers::retrieve_index),
        )
        .route(
            "/backend/v3/api/knowledge/indexes/:index_id/rebuild",
            post(handlers::rebuild_index),
        )
        .route(
            "/backend/v3/api/knowledge/retrieval_profiles",
            post(handlers::create_retrieval_profile),
        )
        .route(
            "/backend/v3/api/knowledge/retrieval_profiles/:profile_id",
            get(handlers::retrieve_retrieval_profile).patch(handlers::update_retrieval_profile),
        )
        .route(
            "/backend/v3/api/knowledge/retrieval_traces",
            get(handlers::list_retrieval_traces),
        )
        .route(
            "/backend/v3/api/knowledge/retrieval_traces/:trace_id",
            get(handlers::retrieve_retrieval_trace),
        )
        .route(
            "/backend/v3/api/knowledge/provider_health",
            get(handlers::retrieve_provider_health),
        )
        .with_state(BackendState { api })
}
