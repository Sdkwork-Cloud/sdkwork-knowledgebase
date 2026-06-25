use axum::{http::StatusCode, Extension};

use crate::{ApiProblem, KnowledgeOpenApiRequestContext};

pub fn require_context(
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
) -> Result<KnowledgeOpenApiRequestContext, ApiProblem> {
    context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_open_api_request_context",
            "authenticated open API credential context is required",
        )
    })
}
