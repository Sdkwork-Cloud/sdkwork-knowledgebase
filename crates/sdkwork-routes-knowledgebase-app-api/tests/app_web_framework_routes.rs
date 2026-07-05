use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_knowledgebase_contract::browser::ListKnowledgeBrowserRequest;
use sdkwork_routes_knowledgebase_app_api::{
    app_route_manifest, build_router_with_browser, manifest, pagination::browser_list_page_data,
    wrap_router_with_web_framework, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
};
use sdkwork_web_core::RouteAuth;
use sdkwork_web_core::{access_token_jwt, auth_token_jwt_with_permissions};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

#[test]
fn app_route_manifest_declares_dual_token_auth_for_all_operations() {
    let manifest = app_route_manifest();
    assert_eq!(manifest::ROUTES.len(), manifest.routes().len());
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
async fn app_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_browser(EmptyBrowserApi),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/knowledge/spaces/7/browser")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn web_framework_accepts_dev_jwt_dual_tokens_before_handler() {
    std::env::set_var("SDKWORK_ENV", "dev");
    std::env::set_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK", "true");
    let service = RecordingBrowserApi::default();
    let app = wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_browser(service.clone()),
    );
    let auth = format!(
        "Bearer {}",
        auth_token_jwt_with_permissions(
            "100001",
            "30001",
            "session-1",
            "sdkwork-knowledgebase",
            "knowledge.spaces.read",
        )
    );
    let access = access_token_jwt("100001", "30001", "session-1", "sdkwork-knowledgebase");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/app/v3/api/knowledge/spaces/7/browser")
                .header("Authorization", auth)
                .header("Access-Token", access)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(service.tenant_ids(), vec![100001]);
    std::env::remove_var("SDKWORK_ENV");
    std::env::remove_var("SDKWORK_IAM_ALLOW_DEV_AUTH_FALLBACK");
}

#[tokio::test]
async fn web_framework_allows_browser_origin_in_development() {
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    let app = wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_browser(EmptyBrowserApi),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/app/v3/api/knowledge/spaces")
                .header("origin", "http://127.0.0.1:5184")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Dev Space"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::FORBIDDEN);
    std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
}

#[tokio::test]
async fn web_framework_rejects_unlisted_browser_origin_in_development() {
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    let app = wrap_router_with_web_framework(
        IamWebRequestContextResolver::new(None),
        build_router_with_browser(EmptyBrowserApi),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/app/v3/api/knowledge/spaces")
                .header("origin", "http://evil.example")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Dev Space"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
}

struct EmptyBrowserApi;

#[async_trait]
impl KnowledgeBrowserApi for EmptyBrowserApi {
    async fn list_browser(
        &self,
        _context: KnowledgeAppRequestContext,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<
        sdkwork_utils_rust::SdkWorkPageData<sdkwork_knowledgebase_contract::KnowledgeBrowserNode>,
    > {
        unreachable!("unauthenticated requests must not reach handlers")
    }
}

#[derive(Clone, Default)]
struct RecordingBrowserApi {
    tenant_ids: Arc<Mutex<Vec<u64>>>,
}

impl RecordingBrowserApi {
    fn tenant_ids(&self) -> Vec<u64> {
        self.tenant_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeBrowserApi for RecordingBrowserApi {
    async fn list_browser(
        &self,
        context: KnowledgeAppRequestContext,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<
        sdkwork_utils_rust::SdkWorkPageData<sdkwork_knowledgebase_contract::KnowledgeBrowserNode>,
    > {
        self.tenant_ids.lock().unwrap().push(context.tenant_id);
        Ok(browser_list_page_data(vec![], None, 50))
    }
}
