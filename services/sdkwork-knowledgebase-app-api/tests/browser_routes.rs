use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use sdkwork_knowledgebase_app_api::{
    build_router_with_browser, KnowledgeBrowserApi, ProblemDetails,
};
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserNode, KnowledgeBrowserNodePermissions, KnowledgeBrowserNodeType,
    KnowledgeBrowserPage, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use std::sync::Mutex;
use tower::util::ServiceExt;

#[tokio::test]
async fn app_router_exposes_browser_route_with_query_parameters() {
    let browser = RecordingBrowserApi::default();
    let app = build_router_with_browser(browser.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/knowledge/spaces/7/browser?view=wiki&pageSize=25&parentId=node-wiki&cursor=c1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let page: KnowledgeBrowserPage = serde_json::from_slice(&body).unwrap();
    assert_eq!(page.space_id, 7);
    assert_eq!(page.view, KnowledgeBrowserView::Wiki);
    assert_eq!(page.items[0].node_type, KnowledgeBrowserNodeType::WikiPage);
    assert_eq!(
        browser.last_request().unwrap(),
        ListKnowledgeBrowserRequest {
            space_id: 7,
            parent_id: Some("node-wiki".to_string()),
            view: KnowledgeBrowserView::Wiki,
            cursor: Some("c1".to_string()),
            page_size: Some(25),
        }
    );
}

#[tokio::test]
async fn app_router_rejects_invalid_browser_view() {
    let app = build_router_with_browser(RecordingBrowserApi::default());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/app/v3/api/knowledge/spaces/7/browser?view=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let problem: ProblemDetails = serde_json::from_slice(&body).unwrap();
    assert_eq!(problem.r#type, "about:blank");
    assert_eq!(problem.title, "Bad Request");
    assert_eq!(problem.status, 400);
    assert_eq!(problem.code.as_deref(), Some("invalid_browser_view"));
    assert_eq!(
        problem.detail.as_deref(),
        Some("unsupported browser view: invalid")
    );
}

#[derive(Clone, Default)]
struct RecordingBrowserApi {
    last_request: std::sync::Arc<Mutex<Option<ListKnowledgeBrowserRequest>>>,
}

impl RecordingBrowserApi {
    fn last_request(&self) -> Option<ListKnowledgeBrowserRequest> {
        self.last_request.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeBrowserApi for RecordingBrowserApi {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> Result<KnowledgeBrowserPage, String> {
        *self.last_request.lock().unwrap() = Some(request.clone());
        Ok(KnowledgeBrowserPage {
            space_id: request.space_id,
            drive_space_id: "drv-kb-001".to_string(),
            parent_id: request.parent_id,
            view: request.view,
            page_size: request.page_size.unwrap_or(50),
            items: vec![KnowledgeBrowserNode {
                id: "node-index".to_string(),
                node_type: KnowledgeBrowserNodeType::WikiPage,
                name: "index.md".to_string(),
                parent_id: Some("node-wiki".to_string()),
                path: "wiki/index.md".to_string(),
                drive_space_id: Some("drv-kb-001".to_string()),
                drive_node_id: Some("node-index".to_string()),
                document_id: None,
                document_version_id: None,
                wiki_page_id: Some(1),
                wiki_revision_id: Some(2),
                mime_type: Some("text/markdown; charset=utf-8".to_string()),
                size_bytes: Some(64),
                ingest_state: None,
                parse_state: None,
                index_state: None,
                wiki_state: Some("published".to_string()),
                children_count: None,
                updated_at: "2026-06-04T12:00:00Z".to_string(),
                permissions: KnowledgeBrowserNodePermissions::read_only(),
            }],
            next_cursor: None,
        })
    }
}
