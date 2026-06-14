use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Extension, Json, Router};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, KnowledgeAgentBindingRequest, KnowledgeAgentProfileRequest,
    KnowledgeBrowserView, KnowledgeContextPackRequest, KnowledgeDriveImportRequest,
    KnowledgeIngestRequest, KnowledgeRetrievalRequest, ListKnowledgeBrowserRequest,
    WikiContextPackRequest, WikiFileAnswerRequest, WikiQueryRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::adapters::{
    AgentAndRetrievalAppApi, AgentOnlyAppApi, BrowserOnlyAppApi, FullAppApi, RetrievalOnlyAppApi,
};
use crate::{
    ApiProblem, ApiResult, KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext,
    KnowledgeBrowserApi, KnowledgeDocumentAppService, KnowledgeDriveImportAppService,
    KnowledgeIngestAppService, KnowledgeRetrievalAppService, KnowledgeSpaceAppService,
    KnowledgeWikiAppService,
};

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
    build_router_with_shared_app_api(Arc::new(BrowserOnlyAppApi::new(browser)))
}

pub fn build_router_with_retrieval_service<R>(retrieval: R) -> Router
where
    R: KnowledgeRetrievalAppService,
{
    build_router_with_shared_retrieval_service(Arc::new(retrieval))
}

pub fn build_router_with_shared_retrieval_service(
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
) -> Router {
    build_router_with_shared_app_api(Arc::new(RetrievalOnlyAppApi::new(retrieval)))
}

pub fn build_router_with_agent_service<A>(agent: A) -> Router
where
    A: KnowledgeAgentAppService,
{
    build_router_with_shared_agent_service(Arc::new(agent))
}

pub fn build_router_with_shared_agent_service(agent: Arc<dyn KnowledgeAgentAppService>) -> Router {
    build_router_with_shared_app_api(Arc::new(AgentOnlyAppApi::new(agent)))
}

pub fn build_router_with_agent_and_retrieval_services<A, R>(agent: A, retrieval: R) -> Router
where
    A: KnowledgeAgentAppService,
    R: KnowledgeRetrievalAppService,
{
    build_router_with_shared_agent_and_retrieval_services(Arc::new(agent), Arc::new(retrieval))
}

pub fn build_router_with_shared_agent_and_retrieval_services(
    agent: Arc<dyn KnowledgeAgentAppService>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
) -> Router {
    build_router_with_shared_app_api(Arc::new(AgentAndRetrievalAppApi::new(agent, retrieval)))
}

pub fn build_router_with_app_api<A>(api: A) -> Router
where
    A: KnowledgeAppApi,
{
    build_router_with_shared_app_api(Arc::new(api))
}

#[allow(clippy::too_many_arguments)]
pub fn build_router_with_full_app_api(
    space: Arc<dyn KnowledgeSpaceAppService>,
    drive_import: Arc<dyn KnowledgeDriveImportAppService>,
    ingest: Arc<dyn KnowledgeIngestAppService>,
    document: Arc<dyn KnowledgeDocumentAppService>,
    wiki: Arc<dyn KnowledgeWikiAppService>,
    browser: Arc<dyn KnowledgeBrowserApi>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    agent: Arc<dyn KnowledgeAgentAppService>,
) -> Router {
    build_router_with_shared_app_api(Arc::new(FullAppApi::new(
        space,
        drive_import,
        ingest,
        document,
        wiki,
        browser,
        retrieval,
        agent,
    )))
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
        .route("/app/v3/api/knowledge/retrievals", post(create_retrieval))
        .route(
            "/app/v3/api/knowledge/retrievals/:retrieval_id",
            get(retrieve_retrieval),
        )
        .route(
            "/app/v3/api/knowledge/context_packs",
            post(create_context_pack),
        )
        .route(
            "/app/v3/api/knowledge/agent_profiles",
            post(create_agent_profile),
        )
        .route(
            "/app/v3/api/knowledge/agent_profiles/:profile_id",
            get(retrieve_agent_profile)
                .patch(update_agent_profile)
                .delete(delete_agent_profile),
        )
        .route(
            "/app/v3/api/knowledge/agent_profiles/:profile_id/bindings",
            get(list_agent_profile_bindings).post(create_agent_profile_binding),
        )
        .route(
            "/app/v3/api/knowledge/agent_profiles/:profile_id/bindings/:binding_id",
            patch(update_agent_profile_binding).delete(delete_agent_profile_binding),
        )
        .route(
            "/app/v3/api/knowledge/agent_profiles/:profile_id/retrieval_preview",
            post(create_agent_profile_retrieval_preview),
        )
        .with_state(AppState { api })
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

async fn create_retrieval(
    State(state): State<AppState>,
    Json(request): Json<KnowledgeRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_retrieval(request).await)
}

async fn retrieve_retrieval(
    State(state): State<AppState>,
    Path(retrieval_id): Path<u64>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = context.map(|Extension(context)| context).ok_or_else(|| {
        ApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "missing_app_request_context",
            "authenticated app request context is required",
        )
    })?;
    ok_json(state.api.retrieve_retrieval(context, retrieval_id).await)
}

async fn create_context_pack(
    State(state): State<AppState>,
    Json(request): Json<KnowledgeContextPackRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_context_pack(request).await)
}

async fn create_agent_profile(
    State(state): State<AppState>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    created_json(state.api.create_agent_profile(request).await)
}

async fn retrieve_agent_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.retrieve_agent_profile(profile_id).await)
}

async fn update_agent_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.update_agent_profile(profile_id, request).await)
}

async fn delete_agent_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    state
        .api
        .delete_agent_profile(profile_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_agent_profile_bindings(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    ok_json(state.api.list_agent_profile_bindings(profile_id).await)
}

async fn create_agent_profile_binding(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentBindingRequest>,
) -> Result<Response, ApiProblem> {
    if request.profile_id != profile_id {
        return Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "profile_id_mismatch",
            "profileId in body must match profileId in path",
        ));
    }
    created_json(
        state
            .api
            .create_agent_profile_binding(profile_id, request)
            .await,
    )
}

async fn update_agent_profile_binding(
    State(state): State<AppState>,
    Path((profile_id, binding_id)): Path<(u64, u64)>,
    Json(request): Json<KnowledgeAgentBindingRequest>,
) -> Result<Response, ApiProblem> {
    if request.profile_id != profile_id {
        return Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "profile_id_mismatch",
            "profileId in body must match profileId in path",
        ));
    }
    ok_json(
        state
            .api
            .update_agent_profile_binding(profile_id, binding_id, request)
            .await,
    )
}

async fn delete_agent_profile_binding(
    State(state): State<AppState>,
    Path((profile_id, binding_id)): Path<(u64, u64)>,
) -> Result<Response, ApiProblem> {
    state
        .api
        .delete_agent_profile_binding(profile_id, binding_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn create_agent_profile_retrieval_preview(
    State(state): State<AppState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    created_json(
        state
            .api
            .create_agent_profile_retrieval_preview(profile_id, request)
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
