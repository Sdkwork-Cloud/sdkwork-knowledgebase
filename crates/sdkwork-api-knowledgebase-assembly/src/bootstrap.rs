//! Gateway bootstrap for sdkwork-knowledgebase.
//! Multi-surface merges mount shared infrastructure routes once at the assembly layer.

use axum::Router;
use sdkwork_intelligence_knowledgebase_service::ports::group_launch_ticket_consumer::GroupLaunchTicketConsumer;
use sdkwork_routes_knowledgebase_app_api::bootstrap::{
    resolve_database_url, validate_process_config,
};
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;
use sdkwork_routes_knowledgebase_backend_api::health;
use sdkwork_web_bootstrap::assemble_multi_surface_router;
use std::sync::Arc;

async fn ensure_iam_session_resolution_database_ready() {
    if let Err(error) = sdkwork_iam_database_host::bootstrap_iam_database_from_env().await {
        eprintln!(
            "[sdkwork-api-knowledgebase-assembly] IAM database bootstrap for session resolution skipped: {error}"
        );
    }
}

pub struct ApiAssembly {
    pub router: Router,
}

async fn runtime_from_environment_with_group_launch_ticket_consumer(
    group_launch_ticket_consumer: Option<Arc<dyn GroupLaunchTicketConsumer>>,
) -> Result<Arc<KnowledgebaseRuntime>, Box<dyn std::error::Error + Send + Sync>> {
    validate_process_config();
    let database_url = resolve_database_url();
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id).await?;
    let runtime = match group_launch_ticket_consumer {
        Some(consumer) => runtime.with_group_launch_ticket_consumer(consumer),
        None => runtime,
    };
    runtime.readiness_check().await?;
    Ok(Arc::new(runtime))
}

impl ApiAssembly {
    pub async fn from_environment<T>(
        group_launch_ticket_consumer: Option<T>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        T: GroupLaunchTicketConsumer + 'static,
    {
        let runtime = runtime_from_environment_with_group_launch_ticket_consumer(
            group_launch_ticket_consumer
                .map(|consumer| Arc::new(consumer) as Arc<dyn GroupLaunchTicketConsumer>),
        )
        .await?;
        Ok(assemble_api_router(runtime).await)
    }
}

pub async fn assemble_api_router_from_environment(
) -> Result<ApiAssembly, Box<dyn std::error::Error + Send + Sync>> {
    let runtime = runtime_from_environment_with_group_launch_ticket_consumer(None).await?;
    Ok(assemble_api_router(runtime).await)
}

pub async fn assemble_business_routes_from_environment(
) -> Result<ApiAssembly, Box<dyn std::error::Error + Send + Sync>> {
    let runtime = runtime_from_environment_with_group_launch_ticket_consumer(None).await?;
    Ok(assemble_business_routes(runtime).await)
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

pub async fn assemble_business_routes(runtime: Arc<KnowledgebaseRuntime>) -> ApiAssembly {
    ensure_iam_session_resolution_database_ready().await;

    // Embed IAM app-api business routes through sdkwork-api-iam-assembly so
    // `/app/v3/api/auth|oauth/*` resolve locally without coupling to IAM route crates.
    // Unified-process hosts such as sdkwork-im-standalone-gateway mount IAM once at
    // the platform assembly layer and must set `SDKWORK_IAM_APP_API_HOST_MOUNTED=true`.
    let mut router = Router::new();
    if !host_mounts_iam_app_api_routes() {
        let (iam, host) = sdkwork_api_iam_assembly::bootstrap_iam_for_application()
            .await
            .expect("initialize embedded IAM owner API surfaces");
        let resolver = sdkwork_iam_web_adapter::IamWebRequestContextResolver::from_database_pool(
            Some(host.pool().clone()),
        );
        router = router.merge(
            sdkwork_iam_web_adapter::wrap_router_with_iam_owner_web_framework(
                iam.router,
                resolver,
                iam.route_manifest,
            ),
        );
    }
    let router = router
        .merge(runtime.build_full_app_router_with_web_framework().await)
        .merge(
            runtime
                .build_internal_business_router_with_web_framework()
                .await,
        )
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

pub async fn assemble_api_router(runtime: Arc<KnowledgebaseRuntime>) -> ApiAssembly {
    let readiness = runtime.readiness_check_adapter();
    let business = assemble_business_routes(runtime).await;
    let router = assemble_multi_surface_router(
        [business.router],
        health::knowledgebase_service_router_config(Some(readiness)),
    );
    ApiAssembly { router }
}
