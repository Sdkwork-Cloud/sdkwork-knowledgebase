use sdkwork_router_knowledgebase_app_api::{dev_auth, KnowledgebaseSqliteRuntime};

#[tokio::main]
async fn main() {
    let database_url = std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/knowledgebase.db?mode=rwc".to_string());
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let actor_id = std::env::var("SDKWORK_KNOWLEDGEBASE_ACTOR_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());
    let listen_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_BIND")
        .unwrap_or_else(|_| "127.0.0.1:18083".to_string());

    let runtime = KnowledgebaseSqliteRuntime::connect(&database_url, tenant_id)
        .await
        .expect("initialize knowledgebase sqlite runtime");
    runtime
        .readiness_check()
        .await
        .expect("knowledgebase database readiness check failed");

    let router = dev_auth::with_dev_open_auth(runtime.build_open_api_router(), tenant_id, actor_id);
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .expect("bind knowledgebase open api listener");
    eprintln!("sdkwork-knowledgebase-open-api listening on {listen_addr}");
    axum::serve(listener, router)
        .await
        .expect("serve knowledgebase open api");
}
