use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_knowledgebase_contract::ProblemDetails;
use sdkwork_knowledgebase_product::agent::KnowledgeAgentServiceError;
use sdkwork_knowledgebase_product::ports::knowledge_agent_profile_store::KnowledgeAgentProfileStoreError;
use sdkwork_knowledgebase_product::ports::knowledge_memory_context::KnowledgeMemoryContextProviderError;
use sdkwork_knowledgebase_product::ports::knowledge_retrieval_backend::KnowledgeRetrievalBackendError;
use sdkwork_knowledgebase_product::ports::knowledge_retrieval_trace_store::KnowledgeRetrievalTraceStoreError;
use sdkwork_knowledgebase_product::retrieval::KnowledgeRetrievalServiceError;

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

    pub fn invalid_request(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, detail)
    }

    pub fn not_found(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, detail)
    }

    pub fn conflict(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, detail)
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

impl From<KnowledgeRetrievalServiceError> for ApiError {
    fn from(error: KnowledgeRetrievalServiceError) -> Self {
        match error {
            KnowledgeRetrievalServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_retrieval_request", detail)
            }
            KnowledgeRetrievalServiceError::TraceStore(
                KnowledgeRetrievalTraceStoreError::NotFound(retrieval_id),
            ) => Self::not_found(
                "knowledge_retrieval_not_found",
                format!("knowledge retrieval trace was not found: {retrieval_id}"),
            ),
            KnowledgeRetrievalServiceError::Backend(KnowledgeRetrievalBackendError::Internal(
                detail,
            ))
            | KnowledgeRetrievalServiceError::MemoryProvider(
                KnowledgeMemoryContextProviderError::Upstream(detail)
                | KnowledgeMemoryContextProviderError::Internal(detail),
            )
            | KnowledgeRetrievalServiceError::TraceStore(
                KnowledgeRetrievalTraceStoreError::Internal(detail),
            ) => Self::internal("knowledge_retrieval_failed", detail),
            KnowledgeRetrievalServiceError::MemoryProvider(
                KnowledgeMemoryContextProviderError::InvalidRequest(detail),
            ) => Self::invalid_request("invalid_knowledge_memory_context_request", detail),
        }
    }
}

impl From<KnowledgeAgentServiceError> for ApiError {
    fn from(error: KnowledgeAgentServiceError) -> Self {
        match error {
            KnowledgeAgentServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_agent_request", detail)
            }
            KnowledgeAgentServiceError::Store(KnowledgeAgentProfileStoreError::NotFound(id)) => {
                Self::not_found(
                    "knowledge_agent_profile_not_found",
                    format!("knowledge agent resource was not found: {id}"),
                )
            }
            KnowledgeAgentServiceError::Store(KnowledgeAgentProfileStoreError::Conflict(
                detail,
            )) => Self::conflict("knowledge_agent_profile_conflict", detail),
            KnowledgeAgentServiceError::Store(KnowledgeAgentProfileStoreError::Internal(
                detail,
            )) => Self::internal("knowledge_agent_profile_store_failed", detail),
            KnowledgeAgentServiceError::Retrieval(error) => Self::from(error),
        }
    }
}
