use sdkwork_knowledgebase_standalone_gateway::shutdown_signal;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;

pub mod health;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTickResult {
    pub outbox_published: usize,
    pub ingestion_jobs_processed: usize,
}

pub async fn run_maintenance_tick(
    runtime: &KnowledgebaseRuntime,
    outbox_limit: u32,
    ingestion_job_limit: u32,
) -> MaintenanceTickResult {
    let outbox_published = runtime.publish_pending_outbox_events(outbox_limit).await;
    let ingestion_jobs_processed = runtime
        .process_queued_ingestion_jobs(ingestion_job_limit)
        .await;
    MaintenanceTickResult {
        outbox_published,
        ingestion_jobs_processed,
    }
}

pub async fn run_polling_loop(
    runtime: KnowledgebaseRuntime,
    interval_ms: u64,
    outbox_limit: u32,
    ingestion_job_limit: u32,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(interval_ms.max(250)));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let result = run_maintenance_tick(&runtime, outbox_limit, ingestion_job_limit).await;
                if result.outbox_published > 0 || result.ingestion_jobs_processed > 0 {
                    tracing::info!(
                        outbox_published = result.outbox_published,
                        ingestion_jobs_processed = result.ingestion_jobs_processed,
                        "knowledgebase worker maintenance tick"
                    );
                }
            }
            _ = shutdown_signal() => {
                tracing::info!("knowledgebase worker shutdown signal received; exiting maintenance loop");
                break;
            }
        }
    }
}
