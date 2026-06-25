use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use sdkwork_knowledgebase_contract::{
    KnowledgeBrowserView, KnowledgeContextPackRequest, KnowledgeIngestRequest,
    KnowledgeRetrievalRequest, ListKnowledgeBrowserRequest,
};
use sdkwork_router_knowledgebase_backend_api::{health, DbReadinessCheck};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{paths, ApiProblem, ApiResult, KnowledgeOpenApi, KnowledgeOpenApiRequestContext};

#[derive(Clone)]
struct OpenState {
    api: Arc<dyn KnowledgeOpenApi>,
}

pub fn build_router_with_open_api<A>(api: A) -> Router
where
    A: KnowledgeOpenApi,
{
    build_router_with_shared_open_api(Arc::new(api))
}

pub fn build_router_with_shared_open_api(api: Arc<dyn KnowledgeOpenApi>) -> Router {
    build_router_with_shared_open_api_and_readiness(api, None)
}

pub fn build_router_with_shared_open_api_and_readiness(
    api: Arc<dyn KnowledgeOpenApi>,
    readiness: Option<DbReadinessCheck>,
) -> Router {
    let readiness_for_routes = readiness.clone();
    Router::new()
        .route(paths::LIVEZ, get(health::livez))
        .route(
            paths::READYZ,
            get({
                let readiness = readiness_for_routes.clone();
                move || async move {
                    map_backend_health_problem(health::readyz_with_state(readiness).await)
                }
            }),
        )
        .route(
            paths::HEALTHZ,
            get({
                let readiness = readiness_for_routes.clone();
                move || async move {
                    map_backend_health_problem(health::healthz_with_state(readiness).await)
                }
            }),
        )
        .route(paths::RETRIEVALS, post(create_retrieval))
        .route(paths::RETRIEVAL, get(retrieve_retrieval))
        .route(paths::CONTEXT_PACKS, post(create_context_pack))
        .route(paths::INGESTS, post(create_ingest))
        .route(paths::INGEST, get(retrieve_ingest))
        .route(paths::DOCUMENTS, get(list_documents))
        .route(paths::DOCUMENT, get(retrieve_document))
        .route(paths::SPACE_BROWSER, get(list_browser))
        .with_state(OpenState { api })
}

fn map_backend_health_problem(
    result: Result<
        Json<serde_json::Value>,
        sdkwork_router_knowledgebase_backend_api::BackendApiProblem,
    >,
) -> Result<Json<serde_json::Value>, ApiProblem> {
    result.map_err(|_| {
        ApiProblem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "dependencies_unavailable",
            "One or more dependencies are unavailable.",
        )
    })
}

async fn create_retrieval(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Json(request): Json<KnowledgeRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    let tenant_id = context.tenant_id;
    created_json(
        state
            .api
            .create_retrieval(context, request.with_tenant_id(tenant_id))
            .await,
    )
}

async fn retrieve_retrieval(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Path(retrieval_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    ok_json(state.api.retrieve_retrieval(context, retrieval_id).await)
}

async fn create_context_pack(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Json(request): Json<KnowledgeContextPackRequest>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    let tenant_id = context.tenant_id;
    created_json(
        state
            .api
            .create_context_pack(context, request.with_tenant_id(tenant_id))
            .await,
    )
}

async fn create_ingest(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Json(request): Json<KnowledgeIngestRequest>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    created_json(state.api.create_ingest(context, request).await)
}

async fn retrieve_ingest(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Path(ingest_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    ok_json(state.api.retrieve_ingest(context, ingest_id).await)
}

async fn list_documents(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    ok_json(state.api.list_documents(context, query.space_id).await)
}

async fn retrieve_document(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    ok_json(state.api.retrieve_document(context, document_id).await)
}

async fn list_browser(
    State(state): State<OpenState>,
    context: Option<Extension<KnowledgeOpenApiRequestContext>>,
    Path(space_id): Path<u64>,
    Query(query): Query<ListBrowserQuery>,
) -> Result<Response, ApiProblem> {
    let context = crate::auth::require_context(context)?;
    let view = parse_view(query.view.as_deref())?;
    ok_json(
        state
            .api
            .list_browser(
                context,
                ListKnowledgeBrowserRequest {
                    space_id,
                    parent_id: query.parent_id,
                    view,
                    cursor: query.cursor,
                    page_size: query.page_size,
                },
            )
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
struct ListDocumentsQuery {
    space_id: u64,
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
        "okf_bundle" => Ok(KnowledgeBrowserView::OkfBundle),
        "outputs" => Ok(KnowledgeBrowserView::Outputs),
        value => Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "invalid_browser_view",
            format!("unsupported browser view: {value}"),
        )),
    }
}
