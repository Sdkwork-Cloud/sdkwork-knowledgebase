use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::KnowledgeStorageError;
use sdkwork_intelligence_knowledgebase_service::{
    agent::KnowledgeAgentServiceError,
    agent_chat::KnowledgeAgentChatServiceError,
    browser::KnowledgeBrowserServiceError,
    context_binding::KnowledgeContextBindingServiceError,
    imports::{KnowledgeDriveImportServiceError, KnowledgeGitImportServiceError},
    ingest::{
        ApiMarkdownIngestPipelineError, KnowledgeApiMarkdownIndexServiceError,
        KnowledgeApiPayloadIngestServiceError, KnowledgeIngestionServiceError,
        KnowledgeUploadSessionServiceError,
    },
    okf::OkfConceptServiceError,
    ports::{
        knowledge_agent_profile_store::KnowledgeAgentProfileStoreError,
        knowledge_context_binding_store::KnowledgeContextBindingStoreError,
        knowledge_document_store::KnowledgeDocumentStoreError,
        knowledge_ingestion_job_store::IngestionJobStoreError,
        knowledge_memory_context::KnowledgeMemoryContextProviderError,
        knowledge_retrieval_backend::KnowledgeRetrievalBackendError,
        knowledge_retrieval_trace_store::KnowledgeRetrievalTraceStoreError,
        knowledge_source_store::KnowledgeSourceStoreError,
        knowledge_space_store::KnowledgeSpaceStoreError,
    },
    retrieval::KnowledgeRetrievalServiceError,
    space::KnowledgeSpaceServiceError,
    wechat::KnowledgeWechatServiceError,
};
use sdkwork_knowledgebase_contract::ProblemDetails;

pub type ApiResult<T> = Result<T, ApiError>;

const INTERNAL_CLIENT_DETAIL: &str = "An internal error occurred. Please try again later.";

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
        Self::sanitized_internal(code, detail)
    }

    pub fn sanitized_internal(code: impl Into<String>, internal_detail: impl Into<String>) -> Self {
        let code_value = code.into();
        eprintln!(
            "[knowledgebase-app-api] internal error code={code_value}: {}",
            internal_detail.into()
        );
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            code_value,
            INTERNAL_CLIENT_DETAIL,
        )
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
        let client_detail = if status.is_server_error() {
            INTERNAL_CLIENT_DETAIL.to_string()
        } else {
            detail.into()
        };
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
                detail: Some(client_detail),
                instance: None,
                code: Some(code.into()),
                trace_id: None,
            }),
        }
    }

    pub fn from_internal(code: impl Into<String>, internal_detail: impl Into<String>) -> Self {
        Self::from(ApiError::sanitized_internal(code, internal_detail))
    }
}

impl From<ApiError> for ApiProblem {
    fn from(error: ApiError) -> Self {
        Self::new(error.status, error.code, error.detail)
    }
}

impl IntoResponse for ApiProblem {
    fn into_response(self) -> Response {
        sdkwork_knowledgebase_observability::request_correlation::problem_json_response(
            self.status,
            *self.problem,
        )
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
                if detail.contains("capability unsupported") {
                    Self::invalid_request("knowledge_agent_chat_provider_unsupported", detail)
                } else {
                    Self::internal("knowledge_agent_chat_provider_failed", detail)
                }
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
            KnowledgeSpaceServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_space_request", detail)
            }
            KnowledgeSpaceServiceError::AccessDenied(detail) => Self::new(
                StatusCode::FORBIDDEN,
                "knowledge_space_access_denied",
                detail,
            ),
            KnowledgeSpaceServiceError::Store(error) => Self::from(error),
            KnowledgeSpaceServiceError::OkfBundleInitializer(error) => Self::internal(
                "knowledge_space_okf_initialization_failed",
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

impl From<KnowledgeIngestionServiceError> for ApiError {
    fn from(error: KnowledgeIngestionServiceError) -> Self {
        match error {
            KnowledgeIngestionServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_ingestion_job_request", detail)
            }
            KnowledgeIngestionServiceError::InvalidTransition { from, to } => Self::conflict(
                "invalid_ingestion_job_transition",
                format!("invalid ingestion job transition: {from:?} -> {to:?}"),
            ),
            KnowledgeIngestionServiceError::Store(error) => Self::from(error),
        }
    }
}

impl From<KnowledgeApiPayloadIngestServiceError> for ApiError {
    fn from(error: KnowledgeApiPayloadIngestServiceError) -> Self {
        match error {
            KnowledgeApiPayloadIngestServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_ingest_request", detail)
            }
            KnowledgeApiPayloadIngestServiceError::WebLink(error) => match error {
                sdkwork_intelligence_knowledgebase_service::ingest::WebLinkFetchError::InvalidRequest(
                    detail,
                ) => Self::invalid_request("invalid_knowledge_ingest_source_url", detail),
                sdkwork_intelligence_knowledgebase_service::ingest::WebLinkFetchError::Upstream(
                    detail,
                ) => Self::internal("knowledge_ingest_source_url_fetch_failed", detail),
            },
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

impl From<KnowledgeWechatServiceError> for ApiError {
    fn from(error: KnowledgeWechatServiceError) -> Self {
        match error {
            KnowledgeWechatServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_wechat_request", detail)
            }
            KnowledgeWechatServiceError::Storage(error) => error.into(),
            KnowledgeWechatServiceError::Api(error) => {
                Self::internal("wechat_upstream_failed", error.to_string())
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
            KnowledgeDriveImportServiceError::Storage(error) => Self::from(error),
            KnowledgeDriveImportServiceError::Metadata(error) => match error {
                sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError::InvalidRequest(detail) => {
                    Self::invalid_request("invalid_knowledge_drive_import_request", detail)
                }
                sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError::Conflict(detail) => {
                    Self::conflict("knowledge_drive_import_conflict", detail)
                }
                sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError::Internal(detail) => {
                    Self::internal("drive_import_metadata_store_failed", detail)
                }
            },
        }
    }
}

impl From<ApiMarkdownIngestPipelineError> for ApiError {
    fn from(error: ApiMarkdownIngestPipelineError) -> Self {
        match error {
            ApiMarkdownIngestPipelineError::Payload(error) => error.into(),
            ApiMarkdownIngestPipelineError::Ingestion(error) => error.into(),
            ApiMarkdownIngestPipelineError::Index(error) => error.into(),
            ApiMarkdownIngestPipelineError::Store(error) => error.into(),
            ApiMarkdownIngestPipelineError::Storage(error) => error.into(),
        }
    }
}

impl From<KnowledgeGitImportServiceError> for ApiError {
    fn from(error: KnowledgeGitImportServiceError) -> Self {
        match error {
            KnowledgeGitImportServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_git_import_request", detail)
            }
            KnowledgeGitImportServiceError::GitHub(
                sdkwork_intelligence_knowledgebase_service::imports::GitHubApiError::InvalidRequest(
                    detail,
                ),
            ) => Self::invalid_request("invalid_knowledge_git_import_request", detail),
            KnowledgeGitImportServiceError::GitHub(error) => {
                Self::internal("knowledge_git_import_upstream_failed", error.to_string())
            }
            KnowledgeGitImportServiceError::Pipeline(error) => error.into(),
        }
    }
}

impl From<sdkwork_intelligence_knowledgebase_service::imports::KnowledgeGitSyncServiceError>
    for ApiError
{
    fn from(
        error: sdkwork_intelligence_knowledgebase_service::imports::KnowledgeGitSyncServiceError,
    ) -> Self {
        use sdkwork_intelligence_knowledgebase_service::imports::KnowledgeGitSyncServiceError;
        use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::KnowledgeDocumentStoreError;
        match error {
            KnowledgeGitSyncServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_git_sync_request", detail)
            }
            KnowledgeGitSyncServiceError::GitHub(
                sdkwork_intelligence_knowledgebase_service::imports::GitHubApiError::InvalidRequest(
                    detail,
                ),
            ) => Self::invalid_request("invalid_knowledge_git_sync_request", detail),
            KnowledgeGitSyncServiceError::GitHub(error) => {
                Self::internal("knowledge_git_sync_upstream_failed", error.to_string())
            }
            KnowledgeGitSyncServiceError::DocumentContent(detail) => {
                Self::invalid_request("invalid_knowledge_git_sync_request", detail)
            }
            KnowledgeGitSyncServiceError::DocumentStore(
                KnowledgeDocumentStoreError::InvalidRecord(detail),
            ) => Self::invalid_request("invalid_knowledge_git_sync_request", detail),
            KnowledgeGitSyncServiceError::DocumentStore(error) => Self::internal(
                "knowledge_git_sync_document_store_failed",
                error.to_string(),
            ),
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

impl From<KnowledgeSourceStoreError> for ApiError {
    fn from(error: KnowledgeSourceStoreError) -> Self {
        match error {
            KnowledgeSourceStoreError::Internal(detail) => {
                Self::internal("knowledge_source_store_failed", detail)
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

impl From<sdkwork_intelligence_knowledgebase_service::okf::OkfBundleWorkflowError> for ApiError {
    fn from(
        error: sdkwork_intelligence_knowledgebase_service::okf::OkfBundleWorkflowError,
    ) -> Self {
        use sdkwork_intelligence_knowledgebase_service::okf::OkfBundleWorkflowError;
        match error {
            OkfBundleWorkflowError::InvalidRequest(detail) => {
                Self::invalid_request("okf_bundle_workflow_invalid_request", detail)
            }
            OkfBundleWorkflowError::SpaceStore(store_error) => store_error.into(),
            OkfBundleWorkflowError::ConceptStore(store_error) => {
                Self::internal("okf_bundle_workflow_failed", store_error.to_string())
            }
            OkfBundleWorkflowError::SourceStore(store_error) => store_error.into(),
            OkfBundleWorkflowError::IndexRebuild(rebuild_error) => {
                Self::internal("okf_bundle_workflow_failed", rebuild_error.to_string())
            }
            OkfBundleWorkflowError::Linter(linter_error) => {
                Self::internal("okf_bundle_workflow_failed", linter_error.to_string())
            }
            OkfBundleWorkflowError::Storage(storage_error) => storage_error.into(),
            OkfBundleWorkflowError::BundleFileStore(store_error) => {
                Self::internal("okf_bundle_workflow_failed", store_error.to_string())
            }
            OkfBundleWorkflowError::BundleFileRegistry(registry_error) => {
                Self::internal("okf_bundle_workflow_failed", registry_error.to_string())
            }
            OkfBundleWorkflowError::CatalogSync(catalog_sync_error) => {
                use sdkwork_intelligence_knowledgebase_service::okf::StandardBundleCatalogSyncError;
                match catalog_sync_error {
                    StandardBundleCatalogSyncError::Registry(registry_error) => {
                        Self::internal("okf_bundle_workflow_failed", registry_error.to_string())
                    }
                    StandardBundleCatalogSyncError::DriveWorkspace(workspace_error) => {
                        Self::internal("okf_bundle_workflow_failed", workspace_error.to_string())
                    }
                }
            }
            OkfBundleWorkflowError::Engine(engine_error) => engine_error.into(),
        }
    }
}

impl From<sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError> for ApiError {
    fn from(error: sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError) -> Self {
        use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError;
        match error {
            KnowledgeEngineError::Validation(detail) => {
                Self::invalid_request("knowledge_engine_validation_failed", detail)
            }
            KnowledgeEngineError::NotFound(detail) => {
                Self::not_found("knowledge_engine_not_found", detail)
            }
            KnowledgeEngineError::Unsupported(detail) => {
                Self::invalid_request("knowledge_engine_unsupported", detail)
            }
            KnowledgeEngineError::Internal(detail) => {
                Self::internal("knowledge_engine_failed", detail)
            }
        }
    }
}

impl From<OkfConceptServiceError> for ApiError {
    fn from(error: OkfConceptServiceError) -> Self {
        match error {
            OkfConceptServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_okf_concept_request", detail)
            }
            other => Self::internal("knowledge_okf_concept_service_failed", other.to_string()),
        }
    }
}

impl From<sdkwork_intelligence_knowledgebase_service::okf::OkfBundleImporterError> for ApiError {
    fn from(
        error: sdkwork_intelligence_knowledgebase_service::okf::OkfBundleImporterError,
    ) -> Self {
        use sdkwork_intelligence_knowledgebase_service::okf::OkfBundleImporterError;
        match error {
            OkfBundleImporterError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_okf_bundle_import_request", detail)
            }
            OkfBundleImporterError::Conformance(detail) => {
                Self::invalid_request("okf_bundle_import_conformance_failed", detail)
            }
            OkfBundleImporterError::Storage(storage_error) => storage_error.into(),
            OkfBundleImporterError::ConceptService(service_error) => service_error.into(),
        }
    }
}

impl From<KnowledgeContextBindingServiceError> for ApiError {
    fn from(error: KnowledgeContextBindingServiceError) -> Self {
        match error {
            KnowledgeContextBindingServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_context_binding_request", detail)
            }
            KnowledgeContextBindingServiceError::Store(store_error) => store_error.into(),
            KnowledgeContextBindingServiceError::DrivePermission(detail) => Self::internal(
                "knowledge_context_binding_drive_permission_failed",
                detail.to_string(),
            ),
        }
    }
}

impl From<KnowledgeContextBindingStoreError> for ApiError {
    fn from(error: KnowledgeContextBindingStoreError) -> Self {
        match error {
            KnowledgeContextBindingStoreError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_context_binding_request", detail)
            }
            KnowledgeContextBindingStoreError::NotFound(binding_id) => Self::not_found(
                "knowledge_context_binding_not_found",
                format!("knowledge context binding was not found: {binding_id}"),
            ),
            KnowledgeContextBindingStoreError::Conflict(detail) => {
                Self::conflict("knowledge_context_binding_conflict", detail)
            }
            KnowledgeContextBindingStoreError::Internal(detail) => {
                Self::internal("knowledge_context_binding_store_failed", detail)
            }
        }
    }
}

impl From<KnowledgeApiMarkdownIndexServiceError> for ApiError {
    fn from(error: KnowledgeApiMarkdownIndexServiceError) -> Self {
        match error {
            KnowledgeApiMarkdownIndexServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_markdown_index_request", detail)
            }
            KnowledgeApiMarkdownIndexServiceError::Metadata(error) => match error {
                sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::MarkdownIndexMetadataStoreError::InvalidRequest(detail) => {
                    Self::invalid_request("invalid_knowledge_markdown_index_request", detail)
                }
                sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::MarkdownIndexMetadataStoreError::Conflict(detail) => {
                    Self::conflict("markdown_index_metadata_conflict", detail)
                }
                sdkwork_intelligence_knowledgebase_service::ports::markdown_index_metadata_store::MarkdownIndexMetadataStoreError::Internal(detail) => {
                    Self::internal("markdown_index_metadata_store_failed", detail)
                }
            },
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl From<sdkwork_intelligence_knowledgebase_service::imports::KnowledgeDriveImportPipelineServiceError>
    for ApiError
{
    fn from(
        error: sdkwork_intelligence_knowledgebase_service::imports::KnowledgeDriveImportPipelineServiceError,
    ) -> Self {
        match error {
            sdkwork_intelligence_knowledgebase_service::imports::KnowledgeDriveImportPipelineServiceError::Ingestion(
                error,
            ) => Self::from(error),
            sdkwork_intelligence_knowledgebase_service::imports::KnowledgeDriveImportPipelineServiceError::Storage(
                error,
            ) => Self::from(error),
        }
    }
}

impl From<KnowledgeUploadSessionServiceError> for ApiError {
    fn from(error: KnowledgeUploadSessionServiceError) -> Self {
        match error {
            KnowledgeUploadSessionServiceError::InvalidRequest(detail) => {
                Self::invalid_request("invalid_knowledge_upload_session_request", detail)
            }
            KnowledgeUploadSessionServiceError::Internal(detail) => {
                Self::internal("knowledge_upload_session_failed", detail)
            }
            KnowledgeUploadSessionServiceError::Store(store_error) => store_error.into(),
            KnowledgeUploadSessionServiceError::Storage(storage_error) => storage_error.into(),
        }
    }
}
