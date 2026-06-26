use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserPage, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use sdkwork_routes_knowledgebase_app_api::{
    app_route_manifest, build_router_with_browser, manifest,
    wrap_router_with_web_framework, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
};
use sdkwork_web_core::{access_token_jwt, auth_token_jwt};
use sdkwork_web_core::RouteAuth;
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
        auth_token_jwt("100001", "30001", "session-1", "sdkwork-knowledgebase")
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

struct EmptyBrowserApi;

#[async_trait]
impl KnowledgeBrowserApi for EmptyBrowserApi {
    async fn list_browser(
        &self,
        _context: KnowledgeAppRequestContext,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
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
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.tenant_ids.lock().unwrap().push(context.tenant_id);
        Ok(KnowledgeBrowserPage {
            space_id: 7,
            drive_space_id: "drv-kb-001".to_string(),
            parent_id: None,
            view: KnowledgeBrowserView::Files,
            page_size: 50,
            items: vec![],
            next_cursor: None,
        })
    }
}
