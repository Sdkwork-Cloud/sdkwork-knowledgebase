mod backfill;
mod checkpoint;
mod inbox;
mod projection;
mod publication;
mod rendition;

use std::str::FromStr;
use std::sync::Arc;

use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    WikiPersistenceError, WikiPersistenceScope,
};
use sdkwork_utils_rust::uuid;
use sqlx::{any::AnyRow, AnyPool, Row};
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

use crate::db::sql_timestamp::SqlTimestampDialect;
use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const MAX_CLAIM_BATCH_SIZE: u32 = 100;
const MAX_LEASE_SECONDS: u64 = 3_600;
const MAX_RETRY_DELAY_SECONDS: u64 = 86_400;

#[derive(Clone)]
pub struct SqlxWikiPersistenceStore {
    pool: AnyPool,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    dialect: SqlTimestampDialect,
}

impl SqlxWikiPersistenceStore {
    pub fn new(pool: AnyPool) -> Self {
        Self::with_id_generator(pool, default_knowledge_id_generator())
    }

    pub fn with_id_generator(pool: AnyPool, id_generator: Arc<dyn KnowledgeIdGenerator>) -> Self {
        Self {
            pool,
            id_generator,
            dialect: SqlTimestampDialect::default(),
        }
    }

    pub fn with_database_engine(mut self, database_engine: DatabaseEngine) -> Self {
        self.dialect = SqlTimestampDialect::from_database_engine(database_engine);
        self
    }

    fn next_id(&self) -> Result<i64, WikiPersistenceError> {
        next_i64_id(&self.id_generator)
            .map_err(|error| WikiPersistenceError::Internal(error.to_string()))
    }
}

fn validate_scope(scope: WikiPersistenceScope) -> Result<(), WikiPersistenceError> {
    if scope.tenant_id == 0 {
        return Err(WikiPersistenceError::InvalidRequest(
            "tenant_id must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

fn require_id(field: &str, value: u64) -> Result<i64, WikiPersistenceError> {
    if value == 0 {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "{field} must be greater than zero"
        )));
    }
    to_i64(field, value)
}

fn require_text<'a>(
    field: &str,
    value: &'a str,
    max_bytes: usize,
) -> Result<&'a str, WikiPersistenceError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    if value.len() > max_bytes {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "{field} exceeds {max_bytes} bytes"
        )));
    }
    Ok(value)
}

fn require_sha256(field: &str, value: &str) -> Result<(), WikiPersistenceError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "{field} must use the sha256:<lowercase-hex> format"
        )));
    };
    if digest.len() != 64
        || !digest
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || matches!(*byte, b'a'..=b'f'))
    {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "{field} must contain 64 lowercase hexadecimal characters"
        )));
    }
    Ok(())
}

fn claim_limit(limit: u32) -> Result<i64, WikiPersistenceError> {
    if limit == 0 || limit > MAX_CLAIM_BATCH_SIZE {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "limit must be between 1 and {MAX_CLAIM_BATCH_SIZE}"
        )));
    }
    Ok(i64::from(limit))
}

fn lease_times(lease_seconds: u64) -> Result<(String, String), WikiPersistenceError> {
    if lease_seconds == 0 || lease_seconds > MAX_LEASE_SECONDS {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "lease_seconds must be between 1 and {MAX_LEASE_SECONDS}"
        )));
    }
    let now = OffsetDateTime::now_utc();
    let seconds = i64::try_from(lease_seconds).map_err(|_| {
        WikiPersistenceError::InvalidRequest("lease_seconds exceeds int64".to_string())
    })?;
    Ok((
        format_time(now)?,
        format_time(now + Duration::seconds(seconds))?,
    ))
}

fn retry_time(retry_delay_seconds: u64) -> Result<(String, String), WikiPersistenceError> {
    if retry_delay_seconds == 0 || retry_delay_seconds > MAX_RETRY_DELAY_SECONDS {
        return Err(WikiPersistenceError::InvalidRequest(format!(
            "retry_delay_seconds must be between 1 and {MAX_RETRY_DELAY_SECONDS}"
        )));
    }
    let now = OffsetDateTime::now_utc();
    let seconds = i64::try_from(retry_delay_seconds).map_err(|_| {
        WikiPersistenceError::InvalidRequest("retry_delay_seconds exceeds int64".to_string())
    })?;
    Ok((
        format_time(now)?,
        format_time(now + Duration::seconds(seconds))?,
    ))
}

fn now() -> Result<String, WikiPersistenceError> {
    format_time(OffsetDateTime::now_utc())
}

fn format_time(value: OffsetDateTime) -> Result<String, WikiPersistenceError> {
    value
        .format(&Rfc3339)
        .map_err(|error| WikiPersistenceError::Internal(error.to_string()))
}

fn new_lease_token() -> String {
    uuid()
}

fn to_i64(field: &str, value: u64) -> Result<i64, WikiPersistenceError> {
    i64::try_from(value)
        .map_err(|_| WikiPersistenceError::InvalidRequest(format!("{field} exceeds signed int64")))
}

fn from_i64(field: &str, value: i64) -> Result<u64, WikiPersistenceError> {
    u64::try_from(value)
        .map_err(|_| WikiPersistenceError::Internal(format!("database returned negative {field}")))
}

fn from_i32(field: &str, value: i32) -> Result<u32, WikiPersistenceError> {
    u32::try_from(value)
        .map_err(|_| WikiPersistenceError::Internal(format!("database returned negative {field}")))
}

fn optional_u64(row: &AnyRow, field: &str) -> Result<Option<u64>, WikiPersistenceError> {
    row.try_get::<Option<i64>, _>(field)
        .map_err(row_error)?
        .map(|value| from_i64(field, value))
        .transpose()
}

fn parse_enum<T>(field: &str, value: String) -> Result<T, WikiPersistenceError>
where
    T: FromStr<Err = ()>,
{
    value.parse().map_err(|_| {
        WikiPersistenceError::Internal(format!(
            "database returned unsupported {field} value {value}"
        ))
    })
}

fn row_error(error: sqlx::Error) -> WikiPersistenceError {
    WikiPersistenceError::Internal(format!("failed to decode Wiki persistence row: {error}"))
}

fn sql_error(error: sqlx::Error) -> WikiPersistenceError {
    if error
        .as_database_error()
        .is_some_and(|database_error| database_error.is_unique_violation())
    {
        return WikiPersistenceError::Conflict(
            "a Wiki persistence uniqueness constraint was violated".to_string(),
        );
    }
    WikiPersistenceError::Internal(error.to_string())
}
