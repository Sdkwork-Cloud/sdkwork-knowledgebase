use axum::{http::StatusCode, Extension};

use crate::{BackendApiProblem, KnowledgeBackendRequestContext};

pub fn require_backend_context(
    context: Option<Extension<KnowledgeBackendRequestContext>>,
) -> Result<KnowledgeBackendRequestContext, BackendApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        BackendApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_backend_request_context",
            "authenticated backend request context is required",
        )
    })
}
