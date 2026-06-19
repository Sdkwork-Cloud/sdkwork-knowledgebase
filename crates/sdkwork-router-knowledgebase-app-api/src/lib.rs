//! App API route boundary for SDKWork Knowledgebase.

mod adapters;
mod agent_chat_runtime;
mod auth;
pub mod bootstrap;
pub mod dev_auth;
mod error;
pub mod hosted;
mod hosted_backend;
mod hosted_context_binding;
mod hosted_open;
mod hosted_support;
mod hosted_upload;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
mod ports;
mod routes;
pub mod runtime;
mod web_bootstrap;

pub use error::{ApiError, ApiProblem, ApiResult};
pub use http_route_manifest::app_route_manifest;
pub use ports::{
    KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext, KnowledgeBrowserApi,
    KnowledgeContextBindingAppService, KnowledgeDocumentAppService, KnowledgeDriveImportAppService,
    KnowledgeIngestAppService, KnowledgeRetrievalAppService, KnowledgeSpaceAppService,
    KnowledgeUploadSessionAppService, KnowledgeWikiAppService,
};
pub use routes::{
    build_router_with_agent_and_retrieval_services, build_router_with_agent_service,
    build_router_with_app_api, build_router_with_browser, build_router_with_full_app_api,
    build_router_with_retrieval_service, build_router_with_shared_agent_and_retrieval_services,
    build_router_with_shared_agent_service, build_router_with_shared_app_api,
    build_router_with_shared_app_api_and_readiness, build_router_with_shared_browser,
    build_router_with_shared_retrieval_service, ReadinessCheck,
};
pub use runtime::KnowledgebaseRuntime;
pub use sdkwork_knowledgebase_contract::ProblemDetails;
pub use web_bootstrap::{
    knowledgebase_public_path_prefixes, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};
