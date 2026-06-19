//! PostgreSQL repository runtime for SDKWork Knowledgebase.

use sqlx::{AnyPool, PgPool};
use std::fmt;

use crate::migrations::POSTGRES_MIGRATIONS;

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

pub async fn install_postgres_schema(pool: &AnyPool) -> Result<(), PostgresRepositoryError> {
    for migration in POSTGRES_MIGRATIONS {
        sqlx::raw_sql(migration)
            .execute(pool)
            .await
            .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))?;
    }
    Ok(())
}

pub async fn connect_postgres_and_install_schema(
    database_url: &str,
) -> Result<AnyPool, PostgresRepositoryError> {
    let pool = crate::db::bootstrap::connect_knowledgebase_any_pool_from_url(database_url)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))?;
    install_postgres_schema(&pool).await?;
    Ok(pool)
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
