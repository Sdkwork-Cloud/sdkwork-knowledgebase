use axum::{
    extract::{Path, Query, State},
    response::Response,
    Extension, Json,
};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, KnowledgeIndexRequest, KnowledgeOkfProfileRequest,
    KnowledgeRetrievalProfileRequest, OkfBundleExportRequest, OkfBundleImportRequest,
    OkfCandidateReviewRequest, OkfCompileJobRequest, OkfConceptPublishRequest,
    OkfIndexRebuildRequest, OkfLogEntry, OkfQualityRunRequest,
};
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListOkfCandidatesQuery {
    pub space_id: u64,
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

pub(crate) async fn create_okf_compile_job(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_compile_job(request).await)
}

pub(crate) async fn list_okf_candidates(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Query(query): Query<ListOkfCandidatesQuery>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.list_okf_candidates(query.space_id).await)
}

pub(crate) async fn approve_okf_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<OkfCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.approve_okf_candidate(candidate_id, request).await)
}

pub(crate) async fn reject_okf_candidate(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<OkfCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.reject_okf_candidate(candidate_id, request).await)
}

pub(crate) async fn publish_okf_concept(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(concept_id): Path<u64>,
    Json(request): Json<OkfConceptPublishRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.publish_okf_concept(concept_id, request).await)
}

pub(crate) async fn create_okf_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_profile(request).await)
}

pub(crate) async fn update_okf_profile(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.update_okf_profile(profile_id, request).await)
}

pub(crate) async fn rebuild_okf_index(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.rebuild_okf_index(request).await)
}

pub(crate) async fn create_okf_log_entry(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfLogEntry>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_log_entry(request).await)
}

pub(crate) async fn create_okf_export(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfBundleExportRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_export(request).await)
}

pub(crate) async fn create_okf_import(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfBundleImportRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_import(request).await)
}

pub(crate) async fn retrieve_okf_export(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Path(export_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    ok_json(state.api.retrieve_okf_export(export_id).await)
}

backend_handler!(list_okf_bundle_files, |state: BackendState| async move {
    ok_json(state.api.list_okf_bundle_files().await)
});

pub(crate) async fn create_okf_lint_run(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_lint_run(request).await)
}

pub(crate) async fn create_okf_eval_run(
    State(state): State<BackendState>,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(context)?;
    created_json(state.api.create_okf_eval_run(request).await)
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
    Json(request): Json<OkfIndexRebuildRequest>,
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
