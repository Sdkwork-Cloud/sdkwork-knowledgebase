use serde_json::Value;
use sqlx::AnyPool;
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use sdkwork_utils_rust::is_blank;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct KnowledgeAuditEventRecord {
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: String,
    pub resource_type: String,
    pub resource_id: Option<u64>,
    pub result: String,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub payload: Option<Value>,
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
            .map(|value| value.to_string())
            .unwrap_or_default();
        let now = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| KnowledgeAuditEventStoreError::InvalidRequest(error.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO kb_audit_event (
                id, uuid, tenant_id, event_type, actor_type, actor_id,
                resource_type, resource_id, result, request_id, trace_id,
                payload, created_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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
                event_type: "knowledge.space.member_granted".to_string(),
                actor_type: "user".to_string(),
                actor_id: "42".to_string(),
                resource_type: "space".to_string(),
                resource_id: Some(7),
                result: "success".to_string(),
                request_id: Some("req-1".to_string()),
                trace_id: None,
                payload: Some(json!({"role": "writer"})),
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
}
