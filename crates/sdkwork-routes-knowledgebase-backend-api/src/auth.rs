use axum::{http::StatusCode, Extension};

use crate::{
    permission::can_access_knowledge_admin, routes::BackendState, BackendApiProblem,
    KnowledgeBackendRequestContext,
};

pub fn require_backend_context(
    state: &BackendState,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
) -> Result<KnowledgeBackendRequestContext, BackendApiProblem> {
    let context = context.map(|Extension(context)| context).ok_or_else(|| {
        BackendApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_backend_request_context",
            "authenticated backend request context is required",
        )
    })?;
    ensure_runtime_tenant(state, &context)?;
    ensure_runtime_organization(&context)?;
    ensure_knowledge_admin_permission(&context)?;
    Ok(context)
}

pub fn require_backend_mutation_context(
    state: &BackendState,
    context: Option<Extension<KnowledgeBackendRequestContext>>,
    operation: &str,
) -> Result<KnowledgeBackendRequestContext, BackendApiProblem> {
    let context = require_backend_context(state, context)?;
    sdkwork_knowledgebase_observability::record_backend_admin_operation(
        operation,
        context.tenant_id,
        context.operator_id.unwrap_or(0),
    );
    Ok(context)
}

pub fn ensure_runtime_tenant(
    state: &BackendState,
    context: &KnowledgeBackendRequestContext,
) -> Result<(), BackendApiProblem> {
    if context.tenant_id != state.runtime_tenant_id {
        return Err(BackendApiProblem::new(
            StatusCode::FORBIDDEN,
            "tenant_id_mismatch",
            "authenticated tenant does not match configured runtime tenant",
        ));
    }
    Ok(())
}

pub fn ensure_runtime_organization(
    context: &KnowledgeBackendRequestContext,
) -> Result<(), BackendApiProblem> {
    let runtime_org = configured_runtime_organization_id();
    if runtime_org == 0 {
        return Ok(());
    }
    let Some(context_org) = context.organization_id else {
        return Err(BackendApiProblem::new(
            StatusCode::FORBIDDEN,
            "missing_organization_id",
            "organization context is required for this operation",
        ));
    };
    if context_org != runtime_org {
        return Err(BackendApiProblem::new(
            StatusCode::FORBIDDEN,
            "organization_id_mismatch",
            "authenticated organization does not match configured runtime organization",
        ));
    }
    Ok(())
}

fn ensure_knowledge_admin_permission(
    context: &KnowledgeBackendRequestContext,
) -> Result<(), BackendApiProblem> {
    if can_access_knowledge_admin(context) {
        return Ok(());
    }
    Err(BackendApiProblem::new(
        StatusCode::FORBIDDEN,
        "knowledge_admin_permission_required",
        "knowledge.admin permission is required for backend-api operations",
    ))
}

fn configured_runtime_organization_id() -> u64 {
    std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}
