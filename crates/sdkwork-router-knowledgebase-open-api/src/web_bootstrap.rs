use std::sync::Arc;

use axum::Router;
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{DomainContextInjector, WebRequestContext, WebRequestContextProfile};

use crate::http_route_manifest::open_route_manifest;
use crate::paths;
use crate::KnowledgeOpenApiRequestContext;

pub fn knowledgebase_open_api_public_path_prefixes() -> Vec<String> {
    vec![paths::HEALTHZ.to_owned()]
}

pub fn knowledgebase_open_api_prefixes() -> Vec<String> {
    vec![paths::PREFIX.to_owned()]
}

#[derive(Clone, Default)]
struct KnowledgeOpenApiContextInjector;

impl DomainContextInjector for KnowledgeOpenApiContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(open_context) = knowledge_open_api_context_from_web_request(context) {
            request.extensions_mut().insert(open_context);
        }
    }
}

fn knowledge_open_api_context_from_web_request(
    context: &WebRequestContext,
) -> Option<KnowledgeOpenApiRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().parse().ok()?;
    let actor_id = principal.user_id().parse().ok();
    let credential_id = principal
        .api_key_id()
        .map(str::to_owned)
        .or_else(|| principal.session_id().map(str::to_owned))
        .unwrap_or_else(|| principal.user_id().to_owned());
    Some(KnowledgeOpenApiRequestContext {
        api_key_id: credential_id,
        tenant_id,
        actor_id,
    })
}

pub fn wrap_router_with_web_framework(
    resolver: IamDatabaseWebRequestContextResolver,
    router: Router,
) -> Router {
    let route_manifest = open_route_manifest();
    route_manifest
        .validate_public_path_prefixes(&knowledgebase_open_api_public_path_prefixes())
        .expect("knowledgebase open-api public prefixes must not cover protected manifest routes");

    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(WebRequestContextProfile {
            open_api_prefixes: knowledgebase_open_api_prefixes(),
            public_path_prefixes: knowledgebase_open_api_public_path_prefixes(),
            ..WebRequestContextProfile::default()
        })
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(KnowledgeOpenApiContextInjector));
    with_web_request_context(router, layer)
}

pub async fn wrap_router_with_web_framework_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_database_resolver_from_env().await;
    wrap_router_with_web_framework(resolver, router)
}
