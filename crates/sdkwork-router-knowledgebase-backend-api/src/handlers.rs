use axum::{
    extract::{Path, State},
    response::Response,
    Extension, Json,
};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, KnowledgeIndexRequest, KnowledgeRetrievalProfileRequest,
    KnowledgeWikiSchemaProfileRequest, WikiCandidateReviewRequest, WikiCompileJobRequest,
    WikiExportRequest, WikiIndexRebuildRequest, WikiLogEntry, WikiPagePublishRequest,
    WikiQualityRunRequest,
};

use crate::{
    auth::require_backend_context,
    error::BackendApiProblem,
    ports::KnowledgeBackendRequestContext,
    response::{created_json, ok_json},
    routes::BackendState,
};

pub(crate) async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

macro_rules! backend_handler {
    ($name:ident, $body:expr) => {
        pub(crate) async fn $name(
            State(state): State<BackendState>,
            context: Option<Extension<KnowledgeBackendRequestContext>>,
        ) -> Result<Response, BackendApiProblem> {
            require_backend_context(context)?;
            $body(state).await
        }
    };
}

backend_handler!(list_sources, |state: BackendState| async move {
    ok_json(state.api.list_sources().await)
});

pub(crate) async fn create_source(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<CreateKnowledgeSourceRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_source(request).await)
}

pub(crate) async fn create_wiki_compile_job(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_compile_job(request).await)
}

backend_handler!(list_wiki_candidates, |state: BackendState| async move {
    ok_json(state.api.list_wiki_candidates().await)
});

pub(crate) async fn approve_wiki_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(
        state
            .api
            .approve_wiki_candidate(candidate_id, request)
            .await,
    )
}

pub(crate) async fn reject_wiki_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.reject_wiki_candidate(candidate_id, request).await)
}

pub(crate) async fn publish_wiki_page(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(page_id): Path<u64>,
    Json(request): Json<WikiPagePublishRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.publish_wiki_page(page_id, request).await)
}

pub(crate) async fn create_wiki_schema_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_schema_profile(request).await)
}

pub(crate) async fn update_wiki_schema_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_wiki_schema_profile(profile_id, request)
            .await,
    )
}

pub(crate) async fn rebuild_wiki_index(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.rebuild_wiki_index(request).await)
}

pub(crate) async fn create_wiki_log_entry(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiLogEntry>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_log_entry(request).await)
}

pub(crate) async fn create_wiki_export(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiExportRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_export(request).await)
}

pub(crate) async fn retrieve_wiki_export(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(export_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.retrieve_wiki_export(export_id).await)
}

backend_handler!(list_wiki_file_entries, |state: BackendState| async move {
    ok_json(state.api.list_wiki_file_entries().await)
});

pub(crate) async fn create_wiki_lint_run(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_lint_run(request).await)
}

pub(crate) async fn create_wiki_eval_run(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_wiki_eval_run(request).await)
}

pub(crate) async fn create_index(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<KnowledgeIndexRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(
        state
            .api
            .create_index(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

pub(crate) async fn retrieve_index(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(index_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.retrieve_index(index_id).await)
}

pub(crate) async fn rebuild_index(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(index_id): Path<u64>,
    Json(request): Json<WikiIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.rebuild_index(index_id, request).await)
}

pub(crate) async fn create_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    created_json(
        state
            .api
            .create_retrieval_profile(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

pub(crate) async fn retrieve_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.retrieve_retrieval_profile(profile_id).await)
}

pub(crate) async fn update_retrieval_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_context(context)?;
    ok_json(
        state
            .api
            .update_retrieval_profile(profile_id, request.with_tenant_id(context.tenant_id))
            .await,
    )
}

backend_handler!(list_retrieval_traces, |state: BackendState| async move {
    ok_json(state.api.list_retrieval_traces().await)
});

pub(crate) async fn retrieve_retrieval_trace(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(trace_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.retrieve_retrieval_trace(trace_id).await)
}

backend_handler!(retrieve_provider_health, |state: BackendState| async move {
    ok_json(state.api.retrieve_provider_health().await)
});
