//! SDKWork database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{
    create_any_pool_from_config, create_pool_from_config, DatabasePool, PoolError,
};

pub type KnowledgebaseDatabasePool = DatabasePool;

const KNOWLEDGEBASE_POOL_MAX_CONNECTIONS: u32 = 5;

pub fn database_config_from_url(database_url: &str) -> Result<DatabaseConfig, PoolError> {
    let normalized = database_url.trim();
    let engine = DatabaseEngine::from_url(normalized).ok_or_else(|| {
        PoolError::InvalidUrl(format!(
            "unsupported knowledgebase database url: {normalized}"
        ))
    })?;
    Ok(DatabaseConfig {
        engine,
        url: normalized.to_string(),
        max_connections: max_connections_for_url(engine, normalized),
        ..DatabaseConfig::default()
    })
}

fn max_connections_for_url(engine: DatabaseEngine, database_url: &str) -> u32 {
    if engine == DatabaseEngine::Sqlite && database_url.trim() == "sqlite::memory:" {
        return 1;
    }
    KNOWLEDGEBASE_POOL_MAX_CONNECTIONS
}

pub async fn connect_knowledgebase_pool_from_env() -> Result<KnowledgebaseDatabasePool, PoolError> {
    let config = DatabaseConfig::from_env("KNOWLEDGEBASE")?;
    create_pool_from_config(config).await
}

pub async fn connect_knowledgebase_pool_from_url(
    database_url: &str,
) -> Result<KnowledgebaseDatabasePool, PoolError> {
    create_pool_from_config(database_config_from_url(database_url)?).await
}

pub async fn connect_knowledgebase_any_pool_from_url(
    database_url: &str,
) -> Result<sqlx::AnyPool, PoolError> {
    create_any_pool_from_config(database_config_from_url(database_url)?).await
}

pub fn knowledgebase_database_engine_from_url(
    database_url: &str,
) -> Result<DatabaseEngine, PoolError> {
    Ok(database_config_from_url(database_url)?.engine)
}

fn map_pool_error(error: PoolError) -> sqlx::Error {
    sqlx::Error::Configuration(error.to_string().into())
}

pub async fn connect_sqlite_pool_via_framework(
    database_url: &str,
) -> Result<sqlx::AnyPool, sqlx::Error> {
    let config = database_config_from_url(database_url).map_err(map_pool_error)?;
    if config.engine != DatabaseEngine::Sqlite {
        return Err(sqlx::Error::Configuration(
            "expected sqlite knowledgebase database url".into(),
        ));
    }
    create_any_pool_from_config(config)
        .await
        .map_err(map_pool_error)
}

pub async fn connect_postgres_pool_via_framework(
    database_url: &str,
) -> Result<sqlx::PgPool, sqlx::Error> {
    let pool = connect_knowledgebase_pool_from_url(database_url)
        .await
        .map_err(map_pool_error)?;
    pool.as_postgres()
        .cloned()
        .ok_or_else(|| sqlx::Error::Configuration("expected postgres database url".into()))
}
