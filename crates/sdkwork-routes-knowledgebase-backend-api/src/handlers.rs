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
    error::{BackendApiProblem, BackendApiResult},
    ports::KnowledgeBackendRequestContext,
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

async fn audit_backend_mutation<T>(
    context: &KnowledgeBackendRequestContext,
    operation: &str,
    result: BackendApiResult<T>,
) -> Result<BackendApiResult<T>, BackendApiProblem> {
    if result.is_ok() {
        sdkwork_knowledgebase_observability::record_backend_admin_operation(
            operation,
            context.tenant_id,
            context.operator_id.unwrap_or(0),
        )
        .await
        .map_err(|error| {
            BackendApiProblem::from_internal(
                "knowledge_audit_persistence_failed",
                error.to_string(),
            )
        })?;
    }
    Ok(result)
}

backend_handler!(list_sources, |state: BackendState| async move {
    ok_json(state.api.list_sources().await)
});

pub(crate) async fn create_source(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<CreateKnowledgeSourceRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "sources.create")?;
    let result = state.api.create_source(request).await;
    created_json(audit_backend_mutation(&context, "sources.create", result).await?)
}

pub(crate) async fn create_okf_compile_job(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.compileJobs.create")?;
    let result = state.api.create_okf_compile_job(request).await;
    created_json(audit_backend_mutation(&context, "okf.compileJobs.create", result).await?)
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
    let context = require_backend_mutation_context(&state, context, "okf.candidates.approve")?;
    let result = state.api.approve_okf_candidate(candidate_id, request).await;
    ok_json(audit_backend_mutation(&context, "okf.candidates.approve", result).await?)
}

pub(crate) async fn reject_okf_candidate(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(candidate_id): Path<u64>,
    Json(request): Json<OkfCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.candidates.reject")?;
    let result = state.api.reject_okf_candidate(candidate_id, request).await;
    ok_json(audit_backend_mutation(&context, "okf.candidates.reject", result).await?)
}

pub(crate) async fn publish_okf_concept(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(concept_id): Path<u64>,
    Json(request): Json<OkfConceptPublishRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.concepts.publish")?;
    let result = state.api.publish_okf_concept(concept_id, request).await;
    ok_json(audit_backend_mutation(&context, "okf.concepts.publish", result).await?)
}

pub(crate) async fn create_okf_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.profile.create")?;
    let result = state.api.create_okf_profile(request).await;
    created_json(audit_backend_mutation(&context, "okf.profile.create", result).await?)
}

pub(crate) async fn update_okf_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeOkfProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.profile.update")?;
    let result = state.api.update_okf_profile(profile_id, request).await;
    ok_json(audit_backend_mutation(&context, "okf.profile.update", result).await?)
}

pub(crate) async fn rebuild_okf_index(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.bundle.index.create")?;
    let result = state.api.rebuild_okf_index(request).await;
    created_json(audit_backend_mutation(&context, "okf.bundle.index.create", result).await?)
}

pub(crate) async fn create_okf_log_entry(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfLogEntry>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.log.entries.create")?;
    let result = state.api.create_okf_log_entry(request).await;
    created_json(audit_backend_mutation(&context, "okf.log.entries.create", result).await?)
}

pub(crate) async fn create_okf_export(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfBundleExportRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.bundle.export.create")?;
    let result = state.api.create_okf_export(request).await;
    created_json(audit_backend_mutation(&context, "okf.bundle.export.create", result).await?)
}

pub(crate) async fn create_okf_import(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfBundleImportRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.bundle.import.create")?;
    let result = state.api.create_okf_import(request).await;
    created_json(audit_backend_mutation(&context, "okf.bundle.import.create", result).await?)
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
    let context = require_backend_mutation_context(&state, context, "okf.lintRuns.create")?;
    let result = state.api.create_okf_lint_run(request).await;
    created_json(audit_backend_mutation(&context, "okf.lintRuns.create", result).await?)
}

pub(crate) async fn create_okf_eval_run(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "okf.evalRuns.create")?;
    let result = state.api.create_okf_eval_run(request).await;
    created_json(audit_backend_mutation(&context, "okf.evalRuns.create", result).await?)
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
    let result = state
        .api
        .create_index(request.with_tenant_id(context.tenant_id))
        .await;
    created_json(audit_backend_mutation(&context, "indexes.create", result).await?)
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
    let context = require_backend_mutation_context(&state, context, "indexes.rebuild")?;
    let result = state.api.rebuild_index(index_id, request).await;
    ok_json(audit_backend_mutation(&context, "indexes.rebuild", result).await?)
}

pub(crate) async fn create_retrieval_profile(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<KnowledgeRetrievalProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(&state, context, "retrievalProfiles.create")?;
    let result = state
        .api
        .create_retrieval_profile(request.with_tenant_id(context.tenant_id))
        .await;
    created_json(audit_backend_mutation(&context, "retrievalProfiles.create", result).await?)
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
    let result = state
        .api
        .update_retrieval_profile(profile_id, request.with_tenant_id(context.tenant_id))
        .await;
    ok_json(audit_backend_mutation(&context, "retrievalProfiles.update", result).await?)
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
    let context =
        require_backend_mutation_context(&state, context, "compliance.auditEvents.export.create")?;
    let result = state.api.export_audit_events(request).await;
    created_json(
        audit_backend_mutation(&context, "compliance.auditEvents.export.create", result).await?,
    )
}

pub(crate) async fn anonymize_audit_subject(
    State(state): State<BackendState>,
    context: RequiredBackendContext,
    Json(request): Json<AnonymizeKnowledgeAuditSubjectRequest>,
) -> Result<Response, BackendApiProblem> {
    let context = require_backend_mutation_context(
        &state,
        context,
        "compliance.auditEvents.anonymizeActor.create",
    )?;
    let result = state.api.anonymize_audit_subject(request).await;
    created_json(
        audit_backend_mutation(
            &context,
            "compliance.auditEvents.anonymizeActor.create",
            result,
        )
        .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BackendApiError;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    async fn audit_test_lock() -> tokio::sync::MutexGuard<'static, ()> {
        static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
        LOCK.lock().await
    }

    fn admin_context() -> KnowledgeBackendRequestContext {
        KnowledgeBackendRequestContext {
            tenant_id: 100_001,
            operator_id: Some(99),
            organization_id: Some(7),
            permission_scope: vec![],
        }
    }

    #[tokio::test]
    async fn mutation_audit_never_reports_failed_business_result_as_success() {
        let _guard = audit_test_lock().await;
        let called = Arc::new(AtomicBool::new(false));
        let called_by_handler = Arc::clone(&called);
        sdkwork_knowledgebase_observability::install_audit_persistence(move |_event| {
            let called = Arc::clone(&called_by_handler);
            async move {
                called.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        let business_failure: BackendApiResult<()> =
            Err(BackendApiError::unsupported_operation("sources.create"));
        let result = audit_backend_mutation(&admin_context(), "sources.create", business_failure)
            .await
            .expect("business result");

        assert!(result.is_err());
        assert!(!called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn mutation_audit_failure_prevents_success_response() {
        let _guard = audit_test_lock().await;
        sdkwork_knowledgebase_observability::install_audit_persistence(|_event| async {
            Err(
                sdkwork_knowledgebase_observability::AuditPersistenceError::write_failed(
                    "database unavailable",
                ),
            )
        });

        let result = audit_backend_mutation(
            &admin_context(),
            "sources.create",
            Ok::<(), BackendApiError>(()),
        )
        .await;

        assert!(result.is_err());
    }
}
