use sdkwork_api_knowledgebase_standalone_gateway::init_tracing;
use sdkwork_knowledgebase_contract::parse_canonical_positive_signed_i64;
use sdkwork_knowledgebase_worker::{health, run_polling_loop};
use sdkwork_routes_knowledgebase_app_api::{bootstrap, KnowledgebaseRuntime};

#[tokio::main]
async fn main() {
    bootstrap::validate_process_config();
    init_tracing("worker");

    let database_url = bootstrap::resolve_database_url();
    let tenant_id = required_worker_tenant_id();
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
    let ingestion_job_lease_seconds =
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_INGESTION_JOB_LEASE_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (30..=3_600).contains(value))
            .unwrap_or(300);
    let provider_migration_limit =
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_BATCH_SIZE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| (1..=200).contains(value))
            .unwrap_or(25);
    let provider_migration_lease_seconds =
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_PROVIDER_MIGRATION_LEASE_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| (5..=3_600).contains(value))
            .unwrap_or(120);
    let worker_id = resolve_worker_id();
    let group_archive_limit =
        std::env::var("SDKWORK_KNOWLEDGEBASE_WORKER_GROUP_ARCHIVE_BATCH_SIZE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| (1..=200).contains(value))
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
        database_engine = database_engine_label(&database_url),
        tenant_id,
        interval_ms,
        outbox_limit,
        ingestion_job_limit,
        ingestion_job_lease_seconds,
        provider_migration_limit,
        provider_migration_lease_seconds,
        worker_id = %worker_id,
        group_archive_limit,
        %health_addr,
        "starting knowledgebase worker loop"
    );

    let readiness = runtime.readiness_check_adapter();
    let health_addr_for_task = health_addr.clone();
    tokio::spawn(async move {
        health::serve_worker_health(&health_addr_for_task, readiness).await;
    });

    run_polling_loop(
        runtime,
        worker_id,
        time::Duration::seconds(ingestion_job_lease_seconds as i64),
        std::time::Duration::from_secs(provider_migration_lease_seconds),
        interval_ms,
        outbox_limit,
        ingestion_job_limit,
        provider_migration_limit,
        group_archive_limit,
    )
    .await;
}

fn resolve_worker_id() -> String {
    for key in [
        "SDKWORK_KNOWLEDGEBASE_WORKER_ID",
        "POD_UID",
        "HOSTNAME",
        "COMPUTERNAME",
    ] {
        if let Ok(value) = std::env::var(key) {
            let value = value.trim();
            if !value.is_empty() && value.chars().count() <= 255 {
                return value.to_string();
            }
        }
    }
    format!("knowledgebase-worker-process-{}", std::process::id())
}

fn required_worker_tenant_id() -> u64 {
    let value = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .expect("SDKWORK_KNOWLEDGEBASE_TENANT_ID is required for the knowledgebase worker");
    parse_canonical_positive_signed_i64(&value)
        .expect("SDKWORK_KNOWLEDGEBASE_TENANT_ID must be a canonical positive signed BIGINT")
}

fn database_engine_label(database_url: &str) -> &'static str {
    let normalized = database_url.trim().to_ascii_lowercase();
    if normalized.starts_with("postgres://") || normalized.starts_with("postgresql://") {
        "postgres"
    } else if normalized.starts_with("sqlite:") {
        "sqlite"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::{database_engine_label, required_worker_tenant_id, resolve_worker_id};

    #[test]
    fn worker_tenant_id_requires_a_canonical_positive_signed_bigint() {
        for invalid in ["0", "01", "+1", " 1", "1 ", "tenant", "9223372036854775808"] {
            std::env::set_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID", invalid);
            assert!(
                std::panic::catch_unwind(required_worker_tenant_id).is_err(),
                "{invalid} must be rejected"
            );
        }
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID", "9223372036854775807");
        assert_eq!(required_worker_tenant_id(), i64::MAX as u64);
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID");
    }

    #[test]
    fn worker_log_database_label_never_contains_a_connection_secret() {
        assert_eq!(
            database_engine_label("postgres://user:secret@db.internal/knowledge"),
            "postgres"
        );
        assert_eq!(
            database_engine_label("sqlite://data/knowledgebase.db?mode=rwc"),
            "sqlite"
        );
    }

    #[test]
    fn worker_identity_prefers_explicit_stable_identity() {
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_WORKER_ID", "pod-uid-123");
        assert_eq!(resolve_worker_id(), "pod-uid-123");
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_WORKER_ID");
    }
}
