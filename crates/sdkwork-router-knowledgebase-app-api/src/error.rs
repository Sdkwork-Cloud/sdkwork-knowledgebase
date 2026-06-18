use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeStorageError;
use sdkwork_intelligence_knowledgebase_service::{
    agent::KnowledgeAgentServiceError,
    agent_chat::KnowledgeAgentChatServiceError,
    browser::KnowledgeBrowserServiceError,
    imports::KnowledgeDriveImportServiceError,
    ingest::KnowledgeApiPayloadIngestServiceError,
    ports::{
        knowledge_agent_profile_store::KnowledgeAgentProfileStoreError,
        knowledge_document_store::KnowledgeDocumentStoreError,
        knowledge_ingestion_job_store::IngestionJobStoreError,
        knowledge_memory_context::KnowledgeMemoryContextProviderError,
        knowledge_retrieval_backend::KnowledgeRetrievalBackendError,
        knowledge_retrieval_trace_store::KnowledgeRetrievalTraceStoreError,
        knowledge_space_store::KnowledgeSpaceStoreError,
    },
    retrieval::KnowledgeRetrievalServiceError,
    space::KnowledgeSpaceServiceError,
    wiki::KnowledgeWikiPageServiceError,
};
use sdkwork_knowledgebase_contract::ProblemDetails;

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

    pub fn to_open_api_error(self) -> sdkwork_router_knowledgebase_open_api::ApiError {
        sdkwork_router_knowledgebase_open_api::ApiError::new(self.status, self.code, self.detail)
    }

    pub fn to_backend_api_error(self) -> sdkwork_router_knowledgebase_backend_api::BackendApiError {
        sdkwork_router_knowledgebase_backend_api::BackendApiError::new(
            self.status,
            self.code,
            self.detail,
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
            KnowledgeRetrievalServiceError::Backend(
                KnowledgeRetrievalBackendError::TenantMismatch,
            ) => Self::new(
                StatusCode::FORBIDDEN,
                "tenant_id_mismatch",
                "retrieval tenantId does not match backend tenant scope",
            ),
            KnowledgeRetrievalServiceError::Backend(
                KnowledgeRetrievalBackendError::UnsupportedMethod(method),
            ) => Self::invalid_request(
                "unsupported_retrieval_method",
                format!("retrieval method is not supported by the configured backend: {method:?}"),
            ),
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

impl From<KnowledgeAgentChatServiceError> for ApiError {
    fn from(error: KnowledgeAgentChatServiceError) -> Self {
        match error {
            KnowledgeAgentChatServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_agent_chat_request", detail)
            }
            KnowledgeAgentChatServiceError::Retrieval(error) => Self::from(error),
            KnowledgeAgentChatServiceError::KnowledgeProvider(detail) => {
                Self::internal("knowledge_agent_chat_provider_failed", detail)
            }
            KnowledgeAgentChatServiceError::Runtime(detail) => {
                Self::internal("knowledge_agent_chat_runtime_failed", detail)
            }
            KnowledgeAgentChatServiceError::AgentKernel(detail) => {
                Self::internal("knowledge_agent_chat_kernel_failed", detail)
            }
        }
    }
}

impl From<KnowledgeSpaceServiceError> for ApiError {
    fn from(error: KnowledgeSpaceServiceError) -> Self {
        match error {
            KnowledgeSpaceServiceError::InvalidRequest(detail)
            | KnowledgeSpaceServiceError::AccessDenied(detail) => {
                Self::invalid_request("invalid_knowledge_space_request", detail)
            }
            KnowledgeSpaceServiceError::Store(error) => Self::from(error),
            KnowledgeSpaceServiceError::WikiInitializer(error) => Self::internal(
                "knowledge_space_wiki_initialization_failed",
                error.to_string(),
            ),
            KnowledgeSpaceServiceError::DriveSpaceProvisioner(error) => Self::internal(
                "knowledge_space_drive_provisioning_failed",
                error.to_string(),
            ),
            KnowledgeSpaceServiceError::AccessControl(error) => {
                Self::internal("knowledge_space_access_control_failed", error.to_string())
            }
            KnowledgeSpaceServiceError::InitializationCleanup { original, .. }
            | KnowledgeSpaceServiceError::DriveSpaceCleanup { original, .. } => {
                Self::internal("knowledge_space_initialization_failed", original)
            }
        }
    }
}

impl From<KnowledgeSpaceStoreError> for ApiError {
    fn from(error: KnowledgeSpaceStoreError) -> Self {
        match error {
            KnowledgeSpaceStoreError::Conflict(detail) => {
                Self::conflict("knowledge_space_conflict", detail)
            }
            KnowledgeSpaceStoreError::Internal(detail) => {
                if detail.contains("missing knowledge space") {
                    Self::not_found("knowledge_space_not_found", detail)
                } else {
                    Self::internal("knowledge_space_store_failed", detail)
                }
            }
        }
    }
}

impl From<KnowledgeApiPayloadIngestServiceError> for ApiError {
    fn from(error: KnowledgeApiPayloadIngestServiceError) -> Self {
        match error {
            KnowledgeApiPayloadIngestServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_ingest_request", detail)
            }
            KnowledgeApiPayloadIngestServiceError::Store(error) => Self::from(error),
            KnowledgeApiPayloadIngestServiceError::Storage(error) => {
                Self::internal("knowledge_ingest_drive_failed", error.to_string())
            }
        }
    }
}

impl From<IngestionJobStoreError> for ApiError {
    fn from(error: IngestionJobStoreError) -> Self {
        match error {
            IngestionJobStoreError::NotFound(job_id) => Self::not_found(
                "ingestion_job_not_found",
                format!("ingestion job was not found: {job_id}"),
            ),
            IngestionJobStoreError::Conflict(detail) => {
                Self::conflict("ingestion_job_conflict", detail)
            }
            IngestionJobStoreError::Internal(detail) => {
                Self::internal("ingestion_job_store_failed", detail)
            }
        }
    }
}

impl From<KnowledgeDriveImportServiceError> for ApiError {
    fn from(error: KnowledgeDriveImportServiceError) -> Self {
        match error {
            KnowledgeDriveImportServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_drive_import_request", detail)
            }
            KnowledgeDriveImportServiceError::DocumentStore(error) => Self::from(error),
            KnowledgeDriveImportServiceError::IngestionJobStore(error) => Self::from(error),
            error => Self::internal("knowledge_drive_import_failed", error.to_string()),
        }
    }
}

impl From<KnowledgeBrowserServiceError> for ApiError {
    fn from(error: KnowledgeBrowserServiceError) -> Self {
        match error {
            KnowledgeBrowserServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_browser_request", detail)
            }
            KnowledgeBrowserServiceError::AccessDenied(detail) => Self::new(
                axum::http::StatusCode::FORBIDDEN,
                "knowledge_browser_access_denied",
                detail,
            ),
            KnowledgeBrowserServiceError::SpaceStore(error) => Self::from(error),
            KnowledgeBrowserServiceError::DriveTree(error) => {
                Self::internal("knowledge_browser_drive_tree_failed", error.to_string())
            }
            KnowledgeBrowserServiceError::ProjectionStore(error) => {
                Self::internal("knowledge_browser_projection_failed", error.to_string())
            }
            KnowledgeBrowserServiceError::AccessControl(error) => {
                Self::internal("knowledge_browser_access_control_failed", error.to_string())
            }
        }
    }
}

impl From<KnowledgeDocumentStoreError> for ApiError {
    fn from(error: KnowledgeDocumentStoreError) -> Self {
        match error {
            KnowledgeDocumentStoreError::InvalidRecord(detail) => {
                Self::invalid_request("invalid_knowledge_document_record", detail)
            }
            KnowledgeDocumentStoreError::Internal(detail) => {
                if detail.contains("missing knowledge document") {
                    Self::not_found("knowledge_document_not_found", detail)
                } else {
                    Self::internal("knowledge_document_store_failed", detail)
                }
            }
        }
    }
}

impl From<KnowledgeStorageError> for ApiError {
    fn from(error: KnowledgeStorageError) -> Self {
        match error {
            KnowledgeStorageError::NotFound(detail) => {
                Self::not_found("knowledge_storage_not_found", detail)
            }
            KnowledgeStorageError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_storage_request", detail)
            }
            KnowledgeStorageError::IntegrityFailed(detail) => {
                Self::internal("knowledge_storage_integrity_failed", detail)
            }
            KnowledgeStorageError::Upstream(detail) => {
                Self::internal("knowledge_storage_upstream_failed", detail)
            }
            KnowledgeStorageError::Internal(detail) => {
                Self::internal("knowledge_storage_failed", detail)
            }
        }
    }
}

impl From<KnowledgeWikiPageServiceError> for ApiError {
    fn from(error: KnowledgeWikiPageServiceError) -> Self {
        match error {
            KnowledgeWikiPageServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_wiki_page_request", detail)
            }
            other => Self::internal("knowledge_wiki_page_service_failed", other.to_string()),
        }
    }
}
