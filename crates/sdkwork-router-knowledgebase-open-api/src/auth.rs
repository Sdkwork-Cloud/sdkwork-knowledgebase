use axum::{http::StatusCode, Extension};

use crate::{ApiProblem, KnowledgeOpenApiRequestContext};

pub fn require_context(
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
) -> Result<KnowledgeOpenApiRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_open_api_request_context",
            "authenticated open API key context is required",
        )
    })
}

pub fn ensure_tenant_matches(
    context: &KnowledgeOpenApiRequestContext,
    request_tenant_id: u64,
) -> Result<(), ApiProblem> {
    if request_tenant_id != context.tenant_id {
        return Err(ApiProblem::new(
            StatusCode::FORBIDDEN,
            "tenant_id_mismatch",
            "request tenantId must match authenticated open API tenant context",
        ));
    }
    Ok(())
}
