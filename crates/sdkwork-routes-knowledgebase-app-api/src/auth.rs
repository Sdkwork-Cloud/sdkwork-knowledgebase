use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::ops::Deref;

use crate::{ApiProblem, KnowledgeAppRequestContext};

/// Authenticated app request context injected by `sdkwork-web-framework` middleware.
#[derive(Debug, Clone)]
pub struct RequiredAppContext(pub KnowledgeAppRequestContext);

impl Deref for RequiredAppContext {
    type Target = KnowledgeAppRequestContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequiredAppContext
where
    S: Send + Sync,
{
    type Rejection = ApiProblem;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<KnowledgeAppRequestContext>()
            .cloned()
            .map(RequiredAppContext)
            .ok_or_else(|| {
                ApiProblem::new(
                    StatusCode::UNAUTHORIZED,
                    "missing_app_request_context",
                    "authenticated app request context is required",
                )
            })
    }
}

/// Returns the authenticated app request context after extractor validation.
pub fn require_app_context(
    context: RequiredAppContext,
) -> Result<KnowledgeAppRequestContext, ApiProblem> {
    Ok(context.0)
}
