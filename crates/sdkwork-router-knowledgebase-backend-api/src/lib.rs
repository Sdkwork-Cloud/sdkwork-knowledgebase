//! Backend API route boundary for SDKWork Knowledgebase.

pub mod error;
mod handlers;
pub mod manifest;
pub mod paths;
pub mod ports;
mod response;
pub mod routes;

pub use error::{BackendApiError, BackendApiProblem, BackendApiResult};
pub use ports::KnowledgeBackendApi;
pub use routes::{build_router_with_backend_api, build_router_with_shared_backend_api};
pub use sdkwork_knowledgebase_contract::ProblemDetails;
