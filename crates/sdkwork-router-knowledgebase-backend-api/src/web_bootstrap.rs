use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{DomainContextInjector, WebRequestContext, WebRequestContextProfile};

use crate::paths;
use crate::KnowledgeBackendRequestContext;

pub fn knowledgebase_backend_public_path_prefixes() -> Vec<String> {
    vec![paths::HEALTHZ.to_owned()]
}

#[derive(Clone, Default)]
struct KnowledgeBackendContextInjector;

impl DomainContextInjector for KnowledgeBackendContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(backend_context) = knowledge_backend_context_from_web_request(context) {
            request.extensions_mut().insert(backend_context);
        }
    }
}

fn knowledge_backend_context_from_web_request(
    context: &WebRequestContext,
) -> Option<KnowledgeBackendRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let operator_id = principal.user_id().parse().ok();
    Some(KnowledgeBackendRequestContext {
        tenant_id,
        operator_id,
    })
}

pub fn wrap_router_with_web_framework(
    resolver: IamDatabaseWebRequestContextResolver,
    router: Router,
) -> Router {
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            public_path_prefixes: knowledgebase_backend_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_domain_injector(Arc::new(KnowledgeBackendContextInjector));
    with_web_request_context(router, layer)
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_database_resolver_from_env().await;
    wrap_router_with_web_framework(resolver, router)
}
