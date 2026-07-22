use sdkwork_api_knowledgebase_standalone_gateway::shutdown_signal;
use sdkwork_intelligence_knowledgebase_service::{
    ports::knowledge_wiki_persistence::WikiPersistenceScope,
    wiki_backfill::{
        RunWikiPublicationBackfillRequest, WikiPublicationBackfillDisposition,
        MAX_WIKI_BACKFILL_PAGE_SIZE,
    },
    wiki_event_consumer::{
        KnowledgeWikiDriveCheckpointPageResult, ProcessKnowledgeWikiDriveCheckpointPageRequest,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WikiDriveEventMaintenanceConfig {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub actor_id: u64,
    pub checkpoint_page_size: u32,
    pub event_batch_size: u32,
    pub lease_seconds: u64,
    pub retry_delay_seconds: u64,
    pub max_attempts: u32,
    pub delivery_renewal_page_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceConfig {
    pub worker_id: String,
    pub ingestion_job_lease: time::Duration,
    pub provider_migration_lease: std::time::Duration,
    pub outbox_limit: u32,
    pub ingestion_job_limit: u32,
    pub provider_migration_limit: u32,
    pub group_archive_limit: u32,
    pub wiki_backfill: Option<WikiBackfillMaintenanceConfig>,
    pub wiki_drive_events: WikiDriveEventMaintenanceConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MaintenanceTickState {
    pub wiki_checkpoint_cursor: Option<u64>,
    pub wiki_delivery_cursor: Option<u64>,
    pub renew_wiki_event_deliveries: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenancePollingConfig {
    pub interval_ms: u64,
    pub maintenance: MaintenanceConfig,
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
    pub wiki_drive_outbox_events_processed: usize,
    pub wiki_drive_outbox_events_delivered: usize,
    pub wiki_drive_outbox_events_failed: usize,
    pub wiki_drive_checkpoints_processed: usize,
    pub wiki_drive_events_applied: usize,
    pub wiki_drive_events_retried: usize,
    pub wiki_drive_events_dead_lettered: usize,
    pub wiki_drive_public_changes: usize,
    pub wiki_drive_next_after_checkpoint_id: Option<u64>,
    pub wiki_drive_event_deliveries_renewed: usize,
    pub wiki_drive_event_delivery_relays_verified: usize,
    pub wiki_drive_event_delivery_failures: usize,
    pub wiki_drive_next_after_event_delivery_checkpoint_id: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum MaintenanceTickError {
    #[error("ingestion job batch failed: {0}")]
    Ingestion(String),
    #[error("Provider migration batch failed: {0}")]
    ProviderMigration(String),
    #[error("Wiki publication compensation batch failed: {0}")]
    WikiBackfill(String),
    #[error("Wiki Drive event batch failed: {0}")]
    WikiDriveEvents(String),
}

pub async fn run_maintenance_tick(
    runtime: &KnowledgebaseRuntime,
    config: &MaintenanceConfig,
    state: MaintenanceTickState,
) -> Result<MaintenanceTickResult, MaintenanceTickError> {
    let outbox_published = runtime
        .publish_pending_outbox_events(config.outbox_limit)
        .await;
    let ingestion_jobs_processed = runtime
        .process_queued_ingestion_jobs(
            &config.worker_id,
            config.ingestion_job_lease,
            config.ingestion_job_limit,
        )
        .await
        .map_err(MaintenanceTickError::Ingestion)?;
    let provider_migrations = runtime
        .process_provider_migrations(
            &config.worker_id,
            config.provider_migration_lease,
            config.provider_migration_limit,
        )
        .await
        .map_err(MaintenanceTickError::ProviderMigration)?;
    let group_archives_processed = runtime
        .process_resumable_group_space_archives(config.group_archive_limit)
        .await;
    let (wiki_publications_initialized, wiki_publications_failed) =
        run_wiki_backfill_compensation(runtime, config.wiki_backfill).await?;
    let wiki_drive_relay_result = runtime
        .relay_embedded_wiki_drive_outbox_events()
        .await
        .map_err(MaintenanceTickError::WikiDriveEvents)?;
    let wiki_drive_result = run_wiki_drive_event_maintenance(
        runtime,
        &config.worker_id,
        config.wiki_drive_events,
        state.wiki_checkpoint_cursor,
    )
    .await?;
    let wiki_delivery_result = if state.renew_wiki_event_deliveries {
        runtime
            .renew_wiki_drive_event_delivery_page(
                sdkwork_intelligence_knowledgebase_service::wiki_event_delivery::RenewWikiDriveEventDeliveryPageRequest {
                    scope: WikiPersistenceScope {
                        tenant_id: config.wiki_drive_events.tenant_id,
                        organization_id: config.wiki_drive_events.organization_id,
                    },
                    after_checkpoint_id: state.wiki_delivery_cursor,
                    limit: config.wiki_drive_events.delivery_renewal_page_size,
                },
            )
            .await
            .map_err(MaintenanceTickError::WikiDriveEvents)?
    } else {
        sdkwork_intelligence_knowledgebase_service::wiki_event_delivery::WikiDriveEventDeliveryRenewalPageResult {
            checkpoints_scanned: 0,
            cloud_deliveries_renewed: 0,
            embedded_relays_verified: 0,
            failures: Vec::new(),
            next_after_checkpoint_id: state.wiki_delivery_cursor,
        }
    };
    for failure in &wiki_delivery_result.failures {
        tracing::warn!(
            target: "sdkwork.knowledgebase.wiki",
            event = "knowledgebase.wiki.drive_event_delivery_renewal_failed",
            checkpoint_id = failure.checkpoint_id,
            source_scope_uuid = %failure.source_scope_uuid,
            error_code = %failure.error_code,
            retry_scheduled = true,
            retry_policy = "next_renewal_scan",
            "Wiki Drive event delivery renewal failed and remains eligible for the next bounded scan"
        );
    }
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
        wiki_drive_outbox_events_processed: wiki_drive_relay_result.processed,
        wiki_drive_outbox_events_delivered: wiki_drive_relay_result.delivered,
        wiki_drive_outbox_events_failed: wiki_drive_relay_result.failed,
        wiki_drive_checkpoints_processed: wiki_drive_result.checkpoints_processed,
        wiki_drive_events_applied: wiki_drive_result.events.applied,
        wiki_drive_events_retried: wiki_drive_result.events.retried,
        wiki_drive_events_dead_lettered: wiki_drive_result.events.dead_lettered,
        wiki_drive_public_changes: wiki_drive_result.events.public_changes,
        wiki_drive_next_after_checkpoint_id: wiki_drive_result.next_after_checkpoint_id,
        wiki_drive_event_deliveries_renewed: wiki_delivery_result.cloud_deliveries_renewed,
        wiki_drive_event_delivery_relays_verified: wiki_delivery_result.embedded_relays_verified,
        wiki_drive_event_delivery_failures: wiki_delivery_result.failures.len(),
        wiki_drive_next_after_event_delivery_checkpoint_id: wiki_delivery_result
            .next_after_checkpoint_id,
    })
}

async fn run_wiki_drive_event_maintenance(
    runtime: &KnowledgebaseRuntime,
    worker_id: &str,
    config: WikiDriveEventMaintenanceConfig,
    after_checkpoint_id: Option<u64>,
) -> Result<KnowledgeWikiDriveCheckpointPageResult, MaintenanceTickError> {
    if config.tenant_id == 0
        || config.checkpoint_page_size == 0
        || config.checkpoint_page_size > 200
        || config.event_batch_size == 0
        || config.event_batch_size > 100
        || config.actor_id == 0
        || config.lease_seconds == 0
        || config.lease_seconds > 3_600
        || config.retry_delay_seconds == 0
        || config.retry_delay_seconds > 86_400
        || config.max_attempts == 0
        || config.max_attempts > 100
        || config.delivery_renewal_page_size == 0
        || config.delivery_renewal_page_size > 200
    {
        return Err(MaintenanceTickError::WikiDriveEvents(
            "Wiki Drive event maintenance configuration is invalid".to_string(),
        ));
    }
    runtime
        .process_wiki_drive_event_checkpoint_page(ProcessKnowledgeWikiDriveCheckpointPageRequest {
            scope: WikiPersistenceScope {
                tenant_id: config.tenant_id,
                organization_id: config.organization_id,
            },
            after_checkpoint_id,
            worker_id: worker_id.to_string(),
            actor_id: config.actor_id,
            lease_seconds: config.lease_seconds,
            checkpoint_limit: config.checkpoint_page_size,
            event_limit_per_checkpoint: config.event_batch_size,
            retry_delay_seconds: config.retry_delay_seconds,
            max_attempts: config.max_attempts,
        })
        .await
        .map_err(MaintenanceTickError::WikiDriveEvents)
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

pub async fn run_polling_loop(runtime: KnowledgebaseRuntime, config: MaintenancePollingConfig) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(
        config.interval_ms.max(250),
    ));
    let mut wiki_checkpoint_cursor = None;
    let mut wiki_delivery_cursor = None;
    let renewal_interval = std::time::Duration::from_secs(
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_WIKI_EVENT_DELIVERY_RENEWAL_INTERVAL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (60..=86_400).contains(value))
            .unwrap_or(3_600),
    );
    let mut last_delivery_renewal = std::time::Instant::now()
        .checked_sub(renewal_interval)
        .unwrap_or_else(std::time::Instant::now);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let renew_wiki_event_deliveries = last_delivery_renewal.elapsed() >= renewal_interval;
                if renew_wiki_event_deliveries {
                    last_delivery_renewal = std::time::Instant::now();
                }
                match run_maintenance_tick(
                    &runtime,
                    &config.maintenance,
                    MaintenanceTickState {
                        wiki_checkpoint_cursor,
                        wiki_delivery_cursor,
                        renew_wiki_event_deliveries,
                    },
                ).await {
                    Ok(result) => {
                        wiki_checkpoint_cursor = result.wiki_drive_next_after_checkpoint_id;
                        wiki_delivery_cursor = result.wiki_drive_next_after_event_delivery_checkpoint_id;
                        if result.outbox_published > 0
                            || result.ingestion_jobs_processed > 0
                            || result.provider_migration_phases_processed > 0
                            || result.group_archives_processed > 0
                            || result.wiki_publications_initialized > 0
                            || result.wiki_publications_failed > 0
                            || result.wiki_drive_outbox_events_processed > 0
                            || result.wiki_drive_outbox_events_delivered > 0
                            || result.wiki_drive_outbox_events_failed > 0
                            || result.wiki_drive_checkpoints_processed > 0
                            || result.wiki_drive_events_applied > 0
                            || result.wiki_drive_events_retried > 0
                            || result.wiki_drive_events_dead_lettered > 0
                            || result.wiki_drive_public_changes > 0
                            || result.wiki_drive_event_deliveries_renewed > 0
                            || result.wiki_drive_event_delivery_relays_verified > 0
                            || result.wiki_drive_event_delivery_failures > 0
                        {
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
                                wiki_drive_outbox_events_processed = result.wiki_drive_outbox_events_processed,
                                wiki_drive_outbox_events_delivered = result.wiki_drive_outbox_events_delivered,
                                wiki_drive_outbox_events_failed = result.wiki_drive_outbox_events_failed,
                                wiki_drive_checkpoints_processed = result.wiki_drive_checkpoints_processed,
                                wiki_drive_events_applied = result.wiki_drive_events_applied,
                                wiki_drive_events_retried = result.wiki_drive_events_retried,
                                wiki_drive_events_dead_lettered = result.wiki_drive_events_dead_lettered,
                                wiki_drive_public_changes = result.wiki_drive_public_changes,
                                wiki_drive_event_deliveries_renewed = result.wiki_drive_event_deliveries_renewed,
                                wiki_drive_event_delivery_relays_verified = result.wiki_drive_event_delivery_relays_verified,
                                wiki_drive_event_delivery_failures = result.wiki_drive_event_delivery_failures,
                                "knowledgebase worker maintenance tick"
                            );
                        }
                    }
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
            wiki_drive_outbox_events_processed: 4,
            wiki_drive_outbox_events_delivered: 3,
            wiki_drive_outbox_events_failed: 1,
            wiki_drive_checkpoints_processed: 2,
            wiki_drive_events_applied: 3,
            wiki_drive_events_retried: 1,
            wiki_drive_events_dead_lettered: 0,
            wiki_drive_public_changes: 2,
            wiki_drive_next_after_checkpoint_id: Some(9),
            wiki_drive_event_deliveries_renewed: 1,
            wiki_drive_event_delivery_relays_verified: 0,
            wiki_drive_event_delivery_failures: 0,
            wiki_drive_next_after_event_delivery_checkpoint_id: Some(10),
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
        assert_eq!(result.wiki_drive_outbox_events_processed, 4);
        assert_eq!(result.wiki_drive_outbox_events_delivered, 3);
        assert_eq!(result.wiki_drive_outbox_events_failed, 1);
        assert_eq!(result.wiki_drive_checkpoints_processed, 2);
        assert_eq!(result.wiki_drive_events_applied, 3);
        assert_eq!(result.wiki_drive_events_retried, 1);
        assert_eq!(result.wiki_drive_events_dead_lettered, 0);
        assert_eq!(result.wiki_drive_public_changes, 2);
        assert_eq!(result.wiki_drive_next_after_checkpoint_id, Some(9));
        assert_eq!(result.wiki_drive_event_deliveries_renewed, 1);
        assert_eq!(result.wiki_drive_event_delivery_relays_verified, 0);
        assert_eq!(result.wiki_drive_event_delivery_failures, 0);
        assert_eq!(
            result.wiki_drive_next_after_event_delivery_checkpoint_id,
            Some(10)
        );
    }
}
