use sdkwork_knowledgebase_standalone_gateway::shutdown_signal;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;

pub mod health;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTickResult {
    pub outbox_published: usize,
    pub ingestion_jobs_processed: usize,
    pub group_archives_processed: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceTickError {
    #[error("ingestion job batch failed: {0}")]
    Ingestion(String),
}

pub async fn run_maintenance_tick(
    runtime: &KnowledgebaseRuntime,
    outbox_limit: u32,
    ingestion_job_limit: u32,
    group_archive_limit: u32,
) -> Result<MaintenanceTickResult, MaintenanceTickError> {
    let outbox_published = runtime.publish_pending_outbox_events(outbox_limit).await;
    let ingestion_jobs_processed = runtime
        .process_queued_ingestion_jobs(ingestion_job_limit)
        .await
        .map_err(MaintenanceTickError::Ingestion)?;
    let group_archives_processed = runtime
        .process_resumable_group_space_archives(group_archive_limit)
        .await;
    Ok(MaintenanceTickResult {
        outbox_published,
        ingestion_jobs_processed,
        group_archives_processed,
    })
}

pub async fn run_polling_loop(
    runtime: KnowledgebaseRuntime,
    interval_ms: u64,
    outbox_limit: u32,
    ingestion_job_limit: u32,
    group_archive_limit: u32,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(interval_ms.max(250)));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match run_maintenance_tick(
                    &runtime,
                    outbox_limit,
                    ingestion_job_limit,
                    group_archive_limit,
                ).await {
                    Ok(result) if result.outbox_published > 0
                        || result.ingestion_jobs_processed > 0
                        || result.group_archives_processed > 0 => {
                        tracing::info!(
                            outbox_published = result.outbox_published,
                            ingestion_jobs_processed = result.ingestion_jobs_processed,
                            group_archives_processed = result.group_archives_processed,
                            "knowledgebase worker maintenance tick"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::error!(
                            error = %error,
                            "knowledgebase worker maintenance tick failed"
                        );
                    }
                }
            }
            _ = shutdown_signal() => {
                tracing::info!("knowledgebase worker shutdown signal received; exiting maintenance loop");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MaintenanceTickResult;

    #[test]
    fn maintenance_tick_result_tracks_worker_outputs() {
        let result = MaintenanceTickResult {
            outbox_published: 2,
            ingestion_jobs_processed: 3,
            group_archives_processed: 4,
        };
        assert_eq!(result.outbox_published, 2);
        assert_eq!(result.ingestion_jobs_processed, 3);
        assert_eq!(result.group_archives_processed, 4);
    }
}
