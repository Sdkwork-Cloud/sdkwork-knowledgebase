use async_trait::async_trait;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserPage, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[async_trait]
pub trait KnowledgeBrowserApi: Send + Sync + 'static {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> Result<KnowledgeBrowserPage, String>;
}

#[derive(Clone)]
struct AppState {
    browser: Arc<dyn KnowledgeBrowserApi>,
}

pub fn build_router_with_browser<B>(browser: B) -> Router
where
    B: KnowledgeBrowserApi,
{
    build_router_with_shared_browser(Arc::new(browser))
}

pub fn build_router_with_shared_browser(browser: Arc<dyn KnowledgeBrowserApi>) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route(
            "/app/v3/api/knowledge/spaces/:space_id/browser",
            get(list_browser),
        )
        .with_state(AppState { browser })
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_browser(
    State(state): State<AppState>,
    Path(space_id): Path<u64>,
    Query(query): Query<ListBrowserQuery>,
) -> Result<Json<KnowledgeBrowserPage>, (StatusCode, Json<ProblemDetails>)> {
    let view = parse_view(query.view.as_deref())?;
    let page = state
        .browser
        .list_browser(ListKnowledgeBrowserRequest {
            space_id,
            parent_id: query.parent_id,
            view,
            cursor: query.cursor,
            page_size: query.page_size,
        })
        .await
        .map_err(|detail| {
            problem(
                StatusCode::INTERNAL_SERVER_ERROR,
                "browser_list_failed",
                detail,
            )
        })?;
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListBrowserQuery {
    view: Option<String>,
    parent_id: Option<String>,
    cursor: Option<String>,
    page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetails {
    pub code: String,
    pub message: String,
}

fn parse_view(
    value: Option<&str>,
) -> Result<KnowledgeBrowserView, (StatusCode, Json<ProblemDetails>)> {
    match value.unwrap_or("files") {
        "files" => Ok(KnowledgeBrowserView::Files),
        "wiki" => Ok(KnowledgeBrowserView::Wiki),
        "outputs" => Ok(KnowledgeBrowserView::Outputs),
        value => Err(problem(
            StatusCode::BAD_REQUEST,
            "invalid_browser_view",
            format!("unsupported browser view: {value}"),
        )),
    }
}

fn problem(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> (StatusCode, Json<ProblemDetails>) {
    (
        status,
        Json(ProblemDetails {
            code: code.into(),
            message: message.into(),
        }),
    )
}
