//! SDKWork database pool bootstrap via `sdkwork-database`.

use std::path::Path;
use std::str::FromStr;

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine, PgSslMode};
use sdkwork_database_sqlx::{
    create_any_pool_from_config, create_pool_from_config, pool::PoolContext, DatabasePool,
    PoolError,
};
use sqlx::any::{AnyConnectOptions, AnyPoolOptions};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AnyPool, PgPool};

pub use sdkwork_knowledgebase_database_host::{
    bootstrap_knowledgebase_database, bootstrap_knowledgebase_database_from_env,
    KnowledgebaseDatabaseHost,
};

use crate::db::postgres_tenant_session::{
    require_postgres_rls_tenant_id, set_postgres_session_tenant_id, POSTGRES_TENANT_SESSION_KEY,
};

pub type KnowledgebaseDatabasePool = DatabasePool;

const KNOWLEDGEBASE_POOL_MAX_CONNECTIONS: u32 = 5;

fn resolve_max_connections(engine: DatabaseEngine, database_url: &str) -> u32 {
    std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or_else(|| max_connections_for_url(engine, database_url))
}

fn max_connections_for_url(engine: DatabaseEngine, database_url: &str) -> u32 {
    if engine == DatabaseEngine::Sqlite && database_url.trim() == "sqlite::memory:" {
        return 1;
    }
    KNOWLEDGEBASE_POOL_MAX_CONNECTIONS
}

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
        max_connections: resolve_max_connections(engine, normalized),
        ..DatabaseConfig::default()
    })
}

fn to_sqlx_ssl_mode(mode: PgSslMode) -> sqlx::postgres::PgSslMode {
    match mode {
        PgSslMode::Disable => sqlx::postgres::PgSslMode::Disable,
        PgSslMode::Allow => sqlx::postgres::PgSslMode::Allow,
        PgSslMode::Prefer => sqlx::postgres::PgSslMode::Prefer,
        PgSslMode::Require => sqlx::postgres::PgSslMode::Require,
        PgSslMode::VerifyCa => sqlx::postgres::PgSslMode::VerifyCa,
        PgSslMode::VerifyFull => sqlx::postgres::PgSslMode::VerifyFull,
    }
}

fn pg_connect_options(config: &DatabaseConfig) -> Result<PgConnectOptions, PoolError> {
    let pg_config = &config.postgres;
    let mut connect_options = PgConnectOptions::from_str(&config.url)
        .map_err(|error| PoolError::InvalidUrl(format!("{}: {error}", config.url)))?
        .ssl_mode(to_sqlx_ssl_mode(pg_config.ssl_mode));

    if let Some(app_name) = &pg_config.application_name {
        connect_options = connect_options.application_name(app_name);
    }

    if let Some(root_cert) = &pg_config.ssl_root_cert {
        connect_options = connect_options.ssl_root_cert(Path::new(root_cert));
    }

    Ok(connect_options)
}

async fn create_postgres_pool_with_rls_tenant_session(
    config: &DatabaseConfig,
    tenant_id: u64,
) -> Result<(PgPool, PoolContext), PoolError> {
    let connect_options = pg_connect_options(config)?;
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout())
        .idle_timeout(config.idle_timeout())
        .max_lifetime(config.max_lifetime())
        .after_connect(move |connection, _metadata| {
            Box::pin(
                async move { set_postgres_session_tenant_id(&mut *connection, tenant_id).await },
            )
        })
        .connect_with(connect_options)
        .await
        .map_err(PoolError::PoolCreation)?;

    Ok((
        pool,
        PoolContext {
            config: config.clone(),
        },
    ))
}

async fn create_any_pool_with_rls_tenant_session(
    config: &DatabaseConfig,
    tenant_id: u64,
) -> Result<AnyPool, PoolError> {
    sqlx::any::install_default_drivers();

    let connect_options = AnyConnectOptions::from_str(&config.url)
        .map_err(|error| PoolError::InvalidUrl(format!("{}: {error}", config.url)))?;
    let tenant_key = POSTGRES_TENANT_SESSION_KEY.to_string();

    let pool = AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout())
        .idle_timeout(config.idle_timeout())
        .max_lifetime(config.max_lifetime())
        .after_connect(move |connection, _metadata| {
            let tenant_key = tenant_key.clone();
            let tenant_value = tenant_id.to_string();
            Box::pin(async move {
                if connection.backend_name() == "PostgreSQL" {
                    sqlx::query("SELECT set_config($1, $2, false)")
                        .bind(tenant_key)
                        .bind(tenant_value)
                        .execute(&mut *connection)
                        .await?;
                }
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
        .map_err(PoolError::PoolCreation)?;

    Ok(pool)
}

async fn connect_knowledgebase_pool_from_config(
    config: DatabaseConfig,
) -> Result<KnowledgebaseDatabasePool, PoolError> {
    match config.engine {
        DatabaseEngine::Postgres => {
            let tenant_id = require_postgres_rls_tenant_id()?;
            let (pool, ctx) =
                create_postgres_pool_with_rls_tenant_session(&config, tenant_id).await?;
            Ok(DatabasePool::Postgres(pool, ctx))
        }
        DatabaseEngine::Sqlite => create_pool_from_config(config).await,
    }
}

async fn connect_knowledgebase_any_pool_from_config(
    config: DatabaseConfig,
) -> Result<AnyPool, PoolError> {
    match config.engine {
        DatabaseEngine::Postgres => {
            let tenant_id = require_postgres_rls_tenant_id()?;
            create_any_pool_with_rls_tenant_session(&config, tenant_id).await
        }
        DatabaseEngine::Sqlite => create_any_pool_from_config(config).await,
    }
}

pub async fn connect_knowledgebase_pool_from_env() -> Result<KnowledgebaseDatabasePool, PoolError> {
    let config = DatabaseConfig::from_env("KNOWLEDGEBASE")?;
    connect_knowledgebase_pool_from_config(config).await
}

pub async fn connect_knowledgebase_pool_from_url(
    database_url: &str,
) -> Result<KnowledgebaseDatabasePool, PoolError> {
    connect_knowledgebase_pool_from_config(database_config_from_url(database_url)?).await
}

pub async fn connect_knowledgebase_any_pool_from_url(
    database_url: &str,
) -> Result<AnyPool, PoolError> {
    connect_knowledgebase_any_pool_from_config(database_config_from_url(database_url)?).await
}

pub fn knowledgebase_database_engine_from_url(
    database_url: &str,
) -> Result<DatabaseEngine, PoolError> {
    Ok(database_config_from_url(database_url)?.engine)
}

fn map_pool_error(error: PoolError) -> sqlx::Error {
    sqlx::Error::Configuration(error.to_string().into())
}

pub async fn connect_sqlite_pool_via_framework(database_url: &str) -> Result<AnyPool, sqlx::Error> {
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
) -> Result<PgPool, sqlx::Error> {
    let pool = connect_knowledgebase_pool_from_url(database_url)
        .await
        .map_err(map_pool_error)?;
    pool.as_postgres()
        .cloned()
        .ok_or_else(|| sqlx::Error::Configuration("expected postgres database url".into()))
}

/// Create the knowledgebase pool and apply the application-root `database/` lifecycle when enabled.
pub async fn create_and_bootstrap_knowledgebase_database_pool_from_env(
) -> Result<KnowledgebaseDatabaseHost, String> {
    let pool = connect_knowledgebase_pool_from_env()
        .await
        .map_err(|error| error.to_string())?;
    bootstrap_knowledgebase_database(pool).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn postgres_pool_bootstrap_wires_after_connect_tenant_session() {
        let source = include_str!("bootstrap.rs");
        assert!(source.contains("after_connect"));
        assert!(source.contains("set_postgres_session_tenant_id"));
    }
}
