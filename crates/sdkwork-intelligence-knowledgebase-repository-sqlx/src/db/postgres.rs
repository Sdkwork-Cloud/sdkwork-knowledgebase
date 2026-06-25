//! PostgreSQL repository runtime for SDKWork Knowledgebase.

use sqlx::{AnyPool, PgPool};
use std::fmt;

use sdkwork_knowledgebase_database_host::bootstrap_knowledgebase_database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostgresRepositoryError {
    Sqlx(String),
}

impl fmt::Display for PostgresRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlx(detail) => write!(
                formatter,
                "postgresql knowledgebase repository error: {detail}"
            ),
        }
    }
}

impl std::error::Error for PostgresRepositoryError {}

pub fn is_postgres_database_url(database_url: &str) -> bool {
    let normalized = database_url.trim().to_ascii_lowercase();
    normalized.starts_with("postgres://") || normalized.starts_with("postgresql://")
}

pub async fn connect_postgres_pool(database_url: &str) -> Result<PgPool, PostgresRepositoryError> {
    crate::db::bootstrap::connect_postgres_pool_via_framework(database_url)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))
}

#[deprecated(
    since = "0.2.0",
    note = "Use sdkwork_knowledgebase_database_host::bootstrap_knowledgebase_database via connect_postgres_via_framework_lifecycle instead"
)]
pub async fn install_postgres_schema(_pool: &AnyPool) -> Result<(), PostgresRepositoryError> {
    Err(PostgresRepositoryError::Sqlx(
        "install_postgres_schema is deprecated; use application-root database/ lifecycle bootstrap"
            .to_string(),
    ))
}

pub async fn connect_postgres_via_framework_lifecycle(
    database_url: &str,
) -> Result<AnyPool, PostgresRepositoryError> {
    let pool = crate::db::bootstrap::connect_knowledgebase_pool_from_url(database_url)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))?;
    bootstrap_knowledgebase_database(pool)
        .await
        .map_err(PostgresRepositoryError::Sqlx)?;
    crate::db::bootstrap::connect_knowledgebase_any_pool_from_url(database_url)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))
}

pub async fn connect_postgres_and_install_schema(
    database_url: &str,
) -> Result<AnyPool, PostgresRepositoryError> {
    connect_postgres_via_framework_lifecycle(database_url).await
}

pub async fn postgres_health_check(pool: &PgPool) -> Result<(), PostgresRepositoryError> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_postgres_database_urls() {
        assert!(is_postgres_database_url("postgres://localhost/kb"));
        assert!(is_postgres_database_url("postgresql://localhost/kb"));
        assert!(!is_postgres_database_url("sqlite://data/kb.db"));
    }
}
