use async_trait::async_trait;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, IngestionJob, KnowledgeBrowserPage, KnowledgeBrowserView,
    KnowledgeDocument, KnowledgeDocumentList, KnowledgeDocumentVersion,
    KnowledgeDocumentVersionList, KnowledgeDriveImportRequest, KnowledgeDriveImportResult,
    KnowledgeIngestRequest, KnowledgeSpace, KnowledgeWikiFileEntry, KnowledgeWikiPageRevisionList,
    ListKnowledgeBrowserRequest, WikiContextPackRequest, WikiFileAnswerRequest, WikiIndexDocument,
    WikiLogDocument, WikiPageSummary, WikiPageSummaryList, WikiQueryRequest, WikiQueryResult,
    WikiSchemaDocument,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use sdkwork_knowledgebase_contract::ProblemDetails;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    code: String,
    detail: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
        }
    }

    pub fn internal(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, code, detail)
    }

    pub fn not_implemented(operation_id: &'static str) -> Self {
        Self::new(
            StatusCode::NOT_IMPLEMENTED,
            "operation_not_implemented",
            format!("operation is not implemented: {operation_id}"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ApiProblem {
    status: StatusCode,
    problem: Box<ProblemDetails>,
}

impl ApiProblem {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        let title = status
            .canonical_reason()
            .unwrap_or("HTTP Error")
            .to_string();
        Self {
            status,
            problem: Box::new(ProblemDetails {
                r#type: "about:blank".to_string(),
                title,
                status: status.as_u16(),
                detail: Some(detail.into()),
                instance: None,
                code: Some(code.into()),
            }),
        }
    }
}

impl From<ApiError> for ApiProblem {
    fn from(error: ApiError) -> Self {
        Self::new(error.status, error.code, error.detail)
    }
}

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(*self.problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

#[async_trait]
pub trait KnowledgeBrowserApi: Send + Sync + 'static {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> Result<KnowledgeBrowserPage, String>;
}

#[async_trait]
pub trait KnowledgeAppApi: Send + Sync + 'static {
    async fn create_space(
        &self,
        _request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        Err(ApiError::not_implemented("spaces.create"))
    }

    async fn retrieve_space(&self, _space_id: u64) -> ApiResult<KnowledgeSpace> {
        Err(ApiError::not_implemented("spaces.retrieve"))
    }

    async fn create_drive_import(
        &self,
        _request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        Err(ApiError::not_implemented("driveImports.create"))
    }

    async fn create_ingest(&self, _request: KnowledgeIngestRequest) -> ApiResult<IngestionJob> {
        Err(ApiError::not_implemented("ingests.create"))
    }

    async fn retrieve_ingest(&self, _ingest_id: u64) -> ApiResult<IngestionJob> {
        Err(ApiError::not_implemented("ingests.retrieve"))
    }

    async fn list_documents(&self) -> ApiResult<KnowledgeDocumentList> {
        Err(ApiError::not_implemented("documents.list"))
    }

    async fn create_document(
        &self,
        _request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.create"))
    }

    async fn retrieve_document(&self, _document_id: u64) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.retrieve"))
    }

    async fn update_document(
        &self,
        _document_id: u64,
        _request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.update"))
    }

    async fn delete_document(&self, _document_id: u64) -> ApiResult<()> {
        Err(ApiError::not_implemented("documents.delete"))
    }

    async fn list_document_versions(
        &self,
        _document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        Err(ApiError::not_implemented("documents.versions.list"))
    }

    async fn create_document_version(
        &self,
        _document_id: u64,
        _request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        Err(ApiError::not_implemented("documents.versions.create"))
    }

    async fn list_wiki_pages(&self) -> ApiResult<WikiPageSummaryList> {
        Err(ApiError::not_implemented("wiki.pages.list"))
    }

    async fn retrieve_wiki_page(&self, _page_id: u64) -> ApiResult<WikiPageSummary> {
        Err(ApiError::not_implemented("wiki.pages.retrieve"))
    }

    async fn list_wiki_page_revisions(
        &self,
        _page_id: u64,
    ) -> ApiResult<KnowledgeWikiPageRevisionList> {
        Err(ApiError::not_implemented("wiki.pages.revisions.list"))
    }

    async fn retrieve_wiki_index(&self) -> ApiResult<WikiIndexDocument> {
        Err(ApiError::not_implemented("wiki.index.retrieve"))
    }

    async fn retrieve_wiki_log(&self) -> ApiResult<WikiLogDocument> {
        Err(ApiError::not_implemented("wiki.log.retrieve"))
    }

    async fn retrieve_wiki_schema(&self) -> ApiResult<WikiSchemaDocument> {
        Err(ApiError::not_implemented("wiki.schema.retrieve"))
    }

    async fn create_wiki_query(&self, _request: WikiQueryRequest) -> ApiResult<WikiQueryResult> {
        Err(ApiError::not_implemented("wiki.queries.create"))
    }

    async fn file_wiki_query_answer(
        &self,
        _query_id: u64,
        _request: WikiFileAnswerRequest,
    ) -> ApiResult<WikiQueryResult> {
        Err(ApiError::not_implemented("wiki.queries.fileAnswer"))
    }

    async fn create_wiki_context_pack(
        &self,
        _request: WikiContextPackRequest,
    ) -> ApiResult<KnowledgeWikiFileEntry> {
        Err(ApiError::not_implemented("wiki.contextPacks.create"))
    }

    async fn list_browser(
        &self,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        Err(ApiError::not_implemented("spaces.browser.list"))
    }
}

#[derive(Clone)]
struct AppState {
    api: Arc<dyn KnowledgeAppApi>,
}

pub fn build_router_with_browser<B>(browser: B) -> Router
where
    B: KnowledgeBrowserApi,
{
    build_router_with_shared_browser(Arc::new(browser))
}

pub fn build_router_with_shared_browser(browser: Arc<dyn KnowledgeBrowserApi>) -> Router {
    build_router_with_shared_app_api(Arc::new(BrowserOnlyAppApi { browser }))
}

pub fn build_router_with_app_api<A>(api: A) -> Router
where
    A: KnowledgeAppApi,
{
    build_router_with_shared_app_api(Arc::new(api))
}

pub fn build_router_with_shared_app_api(api: Arc<dyn KnowledgeAppApi>) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/app/v3/api/knowledge/spaces", post(create_space))
        .route(
            "/app/v3/api/knowledge/spaces/:space_id",
            get(retrieve_space),
        )
        .route(
            "/app/v3/api/knowledge/drive_imports",
            post(create_drive_import),
        )
        .route("/app/v3/api/knowledge/ingests", post(create_ingest))
        .route(
            "/app/v3/api/knowledge/ingests/:ingest_id",
            get(retrieve_ingest),
        )
        .route(
            "/app/v3/api/knowledge/documents",
            get(list_documents).post(create_document),
        )
        .route(
            "/app/v3/api/knowledge/documents/:document_id",
            get(retrieve_document)
                .patch(update_document)
                .delete(delete_document),
        )
        .route(
            "/app/v3/api/knowledge/documents/:document_id/versions",
            get(list_document_versions).post(create_document_version),
        )
        .route("/app/v3/api/knowledge/wiki_pages", get(list_wiki_pages))
        .route(
            "/app/v3/api/knowledge/wiki_pages/:page_id",
            get(retrieve_wiki_page),
        )
        .route(
            "/app/v3/api/knowledge/wiki_pages/:page_id/revisions",
            get(list_wiki_page_revisions),
        )
        .route("/app/v3/api/knowledge/wiki_index", get(retrieve_wiki_index))
        .route("/app/v3/api/knowledge/wiki_log", get(retrieve_wiki_log))
        .route(
            "/app/v3/api/knowledge/wiki_schema",
            get(retrieve_wiki_schema),
        )
        .route(
            "/app/v3/api/knowledge/wiki_queries",
            post(create_wiki_query),
        )
        .route(
            "/app/v3/api/knowledge/wiki_queries/:query_id/file_answer",
            post(file_wiki_query_answer),
        )
        .route(
            "/app/v3/api/knowledge/wiki_context_packs",
            post(create_wiki_context_pack),
        )
        .route(
            "/app/v3/api/knowledge/spaces/:space_id/browser",
            get(list_browser),
        )
        .with_state(AppState { api })
}

struct BrowserOnlyAppApi {
    browser: Arc<dyn KnowledgeBrowserApi>,
}

#[async_trait]
impl KnowledgeAppApi for BrowserOnlyAppApi {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser
            .list_browser(request)
            .await
            .map_err(|detail| ApiError::internal("browser_list_failed", detail))
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn create_space(
    State(state): State<AppState>,
    Json(request): Json<CreateKnowledgeSpaceRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_space(request).await)
}

async fn retrieve_space(
    State(state): State<AppState>,
    Path(space_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_space(space_id).await)
}

async fn create_drive_import(
    State(state): State<AppState>,
    Json(request): Json<KnowledgeDriveImportRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_drive_import(request).await)
}

async fn create_ingest(
    State(state): State<AppState>,
    Json(request): Json<KnowledgeIngestRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_ingest(request).await)
}

async fn retrieve_ingest(
    State(state): State<AppState>,
    Path(ingest_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_ingest(ingest_id).await)
}

async fn list_documents(State(state): State<AppState>) -> Result<Response, ApiProblem> {
    ok_json(state.api.list_documents().await)
}

async fn create_document(
    State(state): State<AppState>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_document(request).await)
}

async fn retrieve_document(
    State(state): State<AppState>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_document(document_id).await)
}

async fn update_document(
    State(state): State<AppState>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.update_document(document_id, request).await)
}

async fn delete_document(
    State(state): State<AppState>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    state
        .api
        .delete_document(document_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_document_versions(
    State(state): State<AppState>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.list_document_versions(document_id).await)
}

async fn create_document_version(
    State(state): State<AppState>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentVersionRequest>,
) -> Result<Response, ApiProblem> {
    if request.document_id != document_id {
        return Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "document_id_mismatch",
            "documentId in body must match documentId in path",
        ));
    }
    created_json(
        state
            .api
            .create_document_version(document_id, request)
            .await,
    )
}

async fn list_wiki_pages(State(state): State<AppState>) -> Result<Response, ApiProblem> {
    ok_json(state.api.list_wiki_pages().await)
}

async fn retrieve_wiki_page(
    State(state): State<AppState>,
    Path(page_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_wiki_page(page_id).await)
}

async fn list_wiki_page_revisions(
    State(state): State<AppState>,
    Path(page_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.list_wiki_page_revisions(page_id).await)
}

async fn retrieve_wiki_index(State(state): State<AppState>) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_wiki_index().await)
}

async fn retrieve_wiki_log(State(state): State<AppState>) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_wiki_log().await)
}

async fn retrieve_wiki_schema(State(state): State<AppState>) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_wiki_schema().await)
}

async fn create_wiki_query(
    State(state): State<AppState>,
    Json(request): Json<WikiQueryRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_wiki_query(request).await)
}

async fn file_wiki_query_answer(
    State(state): State<AppState>,
    Path(query_id): Path<u64>,
    Json(request): Json<WikiFileAnswerRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.file_wiki_query_answer(query_id, request).await)
}

async fn create_wiki_context_pack(
    State(state): State<AppState>,
    Json(request): Json<WikiContextPackRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_wiki_context_pack(request).await)
}

async fn list_browser(
    State(state): State<AppState>,
    Path(space_id): Path<u64>,
    Query(query): Query<ListBrowserQuery>,
) -> Result<Response, ApiProblem> {
    let view = parse_view(query.view.as_deref())?;
    ok_json(
        state
            .api
            .list_browser(ListKnowledgeBrowserRequest {
                space_id,
                parent_id: query.parent_id,
                view,
                cursor: query.cursor,
                page_size: query.page_size,
            })
            .await,
    )
}

fn ok_json<T>(result: ApiResult<T>) -> Result<Response, ApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| Json(value).into_response())
        .map_err(ApiProblem::from)
}

fn created_json<T>(result: ApiResult<T>) -> Result<Response, ApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| (StatusCode::CREATED, Json(value)).into_response())
        .map_err(ApiProblem::from)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListBrowserQuery {
    view: Option<String>,
    parent_id: Option<String>,
    cursor: Option<String>,
    page_size: Option<u32>,
}

fn parse_view(value: Option<&str>) -> Result<KnowledgeBrowserView, ApiProblem> {
    match value.unwrap_or("files") {
        "files" => Ok(KnowledgeBrowserView::Files),
        "wiki" => Ok(KnowledgeBrowserView::Wiki),
        "outputs" => Ok(KnowledgeBrowserView::Outputs),
        value => Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "invalid_browser_view",
            format!("unsupported browser view: {value}"),
        )),
    }
}
