//! PostgreSQL repository runtime for SDKWork Knowledgebase.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fmt;

use crate::migrations::POSTGRES_MIGRATIONS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostgresRepositoryError {
    Phase1SqliteOnly,
    Sqlx(String),
}

impl fmt::Display for PostgresRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase1SqliteOnly => formatter.write_str(
                "postgresql knowledgebase repository runtime is not wired to HTTP handlers in phase 1; use sqlite via SDKWORK_KNOWLEDGEBASE_DATABASE_URL",
            ),
            Self::Sqlx(detail) => write!(formatter, "postgresql knowledgebase repository error: {detail}"),
        }
    }
}

impl std::error::Error for PostgresRepositoryError {}

pub fn is_postgres_database_url(database_url: &str) -> bool {
    let normalized = database_url.trim().to_ascii_lowercase();
    normalized.starts_with("postgres://") || normalized.starts_with("postgresql://")
}

pub async fn connect_postgres_pool(database_url: &str) -> Result<PgPool, PostgresRepositoryError> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))
}

pub async fn install_postgres_schema(pool: &PgPool) -> Result<(), PostgresRepositoryError> {
    for migration in POSTGRES_MIGRATIONS {
        for statement in migration.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                sqlx::query(statement)
                    .execute(pool)
                    .await
                    .map_err(|error| PostgresRepositoryError::Sqlx(error.to_string()))?;
            }
        }
    }
    Ok(())
}

pub async fn connect_postgres_and_install_schema(
    database_url: &str,
) -> Result<PgPool, PostgresRepositoryError> {
    let pool = connect_postgres_pool(database_url).await?;
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

/// Returns the phase-1 HTTP gate error when handlers are backed by SQLite-only runtime.
pub fn phase1_postgres_unavailable() -> PostgresRepositoryError {
    PostgresRepositoryError::Phase1SqliteOnly
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

    #[test]
    fn phase1_postgres_gate_returns_explicit_error() {
        let error = phase1_postgres_unavailable();
        assert_eq!(error, PostgresRepositoryError::Phase1SqliteOnly);
        assert!(error.to_string().contains("phase 1"));
    }
}
