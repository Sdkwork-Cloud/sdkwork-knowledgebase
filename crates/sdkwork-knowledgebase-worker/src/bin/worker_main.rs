use sdkwork_knowledgebase_worker::run_polling_loop;
use sdkwork_router_knowledgebase_app_api::{bootstrap, KnowledgebaseSqliteRuntime};

#[tokio::main]
async fn main() {
    bootstrap::validate_process_config();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sdkwork_knowledgebase_worker=debug".into()),
        )
        .init();

    let database_url = std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/knowledgebase.db?mode=rwc".to_string());
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

    let runtime = KnowledgebaseSqliteRuntime::connect(&database_url, tenant_id)
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
        "starting knowledgebase worker loop"
    );
    run_polling_loop(runtime, interval_ms, outbox_limit, ingestion_job_limit).await;
}
