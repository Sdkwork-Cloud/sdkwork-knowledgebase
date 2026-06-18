//! Backend API route boundary for SDKWork Knowledgebase.

mod auth;
pub mod error;
mod handlers;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
pub mod ports;
mod response;
pub mod routes;
mod web_bootstrap;

pub use error::{BackendApiError, BackendApiProblem, BackendApiResult};
pub use ports::{KnowledgeBackendApi, KnowledgeBackendRequestContext};
pub use routes::{build_router_with_backend_api, build_router_with_shared_backend_api};
pub use http_route_manifest::backend_route_manifest;
pub use sdkwork_knowledgebase_contract::ProblemDetails;
pub use web_bootstrap::{
    knowledgebase_backend_public_path_prefixes, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};
