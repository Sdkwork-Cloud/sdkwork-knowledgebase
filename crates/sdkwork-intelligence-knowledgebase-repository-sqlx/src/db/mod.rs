//! Database schema, row mapping, and SQL query modules.
//!
//! This module aggregates database access modules from the parent flat layout.
//! New database modules should be placed directly under this directory.

pub mod postgres;
pub mod sqlite;

pub use postgres::{phase1_postgres_unavailable, PostgresRepositoryError};
pub use sqlite::{
    connect_sqlite_and_install_schema, connect_sqlite_pool, install_sqlite_core_schema,
    install_sqlite_schema, sqlite_health_check,
};
