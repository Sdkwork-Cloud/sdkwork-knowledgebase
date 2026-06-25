use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use sdkwork_knowledgebase_contract::{
    context_binding::{
        CreateKnowledgeSpaceContextBindingRequest, UpdateKnowledgeSpaceContextBindingRequest,
    },
    upload::{CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest},
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, GrantKnowledgeSpaceMemberRequest, KnowledgeAgentBindingRequest,
    KnowledgeAgentChatRequest, KnowledgeAgentProfileRequest, KnowledgeBrowserView,
    KnowledgeContextPackRequest, KnowledgeDriveImportRequest, KnowledgeGitImportRequest,
    KnowledgeGitSyncRequest, KnowledgeIngestRequest, KnowledgeMarketSubscriptionRequest,
    KnowledgeMediaTaskRequest, KnowledgeRetrievalRequest, KnowledgeSiteDeploymentRequest,
    KnowledgeSpaceMemberSubjectType, KnowledgeWechatArticlesPreviewRequest,
    KnowledgeWechatArticlesPublishRequest, KnowledgeWechatReplaceAppletsRequest,
    KnowledgeWechatReplaceOfficialAccountsRequest, ListKnowledgeBrowserRequest,
    ListOkfConceptsQuery, OkfBundleExportRequest, OkfBundleImportRequest, OkfConceptUpsertRequest,
    OkfContextPackRequest, OkfFileAnswerRequest, OkfQualityRunRequest, OkfQueryRequest,
    UpdateKnowledgeSpaceRequest,
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
    KnowledgeAppRequestContext, KnowledgeBrowserApi, KnowledgeCommerceAppService,
    KnowledgeDocumentAppService, KnowledgeDriveImportAppService, KnowledgeGitImportAppService,
    KnowledgeIngestAppService, KnowledgeOkfAppService, KnowledgeRetrievalAppService,
    KnowledgeSpaceAppService,
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
    git_import: Arc<dyn KnowledgeGitImportAppService>,
    ingest: Arc<dyn KnowledgeIngestAppService>,
    document: Arc<dyn KnowledgeDocumentAppService>,
    okf: Arc<dyn KnowledgeOkfAppService>,
    browser: Arc<dyn KnowledgeBrowserApi>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    agent: Arc<dyn KnowledgeAgentAppService>,
    context_binding: Arc<dyn crate::KnowledgeContextBindingAppService>,
    upload_session: Arc<dyn crate::KnowledgeUploadSessionAppService>,
    wechat: Arc<dyn crate::KnowledgeWechatAppService>,
    commerce: Arc<dyn KnowledgeCommerceAppService>,
) -> Router {
    build_router_with_shared_app_api(Arc::new(FullAppApi::new(
        space,
        drive_import,
        git_import,
        ingest,
        document,
        okf,
        browser,
        retrieval,
        agent,
        context_binding,
        upload_session,
        wechat,
        commerce,
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
        .route(paths::LIVEZ, get(livez))
        .route(paths::READYZ, get(readyz))
        .route(paths::HEALTHZ, get(health))
        .route(paths::SPACES, post(create_space))
        .route(
            paths::SPACE,
            get(retrieve_space).patch(update_space).delete(delete_space),
        )
        .route(paths::DRIVE_IMPORTS, post(create_drive_import))
        .route(paths::GIT_IMPORTS, post(create_git_import))
        .route(paths::GIT_SYNCS, post(create_git_sync))
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
        .route(paths::DOCUMENT_CONTENT, get(retrieve_document_content))
        .route(paths::OKF_CONCEPTS, get(list_okf_concepts))
        .route(paths::OKF_CONCEPT_UPSERT, put(upsert_okf_concept))
        .route(
            paths::OKF_CONCEPT,
            get(retrieve_okf_concept).delete(delete_okf_concept),
        )
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
            paths::SPACE_MEMBERS,
            get(list_space_members)
                .post(grant_space_member)
                .delete(revoke_space_member),
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
        .route(
            paths::WECHAT_OFFICIAL_ACCOUNTS,
            get(list_wechat_official_accounts).put(replace_wechat_official_accounts),
        )
        .route(
            paths::WECHAT_APPLETS,
            get(list_wechat_applets).put(replace_wechat_applets),
        )
        .route(
            paths::WECHAT_ARTICLES_PUBLISH,
            post(publish_wechat_articles),
        )
        .route(
            paths::WECHAT_ARTICLES_PREVIEW,
            post(preview_wechat_articles),
        )
        .route(paths::MARKET_LISTINGS, get(list_market_listings))
        .route(
            paths::MARKET_SUBSCRIPTIONS,
            post(create_market_subscription),
        )
        .route(
            paths::MARKET_SUBSCRIPTION,
            delete(delete_market_subscription),
        )
        .route(paths::SITE_DEPLOYMENTS, post(create_site_deployment))
        .route(
            paths::SITE_DEPLOYMENT_PREVIEW,
            get(retrieve_site_deployment_preview),
        )
        .route(paths::MEDIA_TASKS, post(create_media_task))
        .with_state(AppState { api, readiness })
}

async fn livez() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiProblem> {
    if let Some(readiness) = &state.readiness {
        readiness.check().await.map_err(|error| {
            sdkwork_knowledgebase_observability::set_readiness_status(false);
            eprintln!("[knowledgebase-app-api] readiness check failed: {error}");
            ApiProblem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "dependencies_unavailable",
                "One or more dependencies are unavailable.",
            )
        })?;
    }
    sdkwork_knowledgebase_observability::set_readiness_status(true);
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiProblem> {
    readyz(State(state)).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListDocumentsQuery {
    space_id: u64,
}

async fn create_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<CreateKnowledgeSpaceRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_space(context, request).await)
}

async fn retrieve_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_space(context, space_id).await)
}

async fn update_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Json(request): Json<UpdateKnowledgeSpaceRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.update_space(context, space_id, request).await)
}

async fn delete_space(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_space(context, space_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevokeSpaceMemberQuery {
    subject_type: KnowledgeSpaceMemberSubjectType,
    subject_id: String,
}

async fn list_space_members(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Query(query): Query<ListSpaceMembersQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_space_members(context, space_id, query.cursor, query.page_size)
            .await,
    )
}

async fn grant_space_member(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Json(request): Json<GrantKnowledgeSpaceMemberRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .grant_space_member(context, space_id, request)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn revoke_space_member(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(space_id): Path<u64>,
    Query(query): Query<RevokeSpaceMemberQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .revoke_space_member(context, space_id, query.subject_type, query.subject_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn create_drive_import(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeDriveImportRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_drive_import(context, request).await)
}

async fn create_git_import(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeGitImportRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_git_import(context, request).await)
}

async fn create_git_sync(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeGitSyncRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_git_sync(context, request).await)
}

async fn list_wechat_official_accounts(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_wechat_official_accounts(context).await)
}

async fn replace_wechat_official_accounts(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeWechatReplaceOfficialAccountsRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .replace_wechat_official_accounts(context, request)
            .await,
    )
}

async fn list_wechat_applets(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_wechat_applets(context).await)
}

async fn replace_wechat_applets(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeWechatReplaceAppletsRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.replace_wechat_applets(context, request).await)
}

async fn publish_wechat_articles(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeWechatArticlesPublishRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.publish_wechat_articles(context, request).await)
}

async fn preview_wechat_articles(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeWechatArticlesPreviewRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.preview_wechat_articles(context, request).await)
}

async fn list_market_listings(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_market_listings(context).await)
}

async fn create_market_subscription(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeMarketSubscriptionRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_market_subscription(context, request).await)
}

async fn delete_market_subscription(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(listing_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .delete_market_subscription(context, listing_id)
            .await,
    )
}

async fn create_site_deployment(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeSiteDeploymentRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_site_deployment(context, request).await)
}

async fn retrieve_site_deployment_preview(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(deployment_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_site_deployment_preview(context, deployment_id)
            .await,
    )
}

async fn create_media_task(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeMediaTaskRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_media_task(context, request).await)
}

async fn create_ingest(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeIngestRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_ingest(context, request).await)
}

async fn retrieve_ingest(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(ingest_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_ingest(context, ingest_id).await)
}

async fn list_documents(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_documents(context, query.space_id).await)
}

async fn create_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_document(context, request).await)
}

async fn retrieve_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_document(context, document_id).await)
}

async fn retrieve_document_content(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_document_content(context, document_id)
            .await,
    )
}

async fn update_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .update_document(context, document_id, request)
            .await,
    )
}

async fn delete_document(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_document(context, document_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_document_versions(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_document_versions(context, document_id).await)
}

async fn create_document_version(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(document_id): Path<u64>,
    Json(request): Json<CreateKnowledgeDocumentVersionRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
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
            .create_document_version(context, document_id, request)
            .await,
    )
}

async fn list_okf_concepts(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListOkfConceptsQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.list_okf_concepts(context, query.space_id).await)
}

async fn upsert_okf_concept(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfConceptUpsertRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.upsert_okf_concept(context, request).await)
}

async fn retrieve_okf_concept(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(concept_row_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .retrieve_okf_concept(context, concept_row_id)
            .await,
    )
}

async fn delete_okf_concept(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(concept_row_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_okf_concept(context, concept_row_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_okf_concept_revisions(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(concept_row_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_okf_concept_revisions(context, concept_row_id)
            .await,
    )
}

async fn retrieve_okf_index(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListOkfConceptsQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_okf_index(context, query.space_id).await)
}

async fn retrieve_okf_log(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListOkfConceptsQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_okf_log(context, query.space_id).await)
}

async fn retrieve_okf_schema(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Query(query): Query<ListOkfConceptsQuery>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_okf_schema(context, query.space_id).await)
}

async fn create_okf_query(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfQueryRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_okf_query(context, request).await)
}

async fn file_okf_query_answer(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(query_id): Path<u64>,
    Json(request): Json<OkfFileAnswerRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(
        state
            .api
            .file_okf_query_answer(context, query_id, request)
            .await,
    )
}

async fn create_okf_context_pack(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfContextPackRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_okf_context_pack(context, request).await)
}

async fn create_okf_export(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfBundleExportRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_okf_export(context, request).await)
}

async fn retrieve_okf_export(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(export_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_okf_export(context, export_id).await)
}

async fn create_okf_import(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfBundleImportRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_okf_import(context, request).await)
}

async fn create_okf_lint_run(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<OkfQualityRunRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    created_json(state.api.create_okf_lint_run(context, request).await)
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
    let tenant_id = context.tenant_id;
    let actor_id = context.actor_id;
    created_json(
        state
            .api
            .create_retrieval(
                context,
                request.with_tenant_id(tenant_id).with_actor_id(actor_id),
            )
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
    let tenant_id = context.tenant_id;
    let actor_id = context.actor_id;
    created_json(
        state
            .api
            .create_context_pack(
                context,
                request.with_tenant_id(tenant_id).with_actor_id(actor_id),
            )
            .await,
    )
}

async fn create_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    let tenant_id = context.tenant_id;
    created_json(
        state
            .api
            .create_agent_profile(context, request.with_tenant_id(tenant_id))
            .await,
    )
}

async fn retrieve_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(state.api.retrieve_agent_profile(context, profile_id).await)
}

async fn update_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentProfileRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    let tenant_id = context.tenant_id;
    ok_json(
        state
            .api
            .update_agent_profile(context, profile_id, request.with_tenant_id(tenant_id))
            .await,
    )
}

async fn delete_agent_profile(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_agent_profile(context, profile_id)
        .await
        .map_err(ApiProblem::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_agent_profile_bindings(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    ok_json(
        state
            .api
            .list_agent_profile_bindings(context, profile_id)
            .await,
    )
}

async fn create_agent_profile_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeAgentBindingRequest>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    let tenant_id = context.tenant_id;
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
            .create_agent_profile_binding(context, profile_id, request.with_tenant_id(tenant_id))
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
    let tenant_id = context.tenant_id;
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
                context,
                profile_id,
                binding_id,
                request.with_tenant_id(tenant_id),
            )
            .await,
    )
}

async fn delete_agent_profile_binding(
    State(state): State<AppState>,
    context: Option<Extension<KnowledgeAppRequestContext>>,
    Path((profile_id, binding_id)): Path<(u64, u64)>,
) -> Result<Response, ApiProblem> {
    let context = require_app_context(context)?;
    state
        .api
        .delete_agent_profile_binding(context, profile_id, binding_id)
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
    let tenant_id = context.tenant_id;
    created_json(
        state
            .api
            .create_agent_profile_retrieval_preview(
                context,
                profile_id,
                request.with_tenant_id(tenant_id),
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
    let tenant_id = context.tenant_id;
    created_json(
        state
            .api
            .create_agent_chat(context, profile_id, request.with_tenant_id(tenant_id))
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
struct ListSpaceMembersQuery {
    cursor: Option<String>,
    page_size: Option<u32>,
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
