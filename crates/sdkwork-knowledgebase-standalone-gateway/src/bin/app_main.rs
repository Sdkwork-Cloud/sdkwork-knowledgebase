use sdkwork_knowledgebase_gateway_assembly::{
    assemble_application_router, resolve_database_url, validate_process_config,
    KnowledgebaseRuntime,
};
use sdkwork_knowledgebase_standalone_gateway::serve_router_with_runtime_shutdown;
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

    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id).await?;
    runtime.readiness_check().await?;

    let router = assemble_application_router(Arc::new(runtime)).await.router;
    serve_router_with_runtime_shutdown(
        &listen_addr,
        "sdkwork-knowledgebase-standalone-gateway",
        router,
    )
    .await?;
    Ok(())
}
