use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_routes_knowledgebase_backend_api::apply_knowledgebase_web_framework;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DefaultWebRequestContextResolver, WebRequestContextProfile, WebRequestContextResolver,
};

use crate::http_route_manifest::internal_route_manifest;

pub fn wrap_with_default_resolver(router: Router) -> Router {
    wrap_with_resolver(DefaultWebRequestContextResolver::default(), router)
}

pub async fn wrap_from_env(router: Router) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_with_iam_resolver(resolver, router)
}

pub fn wrap_with_iam_resolver(resolver: IamWebRequestContextResolver, router: Router) -> Router {
    wrap_with_resolver(resolver, router)
}

fn wrap_with_resolver<R>(resolver: R, router: Router) -> Router
where
    R: WebRequestContextResolver + Clone + Send + Sync + 'static,
{
    let route_manifest = internal_route_manifest();
    let profile = WebRequestContextProfile {
        public_path_prefixes: Vec::new(),
        ..WebRequestContextProfile::default()
    };
    route_manifest
        .validate_route_auth_for_surfaces(&profile)
        .expect("Knowledgebase internal-api routes must use ingress-token auth");
    with_web_request_context(
        router,
        apply_knowledgebase_web_framework(
            WebFrameworkLayer::new(resolver)
                .with_profile(profile)
                .with_route_manifest(route_manifest),
        ),
    )
}
