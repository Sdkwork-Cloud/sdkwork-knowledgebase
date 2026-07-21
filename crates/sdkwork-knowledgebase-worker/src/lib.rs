use sdkwork_api_knowledgebase_standalone_gateway::shutdown_signal;
use sdkwork_intelligence_knowledgebase_service::{
    ports::knowledge_wiki_persistence::WikiPersistenceScope,
    wiki_backfill::{
        RunWikiPublicationBackfillRequest, WikiPublicationBackfillDisposition,
        MAX_WIKI_BACKFILL_PAGE_SIZE,
    },
};
use sdkwork_routes_knowledgebase_app_api::KnowledgebaseRuntime;

pub mod health;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WikiBackfillMaintenanceConfig {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub actor_id: u64,
    pub page_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceTickResult {
    pub outbox_published: usize,
    pub ingestion_jobs_processed: usize,
    pub provider_migration_phases_processed: usize,
    pub provider_migrations_completed: usize,
    pub provider_migrations_rolled_back: usize,
    pub provider_migrations_failed: usize,
    pub group_archives_processed: usize,
    pub wiki_publications_initialized: usize,
    pub wiki_publications_failed: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceTickError {
    #[error("ingestion job batch failed: {0}")]
    Ingestion(String),
    #[error("Provider migration batch failed: {0}")]
    ProviderMigration(String),
    #[error("Wiki publication compensation batch failed: {0}")]
    WikiBackfill(String),
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
    wiki_backfill: Option<WikiBackfillMaintenanceConfig>,
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
    let (wiki_publications_initialized, wiki_publications_failed) =
        run_wiki_backfill_compensation(runtime, wiki_backfill).await?;
    Ok(MaintenanceTickResult {
        outbox_published,
        ingestion_jobs_processed,
        provider_migration_phases_processed: provider_migrations.processed,
        provider_migrations_completed: provider_migrations.completed,
        provider_migrations_rolled_back: provider_migrations.rolled_back,
        provider_migrations_failed: provider_migrations.failed,
        group_archives_processed,
        wiki_publications_initialized,
        wiki_publications_failed,
    })
}

async fn run_wiki_backfill_compensation(
    runtime: &KnowledgebaseRuntime,
    config: Option<WikiBackfillMaintenanceConfig>,
) -> Result<(usize, usize), MaintenanceTickError> {
    let Some(config) = config else {
        return Ok((0, 0));
    };
    if config.tenant_id == 0
        || config.actor_id == 0
        || config.page_size == 0
        || config.page_size > MAX_WIKI_BACKFILL_PAGE_SIZE
    {
        return Err(MaintenanceTickError::WikiBackfill(
            "maintenance configuration is invalid".to_string(),
        ));
    }

    let result = runtime
        .run_wiki_publication_backfill_page(RunWikiPublicationBackfillRequest {
            scope: WikiPersistenceScope {
                tenant_id: config.tenant_id,
                organization_id: config.organization_id,
            },
            after_space_id: None,
            page_size: config.page_size,
            actor_id: config.actor_id,
            dry_run: false,
        })
        .await
        .map_err(MaintenanceTickError::WikiBackfill)?;
    let initialized = result
        .outcomes
        .iter()
        .filter(|outcome| outcome.disposition == WikiPublicationBackfillDisposition::Initialized)
        .count();
    let failed = result
        .outcomes
        .iter()
        .filter(|outcome| outcome.disposition == WikiPublicationBackfillDisposition::Failed)
        .count();
    Ok((initialized, failed))
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
    wiki_backfill: Option<WikiBackfillMaintenanceConfig>,
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
                    wiki_backfill,
                ).await {
                    Ok(result) if result.outbox_published > 0
                        || result.ingestion_jobs_processed > 0
                        || result.provider_migration_phases_processed > 0
                        || result.group_archives_processed > 0
                        || result.wiki_publications_initialized > 0
                        || result.wiki_publications_failed > 0 => {
                        tracing::info!(
                            outbox_published = result.outbox_published,
                            ingestion_jobs_processed = result.ingestion_jobs_processed,
                            provider_migration_phases_processed = result.provider_migration_phases_processed,
                            provider_migrations_completed = result.provider_migrations_completed,
                            provider_migrations_rolled_back = result.provider_migrations_rolled_back,
                            provider_migrations_failed = result.provider_migrations_failed,
                            group_archives_processed = result.group_archives_processed,
                            wiki_publications_initialized = result.wiki_publications_initialized,
                            wiki_publications_failed = result.wiki_publications_failed,
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
            wiki_publications_initialized: 6,
            wiki_publications_failed: 1,
        };
        assert_eq!(result.outbox_published, 2);
        assert_eq!(result.ingestion_jobs_processed, 3);
        assert_eq!(result.provider_migration_phases_processed, 5);
        assert_eq!(result.provider_migrations_completed, 1);
        assert_eq!(result.provider_migrations_rolled_back, 1);
        assert_eq!(result.provider_migrations_failed, 0);
        assert_eq!(result.group_archives_processed, 4);
        assert_eq!(result.wiki_publications_initialized, 6);
        assert_eq!(result.wiki_publications_failed, 1);
    }
}
