//! Gateway assembly for sdkwork-knowledgebase.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_application_business_router, assemble_application_router, ApplicationAssembly,
};
pub use sdkwork_routes_knowledgebase_app_api::bootstrap as app_api_bootstrap;
pub use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;
pub use sdkwork_routes_knowledgebase_app_api::bootstrap::{
    resolve_database_url, resolve_deployment_tenant_id, validate_process_config,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
