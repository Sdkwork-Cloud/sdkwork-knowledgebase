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
pub struct PostgresKnowledgeAuditEventStore {
    pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    timestamp_dialect: SqlTimestampDialect,
}

impl PostgresKnowledgeAuditEventStore {
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
impl KnowledgeAuditEventStore for PostgresKnowledgeAuditEventStore {
    async fn record(
        &self,
        event: KnowledgeAuditEventRecord,
    ) -> Result<(), KnowledgeAuditEventStoreError> {
        self.append_event(event).await
    }
}
