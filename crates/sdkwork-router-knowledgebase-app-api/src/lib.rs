//! App API route boundary for SDKWork Knowledgebase.

mod adapters;
mod error;
pub mod manifest;
pub mod paths;
mod ports;
mod routes;

pub use error::{ApiError, ApiProblem, ApiResult};
pub use ports::{
    KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext, KnowledgeBrowserApi,
    KnowledgeRetrievalAppService,
};
pub use routes::{
    build_router_with_agent_and_retrieval_services, build_router_with_agent_service,
    build_router_with_app_api, build_router_with_browser, build_router_with_retrieval_service,
    build_router_with_shared_agent_and_retrieval_services, build_router_with_shared_agent_service,
    build_router_with_shared_app_api, build_router_with_shared_browser,
    build_router_with_shared_retrieval_service,
};
pub use sdkwork_knowledgebase_contract::ProblemDetails;
