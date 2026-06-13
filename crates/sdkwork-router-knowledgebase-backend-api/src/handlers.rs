use axum::extract::{Path, State};
use axum::response::Response;
use axum::Json;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, KnowledgeIndexRequest, KnowledgeRetrievalProfileRequest,
    KnowledgeWikiSchemaProfileRequest, WikiCandidateReviewRequest, WikiCompileJobRequest,
    WikiExportRequest, WikiIndexRebuildRequest, WikiLogEntry, WikiPagePublishRequest,
    WikiQualityRunRequest,
};

use crate::error::BackendApiProblem;
use crate::response::{created_json, ok_json};
use crate::routes::BackendState;

pub(crate) async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub(crate) async fn list_sources(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_sources().await)
}

pub(crate) async fn create_source(
    State(state): State<BackendState>,
    Json(request): Json<CreateKnowledgeSourceRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_source(request).await)
}

pub(crate) async fn create_wiki_compile_job(
    State(state): State<BackendState>,
    Json(request): Json<WikiCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_compile_job(request).await)
}

pub(crate) async fn list_wiki_candidates(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_wiki_candidates().await)
}

pub(crate) async fn approve_wiki_candidate(
    State(state): State<BackendState>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(
        state
            .api
            .approve_wiki_candidate(candidate_id, request)
            .await,
    )
}

pub(crate) async fn reject_wiki_candidate(
    State(state): State<BackendState>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.reject_wiki_candidate(candidate_id, request).await)
}

pub(crate) async fn publish_wiki_page(
    State(state): State<BackendState>,
    Path(page_id): Path<u64>,
    Json(request): Json<WikiPagePublishRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.publish_wiki_page(page_id, request).await)
}

pub(crate) async fn create_wiki_schema_profile(
    State(state): State<BackendState>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_schema_profile(request).await)
}

pub(crate) async fn update_wiki_schema_profile(
    State(state): State<BackendState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(
        state
            .api
            .update_wiki_schema_profile(profile_id, request)
            .await,
    )
}

pub(crate) async fn rebuild_wiki_index(
    State(state): State<BackendState>,
    Json(request): Json<WikiIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.rebuild_wiki_index(request).await)
}

pub(crate) async fn create_wiki_log_entry(
    State(state): State<BackendState>,
    Json(request): Json<WikiLogEntry>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_log_entry(request).await)
}

pub(crate) async fn create_wiki_export(
    State(state): State<BackendState>,
    Json(request): Json<WikiExportRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_export(request).await)
}

pub(crate) async fn retrieve_wiki_export(
    State(state): State<BackendState>,
    Path(export_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_wiki_export(export_id).await)
}

pub(crate) async fn list_wiki_file_entries(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_wiki_file_entries().await)
}

pub(crate) async fn create_wiki_lint_run(
    State(state): State<BackendState>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_lint_run(request).await)
}

pub(crate) async fn create_wiki_eval_run(
    State(state): State<BackendState>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_eval_run(request).await)
}

pub(crate) async fn create_index(
    State(state): State<BackendState>,
    Json(request): Json<KnowledgeIndexRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_index(request).await)
}

pub(crate) async fn retrieve_index(
    State(state): State<BackendState>,
    Path(index_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_index(index_id).await)
}

pub(crate) async fn rebuild_index(
    State(state): State<BackendState>,
    Path(index_id): Path<u64>,
    Json(request): Json<WikiIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.rebuild_index(index_id, request).await)
}

pub(crate) async fn create_retrieval_profile(
    State(state): State<BackendState>,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_retrieval_profile(request).await)
}

pub(crate) async fn retrieve_retrieval_profile(
    State(state): State<BackendState>,
    Path(profile_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_retrieval_profile(profile_id).await)
}

pub(crate) async fn update_retrieval_profile(
    State(state): State<BackendState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(
        state
            .api
            .update_retrieval_profile(profile_id, request)
            .await,
    )
}

pub(crate) async fn list_retrieval_traces(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_retrieval_traces().await)
}

pub(crate) async fn retrieve_retrieval_trace(
    State(state): State<BackendState>,
    Path(trace_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_retrieval_trace(trace_id).await)
}

pub(crate) async fn retrieve_provider_health(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_provider_health().await)
}
