use sdkwork_knowledgebase_gateway_assembly::{
    app_api_bootstrap as bootstrap, assemble_application_router, KnowledgebaseRuntime,
};
use sdkwork_knowledgebase_standalone_gateway::serve_router;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    bootstrap::validate_process_config();

    let database_url = bootstrap::resolve_database_url();
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18081".to_string());

    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id)
        .await
        .expect("initialize knowledgebase runtime");
    runtime
        .readiness_check()
        .await
        .expect("knowledgebase database readiness check failed");

    let router = assemble_application_router(Arc::new(runtime))
        .await
        .router;
    serve_router(
        &listen_addr,
        "sdkwork-knowledgebase-standalone-gateway",
        router,
    )
    .await;
}
