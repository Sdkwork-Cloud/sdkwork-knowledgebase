use sdkwork_router_knowledgebase_app_api::KnowledgebaseSqliteRuntime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTickResult {
    pub outbox_published: usize,
    pub ingestion_jobs_processed: usize,
}

pub async fn run_maintenance_tick(
    runtime: &KnowledgebaseSqliteRuntime,
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
    runtime: KnowledgebaseSqliteRuntime,
    interval_ms: u64,
    outbox_limit: u32,
    ingestion_job_limit: u32,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(interval_ms.max(250)));
    loop {
        ticker.tick().await;
        let result = run_maintenance_tick(&runtime, outbox_limit, ingestion_job_limit).await;
        if result.outbox_published > 0 || result.ingestion_jobs_processed > 0 {
            tracing::info!(
                outbox_published = result.outbox_published,
                ingestion_jobs_processed = result.ingestion_jobs_processed,
                "knowledgebase worker maintenance tick"
            );
        }
    }
}
