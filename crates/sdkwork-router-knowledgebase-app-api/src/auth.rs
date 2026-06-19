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
