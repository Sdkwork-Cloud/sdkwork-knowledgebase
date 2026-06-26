//! Generated gateway assembly for sdkwork-knowledgebase.

mod generated;

pub struct ApplicationAssembly {
    pub router: axum::Router,
}

pub async fn assemble_application_router() -> ApplicationAssembly {
    let mut router = axum::Router::new();
    router = router.merge(sdkwork_routes_knowledgebase_app_api::gateway_mount());
    router = router.merge(sdkwork_routes_knowledgebase_backend_api::gateway_mount());
    router = router.merge(sdkwork_routes_knowledgebase_open_api::gateway_mount());
    ApplicationAssembly { router }
}

pub fn assembly_route_count() -> usize {
    generated::ROUTE_CRATE_COUNT
}
