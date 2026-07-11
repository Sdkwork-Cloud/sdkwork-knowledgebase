use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::KNOWLEDGE_UPLOAD_SESSION_TTL;
use sdkwork_intelligence_knowledgebase_service::tenant_quota::{
    TenantQuotaExceeded, TenantQuotaKind,
};
use sdkwork_knowledgebase_observability::KnowledgebaseTenantQuotaLimits;
use sqlx::{Any, AnyConnection, AnyPool, Transaction};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const KNOWLEDGEBASE_QUOTA_LOCK_NAMESPACE: i64 = 0x4B42_5155_4F54_4100;

pub(crate) async fn begin_tenant_quota_transaction(
    pool: &AnyPool,
    database_engine: DatabaseEngine,
    tenant_id: i64,
) -> Result<Transaction<'static, Any>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    match database_engine {
        DatabaseEngine::Postgres => {
            let lock_key = tenant_id ^ KNOWLEDGEBASE_QUOTA_LOCK_NAMESPACE;
            sqlx::query("SELECT pg_advisory_xact_lock($1)")
                .bind(lock_key)
                .execute(&mut *transaction)
                .await?;
        }
        DatabaseEngine::Sqlite => {
            // A write statement acquires SQLite's cross-connection write reservation
            // before quota usage is read. The impossible id keeps the lock rowless.
            sqlx::query(
                "UPDATE kb_ingestion_job SET version = version WHERE tenant_id = $1 AND id = -1",
            )
            .bind(tenant_id)
            .execute(&mut *transaction)
            .await?;
        }
    }
    Ok(transaction)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TenantQuotaTransactionError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Quota(#[from] TenantQuotaExceeded),
    #[error("tenant quota transaction error: {0}")]
    Invalid(String),
}

pub(crate) async fn enforce_tenant_quotas_after_write(
    connection: &mut AnyConnection,
    database_engine: DatabaseEngine,
    tenant_id: i64,
    limits: KnowledgebaseTenantQuotaLimits,
) -> Result<(), TenantQuotaTransactionError> {
    let document_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_document WHERE tenant_id = $1 AND status = 1")
            .bind(tenant_id)
            .fetch_one(&mut *connection)
            .await?;
    let document_count = u64::try_from(document_count.max(0))
        .map_err(|error| TenantQuotaTransactionError::Invalid(error.to_string()))?;
    if document_count > limits.max_documents {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::Documents,
            usage: document_count,
            limit: limits.max_documents,
        }
        .into());
    }

    let cutoff = OffsetDateTime::now_utc()
        .checked_sub(KNOWLEDGE_UPLOAD_SESSION_TTL)
        .ok_or_else(|| {
            TenantQuotaTransactionError::Invalid(
                "upload session quota cutoff is outside the supported timestamp range".to_string(),
            )
        })?
        .format(&Rfc3339)
        .map_err(|error| TenantQuotaTransactionError::Invalid(error.to_string()))?;
    let cutoff_expr = match database_engine {
        DatabaseEngine::Sqlite => "$6",
        DatabaseEngine::Postgres => "CAST($6 AS TIMESTAMP)",
    };
    let query = format!(
        r#"
        SELECT COUNT(*)
        FROM kb_ingestion_job
        WHERE tenant_id = $1
          AND status = $2
          AND state IN ($3, $4)
          AND NOT (job_type = $5 AND created_at <= {cutoff_expr})
        "#,
    );
    let inflight_count: i64 = sqlx::query_scalar(&query)
        .bind(tenant_id)
        .bind(1_i64)
        .bind(0_i64)
        .bind(1_i64)
        .bind("upload_session")
        .bind(cutoff)
        .fetch_one(&mut *connection)
        .await?;
    let inflight_count = u64::try_from(inflight_count.max(0))
        .map_err(|error| TenantQuotaTransactionError::Invalid(error.to_string()))?;
    if inflight_count > u64::from(limits.max_concurrent_ingest_jobs) {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::IngestConcurrency,
            usage: inflight_count,
            limit: u64::from(limits.max_concurrent_ingest_jobs),
        }
        .into());
    }

    let storage_bytes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(size_bytes), 0) FROM kb_drive_object_ref WHERE tenant_id = $1 AND status = 1",
    )
    .bind(tenant_id)
    .fetch_one(connection)
    .await?;
    let storage_bytes = u64::try_from(storage_bytes.max(0))
        .map_err(|error| TenantQuotaTransactionError::Invalid(error.to_string()))?;
    if storage_bytes > limits.max_storage_bytes {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::StorageBytes,
            usage: storage_bytes,
            limit: limits.max_storage_bytes,
        }
        .into());
    }

    Ok(())
}
