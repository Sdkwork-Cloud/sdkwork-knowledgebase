//! Open API route boundary for SDKWork Knowledgebase.

mod auth;
mod error;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
mod ports;
mod routes;
mod web_bootstrap;

pub use error::{ApiError, ApiProblem, ApiResult};
pub use http_route_manifest::open_route_manifest;
pub use ports::{KnowledgeOpenApi, KnowledgeOpenApiRequestContext};
pub use routes::{build_router_with_open_api, build_router_with_shared_open_api};
pub use sdkwork_knowledgebase_contract::ProblemDetails;
pub use web_bootstrap::{
    knowledgebase_open_api_prefixes, knowledgebase_open_api_public_path_prefixes,
    wrap_router_with_web_framework, wrap_router_with_web_framework_from_env,
};
