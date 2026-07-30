//! SDKWork database pool bootstrap via `sdkwork-database`.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine, PgSslMode};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sqlx::{AnyPool, PgPool};
use url::Url;

pub use sdkwork_knowledgebase_database_host::{
    bootstrap_knowledgebase_database, bootstrap_knowledgebase_database_from_env,
    KnowledgebaseDatabaseHost,
};

use crate::db::postgres_tenant_session::{
    require_postgres_rls_tenant_id, POSTGRES_TENANT_SESSION_KEY,
};

pub type KnowledgebaseDatabasePool = DatabasePool;

const KNOWLEDGEBASE_POOL_MAX_CONNECTIONS: u32 = 5;

fn resolve_max_connections(engine: DatabaseEngine, database_url: &str) -> u32 {
    std::env::var("SDKWORK_DATABASE_MAX_CONNECTIONS")
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
    let url = if engine == DatabaseEngine::Postgres {
        let normalized_url =
            sdkwork_database_config::workspace_database::normalize_workspace_postgres_url(
                normalized,
            )
            .map_err(|error| PoolError::InvalidUrl(error.to_string()))?;
        postgres_url_with_deployment_tenant(&normalized_url, require_postgres_rls_tenant_id()?)?
    } else {
        normalized.to_string()
    };
    let mut config = DatabaseConfig {
        engine,
        url,
        max_connections: resolve_max_connections(engine, normalized),
        ..DatabaseConfig::default()
    };
    if engine == DatabaseEngine::Postgres {
        config.postgres.ssl_mode = resolve_postgres_ssl_mode(&config.url)?;
    }
    Ok(config)
}

fn postgres_url_with_deployment_tenant(
    database_url: &str,
    tenant_id: u64,
) -> Result<String, PoolError> {
    let mut url = Url::parse(database_url)
        .map_err(|error| PoolError::InvalidUrl(format!("invalid PostgreSQL URL: {error}")))?;
    let mut query_pairs = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    let mut options_index = None;
    for (index, (key, value)) in query_pairs.iter().enumerate() {
        if !key.eq_ignore_ascii_case("options") {
            continue;
        }
        if options_index.replace(index).is_some() {
            return Err(PoolError::DatabaseConfig(
                "PostgreSQL URL must not contain duplicate options parameters".to_string(),
            ));
        }
        if value
            .to_ascii_lowercase()
            .contains(POSTGRES_TENANT_SESSION_KEY)
        {
            return Err(PoolError::DatabaseConfig(format!(
                "PostgreSQL URL must not set {POSTGRES_TENANT_SESSION_KEY}; it is deployment-owned"
            )));
        }
    }

    let tenant_option = format!("-c {POSTGRES_TENANT_SESSION_KEY}={tenant_id}");
    if let Some(index) = options_index {
        let existing = query_pairs[index].1.trim();
        query_pairs[index].1 = if existing.is_empty() {
            tenant_option
        } else {
            format!("{existing} {tenant_option}")
        };
    } else {
        query_pairs.push(("options".to_string(), tenant_option));
    }

    url.query_pairs_mut().clear().extend_pairs(query_pairs);
    Ok(url.into())
}

fn resolve_postgres_ssl_mode(database_url: &str) -> Result<PgSslMode, PoolError> {
    let url = Url::parse(database_url)
        .map_err(|error| PoolError::InvalidUrl(format!("invalid PostgreSQL URL: {error}")))?;
    let url_mode = url
        .query_pairs()
        .find(|(key, _)| key.eq_ignore_ascii_case("sslmode"))
        .map(|(_, value)| value.into_owned());
    let configured_mode = url_mode.or_else(|| {
        std::env::var("SDKWORK_DATABASE_SSL_MODE")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    });
    match configured_mode
        .as_deref()
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        None => Ok(PgSslMode::Prefer),
        Some("disable") => Ok(PgSslMode::Disable),
        Some("allow") => Ok(PgSslMode::Allow),
        Some("prefer") => Ok(PgSslMode::Prefer),
        Some("require") => Ok(PgSslMode::Require),
        Some("verify-ca" | "verify_ca") => Ok(PgSslMode::VerifyCa),
        Some("verify-full" | "verify_full") => Ok(PgSslMode::VerifyFull),
        Some(value) => Err(PoolError::DatabaseConfig(format!(
            "unsupported PostgreSQL SSL mode: {value}"
        ))),
    }
}

async fn connect_knowledgebase_pool_from_config(
    config: DatabaseConfig,
) -> Result<KnowledgebaseDatabasePool, PoolError> {
    create_pool_from_config(config).await
}

async fn connect_knowledgebase_any_pool_from_config(
    config: DatabaseConfig,
) -> Result<AnyPool, PoolError> {
    sqlx::any::install_default_drivers();
    sqlx::any::AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(PoolError::from)
}

pub async fn connect_knowledgebase_pool_from_env() -> Result<KnowledgebaseDatabasePool, PoolError> {
    let mut config = DatabaseConfig::from_env("KNOWLEDGEBASE")?;
    if config.engine == DatabaseEngine::Postgres {
        config.url =
            postgres_url_with_deployment_tenant(&config.url, require_postgres_rls_tenant_id()?)?;
    }
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
    sqlx::any::install_default_drivers();
    sqlx::any::AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|error| sqlx::Error::Configuration(error.to_string().into()))
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
    use super::{postgres_url_with_deployment_tenant, resolve_postgres_ssl_mode};
    use sdkwork_database_config::PgSslMode;
    use url::Url;

    #[test]
    fn postgres_tenant_option_preserves_existing_connection_options() {
        let configured = postgres_url_with_deployment_tenant(
            "postgresql://app:secret@localhost/sdkwork_ai_dev?sslmode=verify-full&options=-c%20search_path%3Dsdkwork_ai_dev%2Cpublic",
            42,
        )
        .expect("tenant-scoped URL");
        let parsed = Url::parse(&configured).expect("valid URL");
        let options = parsed
            .query_pairs()
            .find(|(key, _)| key == "options")
            .map(|(_, value)| value.into_owned())
            .expect("options parameter");
        assert_eq!(
            options,
            "-c search_path=sdkwork_ai_dev,public -c app.current_tenant_id=42"
        );
        assert_eq!(
            resolve_postgres_ssl_mode(&configured).expect("SSL mode"),
            PgSslMode::VerifyFull
        );
    }

    #[test]
    fn postgres_tenant_option_is_added_when_options_are_absent() {
        let configured =
            postgres_url_with_deployment_tenant("postgresql://app@localhost/sdkwork_ai_dev", 7)
                .expect("tenant-scoped URL");
        let parsed = Url::parse(&configured).expect("valid URL");
        assert!(parsed
            .query_pairs()
            .any(|(key, value)| key == "options" && value == "-c app.current_tenant_id=7"));
    }

    #[test]
    fn caller_owned_postgres_tenant_option_is_rejected() {
        let error = postgres_url_with_deployment_tenant(
            "postgresql://app@localhost/sdkwork_ai_dev?options=-c%20app.current_tenant_id%3D99",
            7,
        )
        .expect_err("caller tenant option must fail closed");
        assert!(error.to_string().contains("deployment-owned"));
    }

    #[test]
    fn duplicate_postgres_options_are_rejected() {
        let error = postgres_url_with_deployment_tenant(
            "postgresql://app@localhost/sdkwork_ai_dev?options=-c%20timezone%3DUTC&options=-c%20search_path%3Dsdkwork_ai_dev",
            7,
        )
        .expect_err("duplicate options must fail closed");
        assert!(error.to_string().contains("duplicate options"));
    }
}
