use sdkwork_api_knowledgebase_assembly::{
    assemble_api_router, resolve_database_url, validate_process_config,
    KnowledgebaseRuntime,
};
use sdkwork_api_knowledgebase_standalone_gateway::{
    resolve_group_launch_ticket_consumer_from_env, serve_router_with_runtime_shutdown,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    validate_process_config();

    let database_url = resolve_database_url();
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18081".to_string());
    let group_launch_ticket_consumer = resolve_group_launch_ticket_consumer_from_env().await?;

    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id).await?;
    let runtime = match group_launch_ticket_consumer {
        Some(consumer) => runtime.with_group_launch_ticket_consumer(Arc::new(consumer)),
        None => runtime,
    };
    runtime.readiness_check().await?;

    let router = assemble_api_router(Arc::new(runtime)).await.router;
    serve_router_with_runtime_shutdown(
        &listen_addr,
        "sdkwork-api-knowledgebase-standalone-gateway",
        router,
    )
    .await?;
    Ok(())
}
