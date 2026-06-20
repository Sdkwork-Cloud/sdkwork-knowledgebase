use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_knowledgebase_contract::KnowledgeSourceList;
use sdkwork_router_knowledgebase_backend_api::{
    backend_route_manifest, build_router_with_backend_api, manifest,
    wrap_router_with_web_framework, BackendApiResult, KnowledgeBackendApi,
};
use sdkwork_web_core::RouteAuth;
use tower::util::ServiceExt;

const DEV_AUTH_TOKEN: &str =
    "Bearer tenant_id=20001;user_id=30001;session_id=s-1;app_id=knowledgebase;auth_level=password";
const DEV_ACCESS_TOKEN: &str =
    "tenant_id=20001;user_id=30001;session_id=s-1;app_id=knowledgebase;environment=dev;deployment_mode=saas";

#[test]
fn backend_route_manifest_declares_dual_token_auth_for_all_operations() {
    let manifest = backend_route_manifest();
    assert_eq!(manifest::ROUTES.len(), 26);
    for entry in manifest::ROUTES {
        let matched = manifest
            .match_route(entry.method, entry.path)
            .unwrap_or_else(|| {
                panic!(
                    "missing http route manifest for {} {}",
                    entry.method, entry.path
                )
            });
        assert_eq!(matched.auth, RouteAuth::DualToken);
        assert_eq!(matched.operation_id, entry.operation_id);
    }
}

#[tokio::test]
async fn backend_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_backend_api(EmptyBackendApi),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/backend/v3/api/knowledge/sources")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn backend_router_web_framework_accepts_dev_inline_dual_tokens_before_handler() {
    let app = wrap_router_with_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_backend_api(OkBackendApi),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/backend/v3/api/knowledge/sources")
                .header("Authorization", DEV_AUTH_TOKEN)
                .header("Access-Token", DEV_ACCESS_TOKEN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

struct EmptyBackendApi;

impl KnowledgeBackendApi for EmptyBackendApi {}

struct OkBackendApi;

#[async_trait]
impl KnowledgeBackendApi for OkBackendApi {
    async fn list_sources(&self) -> BackendApiResult<KnowledgeSourceList> {
        Ok(KnowledgeSourceList { items: vec![] })
    }
}
