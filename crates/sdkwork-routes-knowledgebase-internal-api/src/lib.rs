//! Knowledgebase application-internal HTTP routes.

mod dto;
mod error;
mod handlers;
pub mod http_route_manifest;
mod response;
mod routes;
mod state;
mod web_bootstrap;

pub use http_route_manifest::internal_route_manifest;
pub use routes::{
    build_router_with_services, gateway_mount, wrap_router_with_web_framework_from_env,
};
pub use state::{
    InternalApiState, KnowledgebaseDriveEventReceiver, KnowledgebaseWikiPublicProvider,
};

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    internal_route_manifest()
}
