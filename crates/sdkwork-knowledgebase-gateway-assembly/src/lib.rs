//! Gateway assembly for sdkwork-knowledgebase.
//! Application bootstrap lives in `bootstrap.rs`; route inventory is in `assembly-manifest.json`.

mod bootstrap;
mod generated;

pub use bootstrap::{
    assemble_application_business_router, assemble_application_router, ApplicationAssembly,
};
pub use sdkwork_routes_knowledgebase_app_api::{
    bootstrap::{resolve_database_url, validate_process_config},
    KnowledgebaseRuntime,
};

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
