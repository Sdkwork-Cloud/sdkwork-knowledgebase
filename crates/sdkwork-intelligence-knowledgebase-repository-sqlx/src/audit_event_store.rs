use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use serde_json::Value;
use sqlx::{AnyPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use sdkwork_utils_rust::is_blank;

use crate::db::sql_timestamp::{utc_sql_timestamp_text, SqlTimestampDialect};
use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const INITIAL_VERSION: i64 = 0;
const MAX_AUDIT_EVENT_TYPE_CHARS: usize = 128;
const MAX_AUDIT_ACTOR_TYPE_CHARS: usize = 64;
const MAX_AUDIT_ACTOR_ID_CHARS: usize = 128;
const MAX_AUDIT_RESOURCE_TYPE_CHARS: usize = 64;
const MAX_AUDIT_RESULT_CHARS: usize = 64;
const MAX_AUDIT_REQUEST_ID_CHARS: usize = 64;
const MAX_AUDIT_TRACE_ID_CHARS: usize = 128;
const MAX_AUDIT_UUID_CHARS: usize = 64;
const MAX_AUDIT_CREATED_AT_CHARS: usize = 64;
const MAX_AUDIT_PAYLOAD_BYTES: usize = 64 * 1024;
const MAX_AUDIT_EXPORT_EVENTS: u32 = 5_000;

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
    #[error("audit export exceeds the maximum of {max_events} events")]
    ExportLimitExceeded { max_events: u32 },
    #[error("audit event data integrity error: {0}")]
    DataIntegrity(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("id generation error: {0}")]
    IdGeneration(String),
}

#[async_trait]
pub trait KnowledgeAuditEventStore: Send + Sync {
    async fn record(
        &self,
        event: KnowledgeAuditEventRecord,
    ) -> Result<(), KnowledgeAuditEventStoreError>;
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeAuditEventStore {
    pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    timestamp_dialect: SqlTimestampDialect,
}

impl SqliteKnowledgeAuditEventStore {
    pub fn new(pool: AnyPool, tenant_id: u64, organization_id: u64) -> Self {
        Self::with_id_generator(
            pool,
            tenant_id,
            organization_id,
            default_knowledge_id_generator(),
        )
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        organization_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            organization_id,
            id_generator,
            timestamp_dialect: SqlTimestampDialect::default(),
        }
    }

    pub fn with_database_engine(mut self, database_engine: DatabaseEngine) -> Self {
        self.timestamp_dialect = SqlTimestampDialect::from_database_engine(database_engine);
        self
    }

    pub async fn append_event(
        &self,
        event: KnowledgeAuditEventRecord,
    ) -> Result<(), KnowledgeAuditEventStoreError> {
        validate_required_audit_text("event_type", &event.event_type, MAX_AUDIT_EVENT_TYPE_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_required_audit_text("actor_type", &event.actor_type, MAX_AUDIT_ACTOR_TYPE_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_required_audit_text("actor_id", &event.actor_id, MAX_AUDIT_ACTOR_ID_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_required_audit_text(
            "resource_type",
            &event.resource_type,
            MAX_AUDIT_RESOURCE_TYPE_CHARS,
        )
        .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_required_audit_text("result", &event.result, MAX_AUDIT_RESULT_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_optional_audit_text(
            "request_id",
            event.request_id.as_deref(),
            MAX_AUDIT_REQUEST_ID_CHARS,
        )
        .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        validate_optional_audit_text(
            "trace_id",
            event.trace_id.as_deref(),
            MAX_AUDIT_TRACE_ID_CHARS,
        )
        .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;

        let id = next_i64_id(&self.id_generator)
            .map_err(|error| KnowledgeAuditEventStoreError::IdGeneration(error.to_string()))?;
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let organization_id = i64::try_from(self.organization_id).map_err(|_| {
            KnowledgeAuditEventStoreError::InvalidRequest("organization_id".to_string())
        })?;
        let resource_id = event
            .resource_id
            .map(i64::try_from)
            .transpose()
            .map_err(|_| {
                KnowledgeAuditEventStoreError::InvalidRequest("resource_id".to_string())
            })?;
        let payload = event.payload.as_ref().map(ToString::to_string);
        if payload
            .as_ref()
            .is_some_and(|value| value.len() > MAX_AUDIT_PAYLOAD_BYTES)
        {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(format!(
                "payload exceeds {MAX_AUDIT_PAYLOAD_BYTES} bytes"
            )));
        }
        let now =
            utc_sql_timestamp_text().map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;

        let payload_expr = self.timestamp_dialect.sql_json_expr("$13");
        let created_at_expr = self.timestamp_dialect.sql_timestamp_expr("$14");
        let query = format!(
            r#"
            INSERT INTO kb_audit_event (
                id, uuid, tenant_id, organization_id, event_type, actor_type, actor_id,
                resource_type, resource_id, result, request_id, trace_id,
                payload, created_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, {payload_expr}, {created_at_expr}, $15)
            "#,
        );
        sqlx::query(sqlx::AssertSqlSafe(query.as_str()))
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(organization_id)
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
        validate_required_audit_text("actor_id", actor_id, MAX_AUDIT_ACTOR_ID_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        if !(1..=MAX_AUDIT_EXPORT_EVENTS).contains(&limit) {
            return Err(KnowledgeAuditEventStoreError::InvalidRequest(format!(
                "limit must be between 1 and {MAX_AUDIT_EXPORT_EVENTS}"
            )));
        }
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let organization_id = i64::try_from(self.organization_id).map_err(|_| {
            KnowledgeAuditEventStoreError::InvalidRequest("organization_id".to_string())
        })?;
        let requested_limit = usize::try_from(limit).map_err(|_| {
            KnowledgeAuditEventStoreError::InvalidRequest("limit is unsupported".to_string())
        })?;
        let query_limit = i64::from(limit) + 1;
        let created_at_expr = self.timestamp_dialect.sql_timestamp_text_expr("created_at");
        let query = format!(
            r#"
            SELECT id, uuid, event_type, actor_type, actor_id, resource_type, resource_id,
                   result, request_id, trace_id, {created_at_expr} AS created_at
            FROM kb_audit_event
            WHERE tenant_id = $1 AND organization_id = $2 AND actor_id = $3
            ORDER BY created_at DESC, id DESC
            LIMIT $4
            "#,
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(query.as_str()))
            .bind(tenant_id)
            .bind(organization_id)
            .bind(actor_id)
            .bind(query_limit)
            .fetch_all(&self.pool)
            .await?;

        if rows.len() > requested_limit {
            return Err(KnowledgeAuditEventStoreError::ExportLimitExceeded { max_events: limit });
        }

        rows.into_iter()
            .map(|row| {
                let id = row
                    .try_get::<i64, _>("id")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                if id <= 0 {
                    return Err(KnowledgeAuditEventStoreError::DataIntegrity(
                        "id must be positive".to_string(),
                    ));
                }
                let uuid = row
                    .try_get::<String, _>("uuid")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text("uuid", &uuid, MAX_AUDIT_UUID_CHARS)
                    .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let created_at = row
                    .try_get::<String, _>("created_at")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text("created_at", &created_at, MAX_AUDIT_CREATED_AT_CHARS)
                    .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let event_type = row
                    .try_get::<String, _>("event_type")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text("event_type", &event_type, MAX_AUDIT_EVENT_TYPE_CHARS)
                    .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let actor_type = row
                    .try_get::<String, _>("actor_type")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text("actor_type", &actor_type, MAX_AUDIT_ACTOR_TYPE_CHARS)
                    .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let decoded_actor_id = row
                    .try_get::<String, _>("actor_id")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text(
                    "actor_id",
                    &decoded_actor_id,
                    MAX_AUDIT_ACTOR_ID_CHARS,
                )
                .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let resource_type = row
                    .try_get::<String, _>("resource_type")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text(
                    "resource_type",
                    &resource_type,
                    MAX_AUDIT_RESOURCE_TYPE_CHARS,
                )
                .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let result = row
                    .try_get::<String, _>("result")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_required_audit_text("result", &result, MAX_AUDIT_RESULT_CHARS)
                    .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let request_id = row
                    .try_get::<Option<String>, _>("request_id")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_optional_audit_text(
                    "request_id",
                    request_id.as_deref(),
                    MAX_AUDIT_REQUEST_ID_CHARS,
                )
                .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let trace_id = row
                    .try_get::<Option<String>, _>("trace_id")
                    .map_err(KnowledgeAuditEventStoreError::Database)?;
                validate_optional_audit_text(
                    "trace_id",
                    trace_id.as_deref(),
                    MAX_AUDIT_TRACE_ID_CHARS,
                )
                .map_err(KnowledgeAuditEventStoreError::DataIntegrity)?;
                let resource_id = row
                    .try_get::<Option<i64>, _>("resource_id")
                    .map_err(KnowledgeAuditEventStoreError::Database)?
                    .map(u64::try_from)
                    .transpose()
                    .map_err(|_| {
                        KnowledgeAuditEventStoreError::DataIntegrity(
                            "resource_id must not be negative".to_string(),
                        )
                    })?;
                Ok(KnowledgeAuditEventRecord {
                    id: Some(id),
                    uuid: Some(uuid),
                    event_type,
                    actor_type,
                    actor_id: decoded_actor_id,
                    resource_type,
                    resource_id,
                    result,
                    request_id,
                    trace_id,
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
        validate_required_audit_text("actor_id", actor_id, MAX_AUDIT_ACTOR_ID_CHARS)
            .map_err(KnowledgeAuditEventStoreError::InvalidRequest)?;
        let tenant_id = i64::try_from(self.tenant_id)
            .map_err(|_| KnowledgeAuditEventStoreError::InvalidRequest("tenant_id".to_string()))?;
        let organization_id = i64::try_from(self.organization_id).map_err(|_| {
            KnowledgeAuditEventStoreError::InvalidRequest("organization_id".to_string())
        })?;
        let result = sqlx::query(
            r#"
            UPDATE kb_audit_event
            SET actor_id = 'gdpr-redacted', actor_type = 'system'
            WHERE tenant_id = $1 AND organization_id = $2 AND actor_id = $3
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(actor_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

fn validate_required_audit_text(field: &str, value: &str, max_chars: usize) -> Result<(), String> {
    if is_blank(Some(value)) {
        return Err(format!("{field} is required"));
    }
    if value.chars().nth(max_chars).is_some() {
        return Err(format!("{field} exceeds {max_chars} characters"));
    }
    Ok(())
}

fn validate_optional_audit_text(
    field: &str,
    value: Option<&str>,
    max_chars: usize,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.chars().nth(max_chars).is_some() {
        return Err(format!("{field} exceeds {max_chars} characters"));
    }
    Ok(())
}

#[async_trait]
impl KnowledgeAuditEventStore for SqliteKnowledgeAuditEventStore {
    async fn record(
        &self,
        event: KnowledgeAuditEventRecord,
    ) -> Result<(), KnowledgeAuditEventStoreError> {
        self.append_event(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connect_sqlite_and_install_schema;
    use serde_json::json;
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};

    #[tokio::test]
    async fn append_audit_event_persists_row() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
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

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM kb_audit_event WHERE tenant_id = 100001 AND organization_id = 7",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn append_audit_event_enforces_cross_engine_text_and_payload_bounds() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool, 100_001, 7);

        let mut oversized_actor = test_audit_event("42");
        oversized_actor.actor_id = "x".repeat(MAX_AUDIT_ACTOR_ID_CHARS + 1);
        assert!(matches!(
            store.append_event(oversized_actor).await,
            Err(KnowledgeAuditEventStoreError::InvalidRequest(_))
        ));

        let mut oversized_payload = test_audit_event("42");
        oversized_payload.payload = Some(json!({
            "value": "x".repeat(MAX_AUDIT_PAYLOAD_BYTES),
        }));
        assert!(matches!(
            store.append_event(oversized_payload).await,
            Err(KnowledgeAuditEventStoreError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn actor_operations_reject_invalid_bounds_instead_of_clamping() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool, 100_001, 7);

        assert!(matches!(
            store.list_events_by_actor("42", 0).await,
            Err(KnowledgeAuditEventStoreError::InvalidRequest(_))
        ));
        assert!(matches!(
            store
                .list_events_by_actor("42", MAX_AUDIT_EXPORT_EVENTS + 1)
                .await,
            Err(KnowledgeAuditEventStoreError::InvalidRequest(_))
        ));
        assert!(matches!(
            store
                .anonymize_actor(&"x".repeat(MAX_AUDIT_ACTOR_ID_CHARS + 1))
                .await,
            Err(KnowledgeAuditEventStoreError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn list_events_by_actor_returns_matching_rows() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
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

        let events = store.list_events_by_actor("42", 100).await.expect("list");
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| event.actor_id == "42"));
        assert!(events.iter().all(|event| event.uuid.is_some()));
        assert!(events.iter().all(|event| event.request_id.is_none()));
        assert!(events.iter().all(|event| event.trace_id.is_none()));
        assert!(events.iter().all(|event| event.created_at.is_some()));
        assert!(events.iter().all(|event| {
            OffsetDateTime::parse(event.created_at.as_deref().expect("created_at"), &Rfc3339)
                .is_ok()
        }));
    }

    #[tokio::test]
    async fn list_events_by_actor_fails_when_export_would_be_truncated() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool, 100_001, 7);
        for _ in 0..2 {
            store
                .append_event(test_audit_event("42"))
                .await
                .expect("append");
        }

        let error = store
            .list_events_by_actor("42", 1)
            .await
            .expect_err("bounded export must not silently truncate");

        assert!(matches!(
            error,
            KnowledgeAuditEventStoreError::ExportLimitExceeded { max_events: 1 }
        ));
    }

    #[tokio::test]
    async fn list_events_rejects_negative_persisted_resource_id() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
        store
            .append_event(test_audit_event("42"))
            .await
            .expect("append");
        sqlx::query(
            "UPDATE kb_audit_event SET resource_id = -1 WHERE tenant_id = 100001 AND organization_id = 7",
        )
        .execute(&pool)
        .await
        .expect("corrupt resource id");

        let error = store
            .list_events_by_actor("42", 100)
            .await
            .expect_err("negative resource id must fail closed");

        assert!(matches!(
            error,
            KnowledgeAuditEventStoreError::DataIntegrity(_)
        ));
    }

    #[tokio::test]
    async fn list_events_propagates_optional_text_decode_failure() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
        store
            .append_event(test_audit_event("42"))
            .await
            .expect("append");
        sqlx::query(
            "UPDATE kb_audit_event SET request_id = X'80' WHERE tenant_id = 100001 AND organization_id = 7",
        )
        .execute(&pool)
        .await
        .expect("corrupt request id");

        let error = store
            .list_events_by_actor("42", 100)
            .await
            .expect_err("invalid request id encoding must fail closed");

        assert!(matches!(error, KnowledgeAuditEventStoreError::Database(_)));
    }

    #[tokio::test]
    async fn anonymize_actor_redacts_matching_rows() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
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
            "SELECT actor_id, actor_type FROM kb_audit_event WHERE tenant_id = 100001 AND organization_id = 7 LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("row");
        assert_eq!(row.0, "gdpr-redacted");
        assert_eq!(row.1, "system");
    }

    #[tokio::test]
    async fn actor_operations_are_isolated_between_organizations() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let organization_seven = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
        let organization_eight = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 8);
        organization_seven
            .append_event(KnowledgeAuditEventRecord {
                id: None,
                uuid: None,
                event_type: "knowledge.document.read".to_string(),
                actor_type: "user".to_string(),
                actor_id: "42".to_string(),
                resource_type: "document".to_string(),
                resource_id: Some(9),
                result: "success".to_string(),
                request_id: None,
                trace_id: None,
                payload: None,
                created_at: None,
            })
            .await
            .expect("append");

        assert!(organization_eight
            .list_events_by_actor("42", 100)
            .await
            .expect("list")
            .is_empty());
        assert_eq!(
            organization_eight
                .anonymize_actor("42")
                .await
                .expect("anonymize"),
            0
        );
    }

    #[tokio::test]
    async fn record_returns_database_failure_instead_of_detaching_write() {
        let pool = connect_sqlite_and_install_schema("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001, 7);
        pool.close().await;

        let error = store
            .record(KnowledgeAuditEventRecord {
                id: None,
                uuid: None,
                event_type: "knowledge.space.member_granted".to_string(),
                actor_type: "user".to_string(),
                actor_id: "42".to_string(),
                resource_type: "space".to_string(),
                resource_id: Some(7),
                result: "success".to_string(),
                request_id: None,
                trace_id: Some("trace-1".to_string()),
                payload: None,
                created_at: None,
            })
            .await
            .expect_err("closed pool must fail synchronously");

        assert!(matches!(error, KnowledgeAuditEventStoreError::Database(_)));
    }

    fn test_audit_event(actor_id: &str) -> KnowledgeAuditEventRecord {
        KnowledgeAuditEventRecord {
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
        }
    }
}
