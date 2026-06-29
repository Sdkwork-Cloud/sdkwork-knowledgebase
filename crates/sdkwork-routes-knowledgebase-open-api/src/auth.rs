use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::ops::Deref;

use crate::{ApiProblem, KnowledgeOpenApiRequestContext};

/// Authenticated open-api request context injected by `sdkwork-web-framework` middleware.
#[derive(Debug, Clone)]
pub struct RequiredOpenContext(pub KnowledgeOpenApiRequestContext);

impl Deref for RequiredOpenContext {
    type Target = KnowledgeOpenApiRequestContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequiredOpenContext
where
    S: Send + Sync,
{
    type Rejection = ApiProblem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<KnowledgeOpenApiRequestContext>()
            .cloned()
            .map(RequiredOpenContext)
            .ok_or_else(|| {
                ApiProblem::new(
                    StatusCode::UNAUTHORIZED,
                    "missing_open_api_request_context",
                    "authenticated open-api request context is required",
                )
            })
    }
}

/// Returns the authenticated open-api request context after extractor validation.
pub fn require_context(
    context: RequiredOpenContext,
) -> Result<KnowledgeOpenApiRequestContext, ApiProblem> {
    Ok(context.0)
}
