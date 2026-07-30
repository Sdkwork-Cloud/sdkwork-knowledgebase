use sdkwork_api_knowledgebase_assembly::assemble_api_router_from_environment_with_group_launch_ticket_consumer;
use sdkwork_api_knowledgebase_standalone_gateway::{
    resolve_group_launch_ticket_consumer_from_env, serve_router_with_runtime_shutdown,
};
use sdkwork_web_bootstrap::{infra_public_path_prefixes, ComposedApiAssembly};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdkwork_database_sqlx::enable_process_shared_database_pool();
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "0.0.0.0:18081".to_string());
    let group_launch_ticket_consumer = resolve_group_launch_ticket_consumer_from_env().await?;
    let knowledgebase = assemble_api_router_from_environment_with_group_launch_ticket_consumer(
        group_launch_ticket_consumer,
    )
    .await?;
    let iam = sdkwork_api_iam_assembly::assemble_app_api_contribution().await?;
    let composed =
        ComposedApiAssembly::try_compose("SDKWork Knowledgebase API", vec![knowledgebase, iam])?;
    let framework = sdkwork_iam_web_adapter::build_web_framework_builder(
        sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await,
        composed.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    let router = composed.into_hosted(framework).router;
    serve_router_with_runtime_shutdown(
        &listen_addr,
        "sdkwork-api-knowledgebase-standalone-gateway",
        router,
    )
    .await?;
    Ok(())
}
