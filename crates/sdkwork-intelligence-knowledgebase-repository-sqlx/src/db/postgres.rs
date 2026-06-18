//! PostgreSQL repository runtime for SDKWork Knowledgebase.
//!
//! Phase 1 ships SQLite-only repository wiring. PostgreSQL migrations exist for forward
//! compatibility, but no production pool or repository adapters are connected yet.

use std::fmt;

/// Explicit gate error when callers request PostgreSQL repository runtime in phase 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostgresRepositoryError {
    Phase1SqliteOnly,
}

impl fmt::Display for PostgresRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase1SqliteOnly => formatter.write_str(
                "postgresql knowledgebase repository runtime is not available in phase 1; use sqlite via SDKWORK_KNOWLEDGEBASE_DATABASE_URL",
            ),
        }
    }
}

impl std::error::Error for PostgresRepositoryError {}

/// Returns the phase-1 PostgreSQL gate error.
pub fn phase1_postgres_unavailable() -> PostgresRepositoryError {
    PostgresRepositoryError::Phase1SqliteOnly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase1_postgres_gate_returns_explicit_error() {
        let error = phase1_postgres_unavailable();
        assert_eq!(error, PostgresRepositoryError::Phase1SqliteOnly);
        assert!(error.to_string().contains("phase 1"));
        assert!(error.to_string().contains("sqlite"));
    }
}
