//! Open API route boundary for SDKWork Knowledgebase.

mod error;
pub mod manifest;
pub mod paths;
mod ports;
mod routes;

pub use error::{ApiError, ApiProblem, ApiResult};
pub use ports::{KnowledgeOpenApi, KnowledgeOpenApiRequestContext};
pub use routes::{build_router_with_open_api, build_router_with_shared_open_api};
pub use sdkwork_knowledgebase_contract::ProblemDetails;
