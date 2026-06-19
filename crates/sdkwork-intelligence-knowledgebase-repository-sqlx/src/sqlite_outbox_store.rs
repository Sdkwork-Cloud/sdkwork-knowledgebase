use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::{
    AppendOutboxEventRecord, KnowledgeOutboxStore, KnowledgeOutboxStoreError, PendingOutboxEvent,
};
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const OUTBOX_STATUS_PENDING: i64 = 0;
const OUTBOX_STATUS_PUBLISHED: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeOutboxStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeOutboxStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
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
        }
    }
}

#[async_trait]
impl KnowledgeOutboxStore for SqliteKnowledgeOutboxStore {
    async fn append_event(
        &self,
        record: AppendOutboxEventRecord,
    ) -> Result<(), KnowledgeOutboxStoreError> {
        if record.aggregate_type.trim().is_empty() {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "aggregate_type is required".to_string(),
            ));
        }
        if record.event_type.trim().is_empty() {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "event_type is required".to_string(),
            ));
        }
        if record.payload_json.trim().is_empty() {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(
                "payload_json is required".to_string(),
            ));
        }

        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let aggregate_id = to_i64("aggregate_id", record.aggregate_id)?;
        let now = now_rfc3339()?;

        sqlx::query(
            r#"
            INSERT INTO kb_outbox_event (
                id, uuid, tenant_id, aggregate_type, aggregate_id, event_type,
                payload, status, created_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
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
            SELECT id, aggregate_type, aggregate_id, event_type, payload
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

    async fn mark_published(&self, event_id: u64) -> Result<(), KnowledgeOutboxStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let event_id = to_i64("event_id", event_id)?;
        let now = now_rfc3339()?;
        let updated = sqlx::query(
            r#"
            UPDATE kb_outbox_event
            SET status = $1, published_at = $2, version = version + 1
            WHERE tenant_id = $3 AND id = $4 AND status = $5
            "#,
        )
        .bind(OUTBOX_STATUS_PUBLISHED)
        .bind(now)
        .bind(tenant_id)
        .bind(event_id)
        .bind(OUTBOX_STATUS_PENDING)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if updated.rows_affected() == 0 {
            return Err(KnowledgeOutboxStoreError::InvalidRequest(format!(
                "outbox event was not pending: {event_id}"
            )));
        }
        Ok(())
    }
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
