use serde_json::Value;
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use sdkwork_utils_rust::is_blank;

use crate::db::sql_timestamp::utc_sql_timestamp_text;
use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct KnowledgeAuditEventRecord {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: String,
    pub resource_type: String,
    pub resource_id: Option<u64>,
    pub result: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub payload: Option<Value>,
    pub created_at: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum KnowledgeAuditEventStoreError {
    #[error("invalid audit event: {0}")]
    InvalidRequest(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("id generation error: {0}")]
    IdGeneration(String),
}

pub trait KnowledgeAuditEventStore: Send + Sync {
    fn record(&self, event: KnowledgeAuditEventRecord);
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeAuditEventStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeAuditEventStore {
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

    pub async fn append_event(
        &self,
        event: KnowledgeAuditEventRecord,
    ) -> Result<(), KnowledgeAuditEventStoreError> {
        if is_blank(Some(event.event_type.as_str())) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(
                "event_type is required".to_string(),
            ));
        }
        if is_blank(Some(event.actor_type.as_str())) || is_blank(Some(event.actor_id.as_str())) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(
                "actor_type and actor_id are required".to_string(),
            ));
        }
        if is_blank(Some(event.resource_type.as_str())) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(
                "resource_type is required".to_string(),
            ));
        }

        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeAuditEventStoreError::IdGeneration(error.to_string()))?;
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let resource_id = event
            .resource_id
            .map(i64::try_from)
            .transpose()
            .map_err(|_| {
                KnowledgeAuditEventStoreError::InvalidRequest("resource_id".to_string())
            })?;
        let payload = event
            .payload
            .as_ref()
            .map(ToString::to_string);
        let now = utc_sql_timestamp_text()
            .map_err(|error| KnowledgeAuditEventStoreError::InvalidRequest(error))?;

        sqlx::query(
            r#"
            INSERT INTO kb_audit_event (
                id, uuid, tenant_id, event_type, actor_type, actor_id,
                resource_type, resource_id, result, request_id, trace_id,
                payload, created_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CAST($12 AS JSONB), CAST($13 AS TIMESTAMP), $14)
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(event.event_type)
        .bind(event.actor_type)
        .bind(event.actor_id)
        .bind(event.resource_type)
        .bind(resource_id)
        .bind(event.result)
        .bind(event.request_id)
        .bind(event.trace_id)
        .bind(payload)
        .bind(now)
        .bind(INITIAL_VERSION)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_events_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
    ) -> Result<Vec<KnowledgeAuditEventRecord>, KnowledgeAuditEventStoreError> {
        if is_blank(Some(actor_id)) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(
                "actor_id is required".to_string(),
            ));
        }
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let limit = i64::from(limit.clamp(1, 5_000));
        let rows = sqlx::query(
            r#"
            SELECT id, uuid, event_type, actor_type, actor_id, resource_type, resource_id,
                   result, request_id, trace_id, created_at
            FROM kb_audit_event
            WHERE tenant_id = $1 AND actor_id = $2
            ORDER BY created_at ASC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(actor_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let id = row
                    .try_get::<i64, _>("id")
                    .map_err(|error| KnowledgeAuditEventStoreError::Database(error))?;
                let created_at = row
                    .try_get::<String, _>("created_at")
                    .map_err(|error| KnowledgeAuditEventStoreError::Database(error))?;
                Ok(KnowledgeAuditEventRecord {
                    id: Some(id),
                    uuid: row.try_get("uuid").ok(),
                    event_type: row.try_get("event_type").map_err(KnowledgeAuditEventStoreError::Database)?,
                    actor_type: row.try_get("actor_type").map_err(KnowledgeAuditEventStoreError::Database)?,
                    actor_id: row.try_get("actor_id").map_err(KnowledgeAuditEventStoreError::Database)?,
                    resource_type: row.try_get("resource_type").map_err(KnowledgeAuditEventStoreError::Database)?,
                    resource_id: row
                        .try_get::<Option<i64>, _>("resource_id")
                        .map_err(KnowledgeAuditEventStoreError::Database)?
                        .map(|value| value as u64),
                    result: row.try_get("result").map_err(KnowledgeAuditEventStoreError::Database)?,
                    request_id: row.try_get("request_id").ok(),
                    trace_id: row.try_get("trace_id").ok(),
                    payload: None,
                    created_at: Some(created_at),
                })
            })
            .collect()
    }

    pub async fn anonymize_actor(
        &self,
        actor_id: &str,
    ) -> Result<u64, KnowledgeAuditEventStoreError> {
        if is_blank(Some(actor_id)) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(
                "actor_id is required".to_string(),
            ));
        }
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let result = sqlx::query(
            r#"
            UPDATE kb_audit_event
            SET actor_id = 'gdpr-redacted', actor_type = 'system'
            WHERE tenant_id = $1 AND actor_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(actor_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

impl KnowledgeAuditEventStore for SqliteKnowledgeAuditEventStore {
    fn record(&self, event: KnowledgeAuditEventRecord) {
        let store = self.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Err(error) = store.append_event(event).await {
                    tracing::warn!(?error, "failed to persist knowledge audit event");
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connect_sqlite_and_install_schema;
    use serde_json::json;

    #[tokio::test]
    async fn append_audit_event_persists_row() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001);
        store
            .append_event(KnowledgeAuditEventRecord {
                id: None,
                uuid: None,
                event_type: "knowledge.space.member_granted".to_string(),
                actor_type: "user".to_string(),
                actor_id: "42".to_string(),
                resource_type: "space".to_string(),
                resource_id: Some(7),
                result: "success".to_string(),
                request_id: Some("req-1".to_string()),
                trace_id: None,
                payload: Some(json!({"role": "writer"})),
                created_at: None,
            })
            .await
            .expect("append");

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM kb_audit_event WHERE tenant_id = 100001")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn list_events_by_actor_returns_matching_rows() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001);
        for actor_id in ["42", "42", "99"] {
            store
                .append_event(KnowledgeAuditEventRecord {
                    id: None,
                    uuid: None,
                    event_type: "knowledge.space.member_granted".to_string(),
                    actor_type: "user".to_string(),
                    actor_id: actor_id.to_string(),
                    resource_type: "space".to_string(),
                    resource_id: Some(7),
                    result: "success".to_string(),
                    request_id: None,
                    trace_id: None,
                    payload: None,
                    created_at: None,
                })
                .await
                .expect("append");
        }

        let events = store
            .list_events_by_actor("42", 100)
            .await
            .expect("list");
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| event.actor_id == "42"));
    }

    #[tokio::test]
    async fn anonymize_actor_redacts_matching_rows() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001);
        store
            .append_event(KnowledgeAuditEventRecord {
                id: None,
                uuid: None,
                event_type: "knowledge.space.member_granted".to_string(),
                actor_type: "user".to_string(),
                actor_id: "42".to_string(),
                resource_type: "space".to_string(),
                resource_id: Some(7),
                result: "success".to_string(),
                request_id: None,
                trace_id: None,
                payload: None,
                created_at: None,
            })
            .await
            .expect("append");

        let anonymized = store.anonymize_actor("42").await.expect("anonymize");
        assert_eq!(anonymized, 1);

        let row: (String, String) = sqlx::query_as(
            "SELECT actor_id, actor_type FROM kb_audit_event WHERE tenant_id = 100001 LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("row");
        assert_eq!(row.0, "gdpr-redacted");
        assert_eq!(row.1, "system");
    }
}
