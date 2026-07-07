use axum::{
    extract::{Path, Query, State},
    response::Response,
    Json,
};
use sdkwork_knowledgebase_contract::{
    AnonymizeKnowledgeAuditSubjectRequest, CreateKnowledgeSourceRequest,
    ExportKnowledgeAuditEventsRequest, KnowledgeIndexRequest, KnowledgeOkfProfileRequest,
    KnowledgeRetrievalProfileRequest, OkfBundleExportRequest, OkfBundleImportRequest,
    OkfCandidateReviewRequest, OkfCompileJobRequest, OkfConceptPublishRequest,
    OkfIndexRebuildRequest, OkfLogEntry, OkfQualityRunRequest,
};
use serde::Deserialize;

use crate::{
    auth::{require_backend_context, require_backend_mutation_context, RequiredBackendContext},
    error::BackendApiProblem,
    response::{created_json, ok_json, ok_list_json},
    routes::BackendState,
};

macro_rules! backend_handler {
    ($name:ident, $body:expr) => {
        pub(crate) async fn $name(
            State(state): State<BackendState>,
            context: RequiredBackendContext,
        ) -> Result<Response, BackendApiProblem> {
            require_backend_context(&state, context)?;
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
    context: RequiredBackendContext,
    Json(request): Json<CreateKnowledgeSourceRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "sources.create")?;
    created_json(state.api.create_source(request).await)
}

pub(crate) async fn create_okf_compile_job(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.compileJobs.create")?;
    created_json(state.api.create_okf_compile_job(request).await)
}

pub(crate) async fn list_okf_candidates(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Query(query): Query<ListOkfCandidatesQuery>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(state.api.list_okf_candidates(query.space_id).await)
}

pub(crate) async fn approve_okf_candidate(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(candidate_id): Path<u64>,
    Json(request): Json<OkfCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.candidates.approve")?;
    ok_json(state.api.approve_okf_candidate(candidate_id, request).await)
}

pub(crate) async fn reject_okf_candidate(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(candidate_id): Path<u64>,
    Json(request): Json<OkfCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.candidates.reject")?;
    ok_json(state.api.reject_okf_candidate(candidate_id, request).await)
}

pub(crate) async fn publish_okf_concept(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(concept_id): Path<u64>,
    Json(request): Json<OkfConceptPublishRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.concepts.publish")?;
    ok_json(state.api.publish_okf_concept(concept_id, request).await)
}

pub(crate) async fn create_okf_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.profile.create")?;
    created_json(state.api.create_okf_profile(request).await)
}

pub(crate) async fn update_okf_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.profile.update")?;
    ok_json(state.api.update_okf_profile(profile_id, request).await)
}

pub(crate) async fn rebuild_okf_index(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.bundle.index.create")?;
    created_json(state.api.rebuild_okf_index(request).await)
}

pub(crate) async fn create_okf_log_entry(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfLogEntry>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.log.entries.create")?;
    created_json(state.api.create_okf_log_entry(request).await)
}

pub(crate) async fn create_okf_export(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfBundleExportRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.bundle.export.create")?;
    created_json(state.api.create_okf_export(request).await)
}

pub(crate) async fn create_okf_import(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfBundleImportRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.bundle.import.create")?;
    created_json(state.api.create_okf_import(request).await)
}

pub(crate) async fn retrieve_okf_export(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(export_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(state.api.retrieve_okf_export(export_id).await)
}

backend_handler!(list_okf_bundle_files, |state: BackendState| async move {
    ok_json(state.api.list_okf_bundle_files().await)
});

pub(crate) async fn create_okf_lint_run(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.lintRuns.create")?;
    created_json(state.api.create_okf_lint_run(request).await)
}

pub(crate) async fn create_okf_eval_run(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "okf.evalRuns.create")?;
    created_json(state.api.create_okf_eval_run(request).await)
}

backend_handler!(list_indexes, |state: BackendState| async move {
    ok_json(state.api.list_indexes().await)
});

pub(crate) async fn create_index(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<KnowledgeIndexRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "indexes.create")?;
    created_json(
        state
            .api
            .create_index(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

pub(crate) async fn retrieve_index(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(index_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(state.api.retrieve_index(index_id).await)
}

pub(crate) async fn rebuild_index(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(index_id): Path<u64>,
    Json(request): Json<OkfIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "indexes.rebuild")?;
    ok_json(state.api.rebuild_index(index_id, request).await)
}

pub(crate) async fn create_retrieval_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "retrievalProfiles.create")?;
    created_json(
        state
            .api
            .create_retrieval_profile(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

pub(crate) async fn retrieve_retrieval_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(profile_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(state.api.retrieve_retrieval_profile(profile_id).await)
}

pub(crate) async fn update_retrieval_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "retrievalProfiles.update")?;
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
    context: RequiredBackendContext,
    Path(trace_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(state.api.retrieve_retrieval_trace(trace_id).await)
}

backend_handler!(retrieve_provider_health, |state: BackendState| async move {
    ok_json(state.api.retrieve_provider_health().await)
});

// ============================================================================
// Tenant status handler
// ============================================================================

// Retrieves the caller's own tenant knowledgebase status.
//
// Security: The tenant is identified by the authenticated principal's token claims.
// Returns space count, document count, and status for the current tenant.
backend_handler!(retrieve_current_tenant, |state: BackendState| async move {
    ok_json(state.api.retrieve_current_tenant().await)
});

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListSpaceMembersQuery {
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListSpacesQuery {
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<u32>,
}

pub(crate) async fn list_spaces(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Query(query): Query<ListSpacesQuery>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_list_json(state.api.list_spaces(query.cursor, query.page_size).await)
}

pub(crate) async fn list_space_members(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(space_id): Path<u64>,
    Query(query): Query<ListSpaceMembersQuery>,
) -> Result<Response, BackendApiProblem> {
    require_backend_context(&state, context)?;
    ok_json(
        state
            .api
            .list_space_members(space_id, query.cursor, query.page_size)
            .await,
    )
}

pub(crate) async fn export_audit_events(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<ExportKnowledgeAuditEventsRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(&state, context, "compliance.auditEvents.export.create")?;
    created_json(state.api.export_audit_events(request).await)
}

pub(crate) async fn anonymize_audit_subject(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<AnonymizeKnowledgeAuditSubjectRequest>,
) -> Result<Response, BackendApiProblem> {
    require_backend_mutation_context(
        &state,
        context,
        "compliance.auditEvents.anonymizeActor.create",
    )?;
    created_json(state.api.anonymize_audit_subject(request).await)
}
