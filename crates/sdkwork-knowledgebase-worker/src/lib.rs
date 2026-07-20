use sdkwork_api_knowledgebase_standalone_gateway::shutdown_signal;
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;

pub mod health;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTickResult {
    pub outbox_published: usize,
    pub ingestion_jobs_processed: usize,
    pub provider_migration_phases_processed: usize,
    pub provider_migrations_completed: usize,
    pub provider_migrations_rolled_back: usize,
    pub provider_migrations_failed: usize,
    pub group_archives_processed: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceTickError {
    #[error("ingestion job batch failed: {0}")]
    Ingestion(String),
    #[error("Provider migration batch failed: {0}")]
    ProviderMigration(String),
}

pub async fn run_maintenance_tick(
    runtime: &KnowledgebaseRuntime,
    worker_id: &str,
    ingestion_job_lease: time::Duration,
    provider_migration_lease: std::time::Duration,
    outbox_limit: u32,
    ingestion_job_limit: u32,
    provider_migration_limit: u32,
    group_archive_limit: u32,
) -> Result<MaintenanceTickResult, MaintenanceTickError> {
    let outbox_published = runtime.publish_pending_outbox_events(outbox_limit).await;
    let ingestion_jobs_processed = runtime
        .process_queued_ingestion_jobs(worker_id, ingestion_job_lease, ingestion_job_limit)
        .await
        .map_err(MaintenanceTickError::Ingestion)?;
    let provider_migrations = runtime
        .process_provider_migrations(
            worker_id,
            provider_migration_lease,
            provider_migration_limit,
        )
        .await
        .map_err(MaintenanceTickError::ProviderMigration)?;
    let group_archives_processed = runtime
        .process_resumable_group_space_archives(group_archive_limit)
        .await;
    Ok(MaintenanceTickResult {
        outbox_published,
        ingestion_jobs_processed,
        provider_migration_phases_processed: provider_migrations.processed,
        provider_migrations_completed: provider_migrations.completed,
        provider_migrations_rolled_back: provider_migrations.rolled_back,
        provider_migrations_failed: provider_migrations.failed,
        group_archives_processed,
    })
}

pub async fn run_polling_loop(
    runtime: KnowledgebaseRuntime,
    worker_id: String,
    ingestion_job_lease: time::Duration,
    provider_migration_lease: std::time::Duration,
    interval_ms: u64,
    outbox_limit: u32,
    ingestion_job_limit: u32,
    provider_migration_limit: u32,
    group_archive_limit: u32,
) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(interval_ms.max(250)));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match run_maintenance_tick(
                    &runtime,
                    &worker_id,
                    ingestion_job_lease,
                    provider_migration_lease,
                    outbox_limit,
                    ingestion_job_limit,
                    provider_migration_limit,
                    group_archive_limit,
                ).await {
                    Ok(result) if result.outbox_published > 0
                        || result.ingestion_jobs_processed > 0
                        || result.provider_migration_phases_processed > 0
                        || result.group_archives_processed > 0 => {
                        tracing::info!(
                            outbox_published = result.outbox_published,
                            ingestion_jobs_processed = result.ingestion_jobs_processed,
                            provider_migration_phases_processed = result.provider_migration_phases_processed,
                            provider_migrations_completed = result.provider_migrations_completed,
                            provider_migrations_rolled_back = result.provider_migrations_rolled_back,
                            provider_migrations_failed = result.provider_migrations_failed,
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
            provider_migration_phases_processed: 5,
            provider_migrations_completed: 1,
            provider_migrations_rolled_back: 1,
            provider_migrations_failed: 0,
            group_archives_processed: 4,
        };
        assert_eq!(result.outbox_published, 2);
        assert_eq!(result.ingestion_jobs_processed, 3);
        assert_eq!(result.provider_migration_phases_processed, 5);
        assert_eq!(result.provider_migrations_completed, 1);
        assert_eq!(result.provider_migrations_rolled_back, 1);
        assert_eq!(result.provider_migrations_failed, 0);
        assert_eq!(result.group_archives_processed, 4);
    }
}
