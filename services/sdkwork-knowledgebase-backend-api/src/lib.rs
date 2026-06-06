use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, IngestionJob, KnowledgeSource, KnowledgeSourceList,
    KnowledgeWikiFileEntry, KnowledgeWikiFileEntryList, KnowledgeWikiSchemaProfileRequest,
    ProblemDetails, WikiCandidateResult, WikiCandidateResultList, WikiCandidateReviewRequest,
    WikiCompileJobRequest, WikiExportRequest, WikiIndexDocument, WikiIndexRebuildRequest,
    WikiLogEntry, WikiPagePublishRequest, WikiPageSummary, WikiQualityRun, WikiQualityRunRequest,
};
use serde::Serialize;
use std::sync::Arc;

pub type BackendApiResult<T> = Result<T, BackendApiError>;

#[derive(Debug, Clone)]
pub struct BackendApiError {
    status: StatusCode,
    code: String,
    detail: String,
}

impl BackendApiError {
    pub fn new(status: StatusCode, code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
        }
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
pub struct BackendApiProblem {
    status: StatusCode,
    problem: Box<ProblemDetails>,
}

impl BackendApiProblem {
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

impl From<BackendApiError> for BackendApiProblem {
    fn from(error: BackendApiError) -> Self {
        Self::new(error.status, error.code, error.detail)
    }
}

impl IntoResponse for BackendApiProblem {
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
pub trait KnowledgeBackendApi: Send + Sync + 'static {
    async fn list_sources(&self) -> BackendApiResult<KnowledgeSourceList> {
        Err(BackendApiError::not_implemented("sources.list"))
    }

    async fn create_source(
        &self,
        _request: CreateKnowledgeSourceRequest,
    ) -> BackendApiResult<KnowledgeSource> {
        Err(BackendApiError::not_implemented("sources.create"))
    }

    async fn create_wiki_compile_job(
        &self,
        _request: WikiCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        Err(BackendApiError::not_implemented("wiki.compileJobs.create"))
    }

    async fn list_wiki_candidates(&self) -> BackendApiResult<WikiCandidateResultList> {
        Err(BackendApiError::not_implemented("wiki.candidates.list"))
    }

    async fn approve_wiki_candidate(
        &self,
        _candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        Err(BackendApiError::not_implemented("wiki.candidates.approve"))
    }

    async fn reject_wiki_candidate(
        &self,
        _candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        Err(BackendApiError::not_implemented("wiki.candidates.reject"))
    }

    async fn publish_wiki_page(
        &self,
        _page_id: u64,
        _request: WikiPagePublishRequest,
    ) -> BackendApiResult<WikiPageSummary> {
        Err(BackendApiError::not_implemented("wiki.pages.publish"))
    }

    async fn create_wiki_schema_profile(
        &self,
        _request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented(
            "wiki.schema.profiles.create",
        ))
    }

    async fn update_wiki_schema_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented(
            "wiki.schema.profiles.update",
        ))
    }

    async fn rebuild_wiki_index(
        &self,
        _request: WikiIndexRebuildRequest,
    ) -> BackendApiResult<WikiIndexDocument> {
        Err(BackendApiError::not_implemented("wiki.index.rebuild"))
    }

    async fn create_wiki_log_entry(
        &self,
        _request: WikiLogEntry,
    ) -> BackendApiResult<WikiLogEntry> {
        Err(BackendApiError::not_implemented("wiki.log.entries.create"))
    }

    async fn create_wiki_export(
        &self,
        _request: WikiExportRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented("wiki.exports.create"))
    }

    async fn retrieve_wiki_export(
        &self,
        _export_id: u64,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented("wiki.exports.retrieve"))
    }

    async fn list_wiki_file_entries(&self) -> BackendApiResult<KnowledgeWikiFileEntryList> {
        Err(BackendApiError::not_implemented("wiki.fileEntries.list"))
    }

    async fn create_wiki_lint_run(
        &self,
        _request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        Err(BackendApiError::not_implemented("wiki.lintRuns.create"))
    }

    async fn create_wiki_eval_run(
        &self,
        _request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        Err(BackendApiError::not_implemented("wiki.evalRuns.create"))
    }
}

#[derive(Clone)]
struct BackendState {
    api: Arc<dyn KnowledgeBackendApi>,
}

pub fn build_router_with_backend_api<A>(api: A) -> Router
where
    A: KnowledgeBackendApi,
{
    build_router_with_shared_backend_api(Arc::new(api))
}

pub fn build_router_with_shared_backend_api(api: Arc<dyn KnowledgeBackendApi>) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route(
            "/backend/v3/api/knowledge/sources",
            get(list_sources).post(create_source),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_compile_jobs",
            post(create_wiki_compile_job),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates",
            get(list_wiki_candidates),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/approve",
            post(approve_wiki_candidate),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/reject",
            post(reject_wiki_candidate),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_pages/:page_id/publish",
            post(publish_wiki_page),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_schema_profiles",
            post(create_wiki_schema_profile),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_schema_profiles/:profile_id",
            patch(update_wiki_schema_profile),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_index/rebuild",
            post(rebuild_wiki_index),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_log_entries",
            post(create_wiki_log_entry),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_exports",
            post(create_wiki_export),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_exports/:export_id",
            get(retrieve_wiki_export),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_file_entries",
            get(list_wiki_file_entries),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_lint_runs",
            post(create_wiki_lint_run),
        )
        .route(
            "/backend/v3/api/knowledge/wiki_eval_runs",
            post(create_wiki_eval_run),
        )
        .with_state(BackendState { api })
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_sources(State(state): State<BackendState>) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_sources().await)
}

async fn create_source(
    State(state): State<BackendState>,
    Json(request): Json<CreateKnowledgeSourceRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_source(request).await)
}

async fn create_wiki_compile_job(
    State(state): State<BackendState>,
    Json(request): Json<WikiCompileJobRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_compile_job(request).await)
}

async fn list_wiki_candidates(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_wiki_candidates().await)
}

async fn approve_wiki_candidate(
    State(state): State<BackendState>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(
        state
            .api
            .approve_wiki_candidate(candidate_id, request)
            .await,
    )
}

async fn reject_wiki_candidate(
    State(state): State<BackendState>,
    Path(candidate_id): Path<u64>,
    Json(request): Json<WikiCandidateReviewRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.reject_wiki_candidate(candidate_id, request).await)
}

async fn publish_wiki_page(
    State(state): State<BackendState>,
    Path(page_id): Path<u64>,
    Json(request): Json<WikiPagePublishRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.publish_wiki_page(page_id, request).await)
}

async fn create_wiki_schema_profile(
    State(state): State<BackendState>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_schema_profile(request).await)
}

async fn update_wiki_schema_profile(
    State(state): State<BackendState>,
    Path(profile_id): Path<u64>,
    Json(request): Json<KnowledgeWikiSchemaProfileRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(
        state
            .api
            .update_wiki_schema_profile(profile_id, request)
            .await,
    )
}

async fn rebuild_wiki_index(
    State(state): State<BackendState>,
    Json(request): Json<WikiIndexRebuildRequest>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.rebuild_wiki_index(request).await)
}

async fn create_wiki_log_entry(
    State(state): State<BackendState>,
    Json(request): Json<WikiLogEntry>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_log_entry(request).await)
}

async fn create_wiki_export(
    State(state): State<BackendState>,
    Json(request): Json<WikiExportRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_export(request).await)
}

async fn retrieve_wiki_export(
    State(state): State<BackendState>,
    Path(export_id): Path<u64>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.retrieve_wiki_export(export_id).await)
}

async fn list_wiki_file_entries(
    State(state): State<BackendState>,
) -> Result<Response, BackendApiProblem> {
    ok_json(state.api.list_wiki_file_entries().await)
}

async fn create_wiki_lint_run(
    State(state): State<BackendState>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_lint_run(request).await)
}

async fn create_wiki_eval_run(
    State(state): State<BackendState>,
    Json(request): Json<WikiQualityRunRequest>,
) -> Result<Response, BackendApiProblem> {
    created_json(state.api.create_wiki_eval_run(request).await)
}

fn ok_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| Json(value).into_response())
        .map_err(BackendApiProblem::from)
}

fn created_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| (StatusCode::CREATED, Json(value)).into_response())
        .map_err(BackendApiProblem::from)
}
