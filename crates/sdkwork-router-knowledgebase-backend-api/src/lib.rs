//! Backend API route boundary for SDKWork Knowledgebase.

mod auth;
pub mod error;
mod handlers;
pub mod health;
pub mod http_route_manifest;
pub mod manifest;
pub mod paths;
pub mod permission;
pub mod ports;
mod response;
pub mod routes;
mod web_audit_store;
mod web_bootstrap;
mod web_framework_assembly;
mod web_rate_limit_store;

pub use auth::{ensure_runtime_tenant, require_backend_context, require_backend_mutation_context};
pub use error::{BackendApiError, BackendApiProblem, BackendApiResult};
pub use health::DbReadinessCheck;
pub use http_route_manifest::backend_route_manifest;
pub use permission::{can_access_knowledge_admin, KNOWLEDGE_ADMIN_PERMISSION};
pub use ports::{KnowledgeBackendApi, KnowledgeBackendRequestContext};
pub use routes::{
    build_router_with_backend_api, build_router_with_shared_backend_api,
    build_router_with_shared_backend_api_and_readiness,
};
pub use sdkwork_knowledgebase_contract::ProblemDetails;
pub use web_audit_store::attach_knowledgebase_audit_emitter;
pub use web_bootstrap::{
    knowledgebase_backend_public_path_prefixes, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env,
};
pub use web_framework_assembly::apply_knowledgebase_web_framework;
pub use web_rate_limit_store::knowledgebase_rate_limit_store;
