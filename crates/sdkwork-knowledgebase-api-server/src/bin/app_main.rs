use sdkwork_knowledgebase_api_server::serve_router;
use sdkwork_routes_knowledgebase_app_api::{bootstrap, KnowledgebaseRuntime};

#[tokio::main]
async fn main() {
    bootstrap::validate_process_config();

    let database_url = bootstrap::resolve_database_url();
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let actor_id = std::env::var("SDKWORK_KNOWLEDGEBASE_ACTOR_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18081".to_string());

    let operator_id = std::env::var("SDKWORK_KNOWLEDGEBASE_OPERATOR_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());

    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id)
        .await
        .expect("initialize knowledgebase runtime");
    runtime
        .readiness_check()
        .await
        .expect("knowledgebase database readiness check failed");

    let router = if bootstrap::is_unified_process_layout() {
        bootstrap::build_served_unified_router(&runtime, tenant_id, actor_id, operator_id).await
    } else {
        bootstrap::build_served_app_router(&runtime, tenant_id, actor_id).await
    };
    serve_router(&listen_addr, "sdkwork-knowledgebase-app-api", router).await;
}
