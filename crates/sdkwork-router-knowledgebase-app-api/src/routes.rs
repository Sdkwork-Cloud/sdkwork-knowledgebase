use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Extension, Json, Router,
};
use sdkwork_knowledgebase_contract::{
    context_binding::{
        CreateKnowledgeSpaceContextBindingRequest, UpdateKnowledgeSpaceContextBindingRequest,
    },
    upload::{CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest},
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest,
    KnowledgeAgentProfileRequest, KnowledgeBrowserView, KnowledgeContextPackRequest,
    KnowledgeDriveImportRequest, KnowledgeIngestRequest, KnowledgeRetrievalRequest,
    ListKnowledgeBrowserRequest, ListOkfConceptsQuery, OkfBundleExportRequest,
    OkfBundleImportRequest, OkfConceptUpsertRequest, OkfContextPackRequest, OkfFileAnswerRequest,
    OkfQualityRunRequest, OkfQueryRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    adapters::{
        AgentAndRetrievalAppApi, AgentOnlyAppApi, BrowserOnlyAppApi, FullAppApi,
        RetrievalOnlyAppApi,
    },
    auth::require_app_context,
    paths, ApiProblem, ApiResult, KnowledgeAgentAppService, KnowledgeAppApi,
    KnowledgeAppRequestContext, KnowledgeBrowserApi, KnowledgeDocumentAppService,
    KnowledgeDriveImportAppService, KnowledgeIngestAppService, KnowledgeOkfAppService,
    KnowledgeRetrievalAppService, KnowledgeSpaceAppService,
};

#[derive(Clone)]
pub struct ReadinessCheck {
    pool: sqlx::AnyPool,
}

impl ReadinessCheck {
    pub fn new(pool: sqlx::AnyPool) -> Self {
        Self { pool }
    }

    pub async fn check(&self) -> Result<(), sqlx::Error> {
        sdkwork_intelligence_knowledgebase_repository_sqlx::knowledgebase_health_check(&self.pool)
            .await
    }
}

#[derive(Clone)]
struct AppState {
    api: Arc<dyn KnowledgeAppApi>,
    readiness: Option<ReadinessCheck>,
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
    okf: Arc<dyn KnowledgeOkfAppService>,
    browser: Arc<dyn KnowledgeBrowserApi>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    agent: Arc<dyn KnowledgeAgentAppService>,
    context_binding: Arc<dyn crate::KnowledgeContextBindingAppService>,
    upload_session: Arc<dyn crate::KnowledgeUploadSessionAppService>,
) -> Router {
    build_router_with_shared_app_api(Arc::new(FullAppApi::new(
        space,
        drive_import,
        ingest,
        document,
        okf,
        browser,
        retrieval,
        agent,
        context_binding,
        upload_session,
    )))
}

pub fn build_router_with_shared_app_api(api: Arc<dyn KnowledgeAppApi>) -> Router {
    build_router_with_shared_app_api_and_readiness(api, None)
}

pub fn build_router_with_shared_app_api_and_readiness(
    api: Arc<dyn KnowledgeAppApi>,
    readiness: Option<ReadinessCheck>,
) -> Router {
    Router::new()
        .route(paths::HEALTHZ, get(health))
        .route(paths::SPACES, post(create_space))
        .route(paths::SPACE, get(retrieve_space))
        .route(paths::DRIVE_IMPORTS, post(create_drive_import))
        .route(paths::INGESTS, post(create_ingest))
        .route(paths::INGEST, get(retrieve_ingest))
        .route(paths::DOCUMENTS, get(list_documents).post(create_document))
        .route(
            paths::DOCUMENT,
            get(retrieve_document)
                .patch(update_document)
                .delete(delete_document),
        )
        .route(
            paths::DOCUMENT_VERSIONS,
            get(list_document_versions).post(create_document_version),
        )
        .route(paths::OKF_CONCEPTS, get(list_okf_concepts))
        .route(paths::OKF_CONCEPT_UPSERT, put(upsert_okf_concept))
        .route(paths::OKF_CONCEPT, get(retrieve_okf_concept))
        .route(
            paths::OKF_CONCEPT_REVISIONS,
            get(list_okf_concept_revisions),
        )
        .route(paths::OKF_INDEX, get(retrieve_okf_index))
        .route(paths::OKF_LOG, get(retrieve_okf_log))
        .route(paths::OKF_PROFILE, get(retrieve_okf_schema))
        .route(paths::OKF_QUERIES, post(create_okf_query))
        .route(paths::OKF_QUERY_FILE_ANSWER, post(file_okf_query_answer))
        .route(paths::OKF_CONTEXT_PACKS, post(create_okf_context_pack))
        .route(paths::OKF_EXPORTS, post(create_okf_export))
        .route(paths::OKF_EXPORT, get(retrieve_okf_export))
        .route(paths::OKF_IMPORTS, post(create_okf_import))
        .route(paths::OKF_LINT_RUNS, post(create_okf_lint_run))
        .route(paths::SPACE_BROWSER, get(list_browser))
        .route(paths::RETRIEVALS, post(create_retrieval))
        .route(paths::RETRIEVAL, get(retrieve_retrieval))
        .route(paths::CONTEXT_PACKS, post(create_context_pack))
        .route(paths::AGENT_PROFILES, post(create_agent_profile))
        .route(
            paths::AGENT_PROFILE,
            get(retrieve_agent_profile)
                .patch(update_agent_profile)
                .delete(delete_agent_profile),
        )
        .route(
            paths::AGENT_PROFILE_BINDINGS,
            get(list_agent_profile_bindings).post(create_agent_profile_binding),
        )
        .route(
            paths::AGENT_PROFILE_BINDING,
            patch(update_agent_profile_binding).delete(delete_agent_profile_binding),
        )
        .route(
            paths::AGENT_PROFILE_RETRIEVAL_PREVIEW,
            post(create_agent_profile_retrieval_preview),
        )
        .route(paths::AGENT_PROFILE_CHAT, post(create_agent_profile_chat))
        .route(
            paths::SPACE_CONTEXT_BINDINGS,
            get(list_space_context_bindings).post(create_space_context_binding),
        )
        .route(
            paths::CONTEXT_BINDING,
            get(retrieve_context_binding)
                .patch(update_context_binding)
                .delete(delete_context_binding),
        )
        .route(paths::UPLOAD_SESSIONS, post(create_upload_session))
        .route(
            paths::UPLOAD_SESSION_COMPLETE,
            post(complete_upload_session),
        )
        .with_state(AppState { api, readiness })
}

async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiProblem> {
    if let Some(readiness) = &state.readiness {
        readiness.check().await.map_err(|error| {
            ApiProblem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "database_unavailable",
                error.to_string(),
            )
        })?;
    }
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn create_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<CreateKnowledgeSpaceRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_space(request).await)
}

async fn retrieve_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_space(space_id).await)
}

async fn create_drive_import(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeDriveImportRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_drive_import(request).await)
}

async fn create_ingest(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeIngestRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_ingest(request).await)
}

async fn retrieve_ingest(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(ingest_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_ingest(ingest_id).await)
}

async fn list_documents(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.list_documents().await)
}

async fn create_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_document(request).await)
}

async fn retrieve_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_document(document_id).await)
}

async fn update_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.update_document(document_id, request).await)
}

async fn delete_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    state
        .api
        .delete_document(document_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_document_versions(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.list_document_versions(document_id).await)
}

async fn create_document_version(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentVersionRequest>,
) -> Result<Response, ApiProblem> {
    let _context = require_app_context(context)?;
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

async fn list_okf_concepts(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListOkfConceptsQuery>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.list_okf_concepts(query.space_id).await)
}

async fn upsert_okf_concept(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfConceptUpsertRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.upsert_okf_concept(request).await)
}

async fn retrieve_okf_concept(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(concept_row_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_okf_concept(concept_row_id).await)
}

async fn list_okf_concept_revisions(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(concept_row_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.list_okf_concept_revisions(concept_row_id).await)
}

async fn retrieve_okf_index(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_okf_index().await)
}

async fn retrieve_okf_log(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_okf_log().await)
}

async fn retrieve_okf_schema(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_okf_schema().await)
}

async fn create_okf_query(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfQueryRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_okf_query(request).await)
}

async fn file_okf_query_answer(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(query_id): Path<u64>,
    Json(request): Json<OkfFileAnswerRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.file_okf_query_answer(query_id, request).await)
}

async fn create_okf_context_pack(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfContextPackRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_okf_context_pack(request).await)
}

async fn create_okf_export(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfBundleExportRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_okf_export(request).await)
}

async fn retrieve_okf_export(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(export_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_okf_export(export_id).await)
}

async fn create_okf_import(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfBundleImportRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_okf_import(request).await)
}

async fn create_okf_lint_run(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_okf_lint_run(request).await)
}

async fn list_browser(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Query(query): Query<ListBrowserQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
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

async fn create_retrieval(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_retrieval(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn retrieve_retrieval(
    State(state): State<AppState>,
    Path(retrieval_id): Path<u64>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_retrieval(context, retrieval_id).await)
}

async fn create_context_pack(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeContextPackRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_context_pack(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn create_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_agent_profile(request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn retrieve_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.retrieve_agent_profile(profile_id).await)
}

async fn update_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .update_agent_profile(profile_id, request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn delete_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    state
        .api
        .delete_agent_profile(profile_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_agent_profile_bindings(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    ok_json(state.api.list_agent_profile_bindings(profile_id).await)
}

async fn create_agent_profile_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentBindingRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
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
            .create_agent_profile_binding(profile_id, request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn update_agent_profile_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path((profile_id, binding_id)): Path<(u64, u64)>,
    Json(request): Json<KnowledgeAgentBindingRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
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
            .update_agent_profile_binding(
                profile_id,
                binding_id,
                request.with_tenant_id(context.tenant_id),
            )
            .await,
    )
}

async fn delete_agent_profile_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path((profile_id, binding_id)): Path<(u64, u64)>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    state
        .api
        .delete_agent_profile_binding(profile_id, binding_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn create_agent_profile_retrieval_preview(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeRetrievalRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_agent_profile_retrieval_preview(
                profile_id,
                request.with_tenant_id(context.tenant_id),
            )
            .await,
    )
}

async fn create_agent_profile_chat(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentChatRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .create_agent_chat(profile_id, request.with_tenant_id(context.tenant_id))
            .await,
    )
}

async fn list_space_context_bindings(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_space_context_bindings(context, space_id)
            .await,
    )
}

async fn create_space_context_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Json(request): Json<CreateKnowledgeSpaceContextBindingRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    if request.space_id != space_id {
        return Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "space_id_mismatch",
            "spaceId in body must match spaceId in path",
        ));
    }
    created_json(
        state
            .api
            .create_space_context_binding(context, space_id, request)
            .await,
    )
}

async fn retrieve_context_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(binding_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_context_binding(context, binding_id)
            .await,
    )
}

async fn update_context_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(binding_id): Path<u64>,
    Json(request): Json<UpdateKnowledgeSpaceContextBindingRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .update_context_binding(context, binding_id, request)
            .await,
    )
}

async fn delete_context_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(binding_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_context_binding(context, binding_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn create_upload_session(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<CreateKnowledgeUploadSessionRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.create_upload_session(request).await)
}

async fn complete_upload_session(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(session_id): Path<u64>,
    Json(request): Json<CompleteKnowledgeUploadSessionRequest>,
) -> Result<Response, ApiProblem> {
    require_app_context(context)?;
    created_json(state.api.complete_upload_session(session_id, request).await)
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
        "okf_bundle" => Ok(KnowledgeBrowserView::OkfBundle),
        "outputs" => Ok(KnowledgeBrowserView::Outputs),
        value => Err(ApiProblem::new(
            StatusCode::BAD_REQUEST,
            "invalid_browser_view",
            format!("unsupported browser view: {value}"),
        )),
    }
}
