use sdkwork_knowledgebase_api_server::init_tracing;
use sdkwork_knowledgebase_worker::{health, run_polling_loop};
use sdkwork_router_knowledgebase_app_api::{bootstrap, KnowledgebaseRuntime};

#[tokio::main]
async fn main() {
    bootstrap::validate_process_config();
    init_tracing("worker");

    let database_url = bootstrap::resolve_database_url();
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let interval_ms = std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_POLL_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5_000);
    let outbox_limit = std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_OUTBOX_BATCH_SIZE")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(50);
    let ingestion_job_limit =
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_INGESTION_JOB_BATCH_SIZE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(25);
    let health_addr = std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_HEALTH_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:18085".to_string());

    let runtime = KnowledgebaseRuntime::connect(&database_url, tenant_id)
        .await
        .expect("initialize knowledgebase worker runtime");
    runtime
        .readiness_check()
        .await
        .expect("knowledgebase worker readiness check failed");

    tracing::info!(
        %database_url,
        tenant_id,
        interval_ms,
        outbox_limit,
        ingestion_job_limit,
        %health_addr,
        "starting knowledgebase worker loop"
    );

    let readiness =
        sdkwork_router_knowledgebase_app_api::ReadinessCheck::new(runtime.pool().clone());
    let health_addr_for_task = health_addr.clone();
    tokio::spawn(async move {
        health::serve_worker_health(&health_addr_for_task, readiness).await;
    });

    run_polling_loop(runtime, interval_ms, outbox_limit, ingestion_job_limit).await;
}
