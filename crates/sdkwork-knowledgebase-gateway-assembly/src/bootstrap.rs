//! Gateway bootstrap for sdkwork-knowledgebase.
//! Multi-surface merges mount shared infrastructure routes once at the assembly layer.

use axum::Router;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;
use sdkwork_routes_knowledgebase_backend_api::{health, DbReadinessCheck};
use sdkwork_web_bootstrap::assemble_multi_surface_router;
use std::sync::Arc;

pub struct ApplicationAssembly {
    pub router: Router,
}

pub async fn assemble_application_business_router(
    runtime: Arc<KnowledgebaseRuntime>,
) -> ApplicationAssembly {
    let router = Router::new()
        .merge(runtime.build_full_app_router_with_web_framework().await)
        .merge(
            runtime
                .build_backend_business_router_with_web_framework()
                .await,
        )
        .merge(
            runtime
                .build_open_business_router_with_web_framework()
                .await,
        );
    ApplicationAssembly { router }
}

pub async fn assemble_application_router(runtime: Arc<KnowledgebaseRuntime>) -> ApplicationAssembly {
    let readiness = DbReadinessCheck::new(runtime.pool().clone());
    let business = assemble_application_business_router(runtime).await;
    let router = assemble_multi_surface_router(
        [business.router],
        health::knowledgebase_service_router_config(Some(readiness)),
    );
    ApplicationAssembly { router }
}
