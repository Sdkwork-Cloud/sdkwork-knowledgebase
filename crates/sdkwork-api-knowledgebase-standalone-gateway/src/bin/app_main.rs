use sdkwork_api_knowledgebase_assembly::ApiAssembly;
use sdkwork_api_knowledgebase_standalone_gateway::{
    resolve_group_launch_ticket_consumer_from_env, serve_router_with_runtime_shutdown,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18081".to_string());
    let group_launch_ticket_consumer = resolve_group_launch_ticket_consumer_from_env().await?;
    let router = ApiAssembly::from_environment(group_launch_ticket_consumer)
        .await?
        .router;
    serve_router_with_runtime_shutdown(
        &listen_addr,
        "sdkwork-api-knowledgebase-standalone-gateway",
        router,
    )
    .await?;
    Ok(())
}
