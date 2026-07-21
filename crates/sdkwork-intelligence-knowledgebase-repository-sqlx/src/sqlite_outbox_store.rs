use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::{
    AppendOutboxEventRecord, KnowledgeOutboxStore, KnowledgeOutboxStoreError, PendingOutboxEvent,
};
use sdkwork_utils_rust::{is_blank, truncate};
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::sql_timestamp::SqlTimestampDialect;
use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const OUTBOX_STATUS_PENDING: i64 = 0;
const OUTBOX_STATUS_PUBLISHED: i64 = 1;
const OUTBOX_STATUS_FAILED: i64 = 2;
const OUTBOX_STATUS_CLAIMED: i64 = 3;
const INITIAL_VERSION: i64 = 0;
const DEFAULT_STALE_CLAIM_SECS: u64 = 300;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeOutboxStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    use_postgres_skip_locked_claim: bool,
    timestamp_dialect: SqlTimestampDialect,
}

impl SqliteKnowledgeOutboxStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_postgres_skip_locked_claim(mut self, enabled: bool) -> Self {
        self.use_postgres_skip_locked_claim = enabled;
        self
    }

    pub fn with_database_engine(mut self, database_engine: DatabaseEngine) -> Self {
        self.timestamp_dialect = SqlTimestampDialect::from_database_engine(database_engine);
        self
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            id_generator,
            use_postgres_skip_locked_claim: false,
            timestamp_dialect: SqlTimestampDialect::default(),
        }
    }
}

#[async_trait]
impl KnowledgeOutboxStore for SqliteKnowledgeOutboxStore {
    async fn append_event(
        &self,
        record: AppendOutboxEventRecord,
    ) -> Result<(), KnowledgeOutboxStoreError> {
        if is_blank(Some(record.aggregate_type.as_str())) {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "aggregate_type is required".to_string(),
            ));
        }
        if is_blank(Some(record.event_type.as_str())) {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "event_type is required".to_string(),
            ));
        }
        if is_blank(Some(record.payload_json.as_str())) {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "payload_json is required".to_string(),
            ));
        }

        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let aggregate_id = to_i64("aggregate_id", record.aggregate_id)?;
        let now = now_rfc3339()?;
        let payload_expr = self.timestamp_dialect.sql_json_expr("$7");
        let created_at_expr = self.timestamp_dialect.sql_timestamp_expr("$9");

        let query = format!(
            r#"
            INSERT INTO kb_outbox_event (
                id, uuid, tenant_id, aggregate_type, aggregate_id, event_type,
                payload, status, created_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, {payload_expr}, $8, {created_at_expr}, $10)
            "#,
        );
        sqlx::query(&query)
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(record.aggregate_type)
            .bind(aggregate_id)
            .bind(record.event_type)
            .bind(record.payload_json)
            .bind(OUTBOX_STATUS_PENDING)
            .bind(now)
            .bind(INITIAL_VERSION)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;

        Ok(())
    }

    async fn list_pending_events(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let limit = i64::from(limit.clamp(1, 200));
        let rows = sqlx::query(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, CAST(payload AS TEXT) AS payload
            FROM kb_outbox_event
            WHERE tenant_id = $1 AND status = $2
            ORDER BY created_at ASC, id ASC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(OUTBOX_STATUS_PENDING)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(PendingOutboxEvent {
                    id: row.try_get::<i64, _>("id").map_err(sqlx_error)? as u64,
                    aggregate_type: row.try_get("aggregate_type").map_err(sqlx_error)?,
                    aggregate_id: row.try_get::<i64, _>("aggregate_id").map_err(sqlx_error)? as u64,
                    event_type: row.try_get("event_type").map_err(sqlx_error)?,
                    payload_json: row.try_get("payload").map_err(sqlx_error)?,
                })
            })
            .collect()
    }

    async fn claim_pending_events(
        &self,
        limit: u32,
    ) -> Result<Vec<PendingOutboxEvent>, KnowledgeOutboxStoreError> {
        let _ = self
            .release_stale_claimed_events(DEFAULT_STALE_CLAIM_SECS)
            .await?;

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let limit = i64::from(limit.clamp(1, 200));
        let now = now_rfc3339()?;
        let claimed_at_expr = self.timestamp_dialect.sql_timestamp_expr("$2");

        let rows = if self.use_postgres_skip_locked_claim {
            let query = format!(
                r#"
                UPDATE kb_outbox_event
                SET status = $1, claimed_at = {claimed_at_expr}, version = version + 1
                WHERE id IN (
                    SELECT id
                    FROM kb_outbox_event
                    WHERE tenant_id = $3 AND status = $4
                    ORDER BY created_at ASC, id ASC
                    LIMIT $5
                    FOR UPDATE SKIP LOCKED
                )
                RETURNING id, aggregate_type, aggregate_id, event_type, CAST(payload AS TEXT) AS payload
                "#,
            );
            sqlx::query(&query)
                .bind(OUTBOX_STATUS_CLAIMED)
                .bind(&now)
                .bind(tenant_id)
                .bind(OUTBOX_STATUS_PENDING)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_error)?
        } else {
            let query = format!(
                r#"
                UPDATE kb_outbox_event
                SET status = $1, claimed_at = {claimed_at_expr}, version = version + 1
                WHERE id IN (
                    SELECT id
                    FROM kb_outbox_event
                    WHERE tenant_id = $3 AND status = $4
                    ORDER BY created_at ASC, id ASC
                    LIMIT $5
                )
                RETURNING id, aggregate_type, aggregate_id, event_type, CAST(payload AS TEXT) AS payload
                "#,
            );
            sqlx::query(&query)
                .bind(OUTBOX_STATUS_CLAIMED)
                .bind(&now)
                .bind(tenant_id)
                .bind(OUTBOX_STATUS_PENDING)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(sqlx_error)?
        };

        rows.into_iter()
            .map(|row| {
                Ok(PendingOutboxEvent {
                    id: row.try_get::<i64, _>("id").map_err(sqlx_error)? as u64,
                    aggregate_type: row.try_get("aggregate_type").map_err(sqlx_error)?,
                    aggregate_id: row.try_get::<i64, _>("aggregate_id").map_err(sqlx_error)? as u64,
                    event_type: row.try_get("event_type").map_err(sqlx_error)?,
                    payload_json: row.try_get("payload").map_err(sqlx_error)?,
                })
            })
            .collect()
    }

    async fn release_stale_claimed_events(
        &self,
        stale_after_secs: u64,
    ) -> Result<usize, KnowledgeOutboxStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let cutoff = OffsetDateTime::now_utc()
            - time::Duration::seconds(i64::try_from(stale_after_secs).unwrap_or(300));
        let cutoff = cutoff
            .format(&Rfc3339)
            .map_err(|error| KnowledgeOutboxStoreError::Internal(error.to_string()))?;
        let cutoff_expr = self.timestamp_dialect.sql_timestamp_expr("$4");

        let query = format!(
            r#"
            UPDATE kb_outbox_event
            SET status = $1, claimed_at = NULL, version = version + 1
            WHERE tenant_id = $2
              AND status = $3
              AND claimed_at IS NOT NULL
              AND claimed_at < {cutoff_expr}
            "#,
        );
        let updated = sqlx::query(&query)
            .bind(OUTBOX_STATUS_PENDING)
            .bind(tenant_id)
            .bind(OUTBOX_STATUS_CLAIMED)
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;

        Ok(updated.rows_affected() as usize)
    }

    async fn mark_published(&self, event_id: u64) -> Result<(), KnowledgeOutboxStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let event_id = to_i64("event_id", event_id)?;
        let now = now_rfc3339()?;
        let published_at_expr = self.timestamp_dialect.sql_timestamp_expr("$2");
        let query = format!(
            r#"
            UPDATE kb_outbox_event
            SET status = $1, published_at = {published_at_expr}, claimed_at = NULL, version = version + 1
            WHERE tenant_id = $3 AND id = $4 AND status = $5
            "#,
        );
        let updated = sqlx::query(&query)
            .bind(OUTBOX_STATUS_PUBLISHED)
            .bind(now)
            .bind(tenant_id)
            .bind(event_id)
            .bind(OUTBOX_STATUS_CLAIMED)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;

        if updated.rows_affected() == 0 {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(format!(
                "outbox event was not claimed: {event_id}"
            )));
        }
        Ok(())
    }

    async fn mark_failed(
        &self,
        event_id: u64,
        error_message: &str,
    ) -> Result<(), KnowledgeOutboxStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let event_id = to_i64("event_id", event_id)?;
        let truncated_error = truncate_outbox_error(error_message);
        let updated = sqlx::query(
            r#"
            UPDATE kb_outbox_event
            SET status = $1,
                last_error = $2,
                claimed_at = NULL,
                retry_count = retry_count + 1,
                version = version + 1
            WHERE tenant_id = $3 AND id = $4 AND status IN ($5, $6)
            "#,
        )
        .bind(OUTBOX_STATUS_FAILED)
        .bind(truncated_error)
        .bind(tenant_id)
        .bind(event_id)
        .bind(OUTBOX_STATUS_PENDING)
        .bind(OUTBOX_STATUS_CLAIMED)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if updated.rows_affected() == 0 {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(format!(
                "outbox event was not pending or claimed: {event_id}"
            )));
        }
        Ok(())
    }

    async fn requeue_failed_events(
        &self,
        limit: u32,
        max_retry_count: u32,
    ) -> Result<usize, KnowledgeOutboxStoreError> {
        const OUTBOX_STATUS_PENDING: i64 = 0;
        const OUTBOX_STATUS_FAILED: i64 = 2;

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let limit = i64::from(limit.clamp(1, 200));
        let max_retry_count = i64::from(max_retry_count);
        let updated = sqlx::query(
            r#"
            UPDATE kb_outbox_event
            SET status = $1, version = version + 1
            WHERE tenant_id = $2
              AND status = $3
              AND retry_count < $4
              AND id IN (
                SELECT id
                FROM kb_outbox_event
                WHERE tenant_id = $2
                  AND status = $3
                  AND retry_count < $4
                ORDER BY created_at ASC, id ASC
                LIMIT $5
              )
            "#,
        )
        .bind(OUTBOX_STATUS_PENDING)
        .bind(tenant_id)
        .bind(OUTBOX_STATUS_FAILED)
        .bind(max_retry_count)
        .bind(limit)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;

        Ok(updated.rows_affected() as usize)
    }
}

fn truncate_outbox_error(error_message: &str) -> String {
    const MAX_OUTBOX_ERROR_LEN: usize = 1024;
    truncate(error_message, MAX_OUTBOX_ERROR_LEN, Some(""))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOutboxStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeOutboxStoreError::InvalidRequest(format!("{field} exceeds i64 range: {value}"))
    })
}

fn now_rfc3339() -> Result<String, KnowledgeOutboxStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOutboxStoreError::Internal(error.to_string()))
}

fn id_error(error: crate::id::KnowledgeIdGeneratorError) -> KnowledgeOutboxStoreError {
    KnowledgeOutboxStoreError::Internal(error.to_string())
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeOutboxStoreError {
    KnowledgeOutboxStoreError::Internal(error.to_string())
}

#[cfg(test)]
mod truncation_tests {
    use super::truncate_outbox_error;

    #[test]
    fn truncates_unicode_error_without_splitting_utf8() {
        let message = "上游错误".repeat(400);
        let truncated = truncate_outbox_error(&message);

        assert_eq!(truncated.chars().count(), 1024);
        assert!(message.starts_with(&truncated));
    }
}
