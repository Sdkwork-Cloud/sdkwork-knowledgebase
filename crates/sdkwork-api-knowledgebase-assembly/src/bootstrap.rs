//! Gateway bootstrap for sdkwork-knowledgebase.
//! Multi-surface merges mount shared infrastructure routes once at the assembly layer.

use axum::Router;
use sdkwork_api_iam_assembly::assemble_api_router as assemble_iam_application_business_router;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;
use sdkwork_routes_knowledgebase_backend_api::health;
use sdkwork_utils_rust::is_blank;
use sdkwork_web_bootstrap::assemble_multi_surface_router;
use std::sync::Arc;

fn bridge_embedded_iam_database_env_from_knowledgebase() {
    if !is_blank(std::env::var("SDKWORK_IAM_DATABASE_URL").ok().as_deref()) {
        return;
    }

    let Ok(knowledgebase_database_url) = std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL") else {
        return;
    };
    let trimmed = knowledgebase_database_url.trim();
    if is_blank(Some(trimmed))
        || (!trimmed.starts_with("postgres://") && !trimmed.starts_with("postgresql://"))
    {
        return;
    }

    let iam_database_url = sdkwork_database_config::claw_database::postgres_url_with_search_path(
        trimmed,
        "SDKWORK_IAM",
    );
    // SAFETY: gateway bootstrap runs sequentially on the main runtime thread.
    unsafe {
        std::env::set_var("SDKWORK_IAM_DATABASE_URL", iam_database_url);
    }
}

async fn ensure_iam_session_resolution_database_ready() {
    bridge_embedded_iam_database_env_from_knowledgebase();
    if let Err(error) = sdkwork_iam_database_host::bootstrap_iam_database_from_env().await {
        eprintln!(
            "[sdkwork-api-knowledgebase-assembly] IAM database bootstrap for session resolution skipped: {error}"
        );
    }
}

pub struct ApiAssembly {
    pub router: Router,
}

fn host_mounts_iam_app_api_routes() -> bool {
    std::env::var("SDKWORK_IAM_APP_API_HOST_MOUNTED")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub async fn assemble_business_routes(
    runtime: Arc<KnowledgebaseRuntime>,
) -> ApiAssembly {
    ensure_iam_session_resolution_database_ready().await;

    // Embed IAM app-api business routes through sdkwork-api-iam-assembly so
    // `/app/v3/api/auth|oauth/*` resolve locally without coupling to IAM route crates.
    // Unified-process hosts such as sdkwork-im-standalone-gateway mount IAM once at
    // the platform assembly layer and must set `SDKWORK_IAM_APP_API_HOST_MOUNTED=true`.
    let mut router = Router::new();
    if !host_mounts_iam_app_api_routes() {
        let iam_router = assemble_iam_application_business_router().await.router;
        router = router.merge(iam_router);
    }
    let router = router
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
    ApiAssembly { router }
}

pub async fn assemble_api_router(
    runtime: Arc<KnowledgebaseRuntime>,
) -> ApiAssembly {
    let readiness = runtime.readiness_check_adapter();
    let business = assemble_business_routes(runtime).await;
    let router = assemble_multi_surface_router(
        [business.router],
        health::knowledgebase_service_router_config(Some(readiness)),
    );
    ApiAssembly { router }
}
