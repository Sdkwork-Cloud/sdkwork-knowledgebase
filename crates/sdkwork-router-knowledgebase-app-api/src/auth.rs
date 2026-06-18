use axum::{http::StatusCode, Extension};

use crate::{ApiProblem, KnowledgeAppRequestContext};

pub fn require_app_context(
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<KnowledgeAppRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_app_request_context",
            "authenticated app request context is required",
        )
    })
}

pub fn ensure_tenant_matches(
    context: &KnowledgeAppRequestContext,
    request_tenant_id: u64,
) -> Result<(), ApiProblem> {
    if request_tenant_id != context.tenant_id {
        return Err(ApiProblem::new(
            StatusCode::FORBIDDEN,
            "tenant_id_mismatch",
            "request tenantId must match authenticated app tenant context",
        ));
    }
    Ok(())
}
