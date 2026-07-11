//! Cross-engine SQL timestamp bindings for SQLite and PostgreSQL pools.

use sdkwork_database_config::DatabaseEngine;
use std::fmt::Display;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SqlTimestampDialect {
    #[default]
    Sqlite,
    Postgres,
}

impl SqlTimestampDialect {
    pub fn from_database_engine(engine: DatabaseEngine) -> Self {
        match engine {
            DatabaseEngine::Postgres => Self::Postgres,
            DatabaseEngine::Sqlite => Self::Sqlite,
        }
    }

    pub fn sql_timestamp_expr(self, placeholder: &str) -> String {
        match self {
            Self::Postgres => format!("CAST({placeholder} AS TIMESTAMP)"),
            Self::Sqlite => placeholder.to_string(),
        }
    }

    pub fn sql_json_expr(self, placeholder: &str) -> String {
        match self {
            Self::Postgres => format!("CAST({placeholder} AS JSONB)"),
            Self::Sqlite => placeholder.to_string(),
        }
    }
}

pub fn utc_sql_timestamp_text() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

pub fn push_sql_timestamp_bind<'args, Sep>(
    row: &mut sqlx::query_builder::Separated<'_, 'args, sqlx::Any, Sep>,
    dialect: SqlTimestampDialect,
    value: &'args str,
) where
    Sep: Display,
{
    match dialect {
        SqlTimestampDialect::Postgres => {
            row.push("CAST(");
            row.push_bind_unseparated(value);
            row.push_unseparated(" AS TIMESTAMP)");
        }
        SqlTimestampDialect::Sqlite => {
            row.push_bind(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SqlTimestampDialect;

    #[test]
    fn postgres_dialect_casts_text_bindings_to_database_types() {
        assert_eq!(
            SqlTimestampDialect::Postgres.sql_timestamp_expr("$1"),
            "CAST($1 AS TIMESTAMP)"
        );
        assert_eq!(
            SqlTimestampDialect::Postgres.sql_json_expr("$2"),
            "CAST($2 AS JSONB)"
        );
    }

    #[test]
    fn sqlite_dialect_preserves_text_bindings() {
        assert_eq!(SqlTimestampDialect::Sqlite.sql_timestamp_expr("$1"), "$1");
        assert_eq!(SqlTimestampDialect::Sqlite.sql_json_expr("$2"), "$2");
    }
}
