//! Database schema, row mapping, and SQL query modules.
//!
//! This module aggregates database access modules from the parent flat layout.
//! New database modules should be placed directly under this directory.

pub mod postgres;
pub mod sqlite;

pub use postgres::{
    connect_postgres_and_install_schema, connect_postgres_pool, install_postgres_schema,
    is_postgres_database_url, phase1_postgres_unavailable, postgres_health_check,
    PostgresRepositoryError,
};
pub use sqlite::{
    connect_sqlite_and_install_schema, connect_sqlite_pool, install_sqlite_core_schema,
    install_sqlite_schema, sqlite_health_check,
};
