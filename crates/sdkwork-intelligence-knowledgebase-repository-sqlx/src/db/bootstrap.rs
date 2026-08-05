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
    require_postgres_rls_organization_id, require_postgres_rls_tenant_id,
    POSTGRES_ORGANIZATION_SESSION_KEY, POSTGRES_TENANT_SESSION_KEY,
};

pub type KnowledgebaseDatabasePool = DatabasePool;

const DEFAULT_SQLITE_POOL_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_POSTGRES_PROCESS_MAX_CONNECTIONS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgebaseProcessPoolBudget {
    pub any_max_connections: u32,
    pub postgres_max_connections: Option<u32>,
}

fn configured_process_max_connections() -> Result<Option<u32>, PoolError> {
    let Some(value) = std::env::var("SDKWORK_DATABASE_MAX_CONNECTIONS").ok() else {
        return Ok(None);
    };
    let max_connections = value.trim().parse::<u32>().map_err(|_| {
        PoolError::DatabaseConfig(
            "SDKWORK_DATABASE_MAX_CONNECTIONS must be a positive integer".to_string(),
        )
    })?;
    if max_connections == 0 {
        return Err(PoolError::DatabaseConfig(
            "SDKWORK_DATABASE_MAX_CONNECTIONS must be greater than zero".to_string(),
        ));
    }
    Ok(Some(max_connections))
}

fn process_pool_budget(
    engine: DatabaseEngine,
    database_url: &str,
    configured_max_connections: Option<u32>,
) -> Result<KnowledgebaseProcessPoolBudget, PoolError> {
    if engine == DatabaseEngine::Sqlite && database_url.trim() == "sqlite::memory:" {
        return Ok(KnowledgebaseProcessPoolBudget {
            any_max_connections: 1,
            postgres_max_connections: None,
        });
    }
    if engine == DatabaseEngine::Sqlite {
        return Ok(KnowledgebaseProcessPoolBudget {
            any_max_connections: configured_max_connections
                .unwrap_or(DEFAULT_SQLITE_POOL_MAX_CONNECTIONS),
            postgres_max_connections: None,
        });
    }

    let total = configured_max_connections.unwrap_or(DEFAULT_POSTGRES_PROCESS_MAX_CONNECTIONS);
    if total < 2 {
        return Err(PoolError::DatabaseConfig(
            "PostgreSQL Knowledgebase requires SDKWORK_DATABASE_MAX_CONNECTIONS >= 2 because the process owns one typed pool and one compatibility pool"
                .to_string(),
        ));
    }
    let postgres_max_connections = total.div_ceil(2);
    Ok(KnowledgebaseProcessPoolBudget {
        any_max_connections: total - postgres_max_connections,
        postgres_max_connections: Some(postgres_max_connections),
    })
}

pub fn knowledgebase_process_pool_budget_from_url(
    database_url: &str,
) -> Result<KnowledgebaseProcessPoolBudget, PoolError> {
    let normalized = database_url.trim();
    let engine = DatabaseEngine::from_url(normalized).ok_or_else(|| {
        PoolError::InvalidUrl(format!(
            "unsupported knowledgebase database url: {normalized}"
        ))
    })?;
    process_pool_budget(engine, normalized, configured_process_max_connections()?)
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
        postgres_url_with_deployment_scope(
            &normalized_url,
            require_postgres_rls_tenant_id()?,
            require_postgres_rls_organization_id()?,
        )?
    } else {
        normalized.to_string()
    };
    let pool_budget =
        process_pool_budget(engine, normalized, configured_process_max_connections()?)?;
    let mut config = DatabaseConfig {
        engine,
        url,
        max_connections: pool_budget.any_max_connections,
        ..DatabaseConfig::default()
    };
    if engine == DatabaseEngine::Postgres {
        config.postgres.ssl_mode = resolve_postgres_ssl_mode(&config.url)?;
    }
    Ok(config)
}

fn postgres_url_with_deployment_scope(
    database_url: &str,
    tenant_id: u64,
    organization_id: u64,
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
        let normalized_options = value.to_ascii_lowercase();
        if normalized_options.contains(POSTGRES_TENANT_SESSION_KEY)
            || normalized_options.contains(POSTGRES_ORGANIZATION_SESSION_KEY)
        {
            return Err(PoolError::DatabaseConfig(
                "PostgreSQL URL must not set deployment-owned tenant or organization scope"
                    .to_string(),
            ));
        }
    }

    let scope_options = format!(
        "-c {POSTGRES_TENANT_SESSION_KEY}={tenant_id} -c {POSTGRES_ORGANIZATION_SESSION_KEY}={organization_id}"
    );
    if let Some(index) = options_index {
        let existing = query_pairs[index].1.trim();
        query_pairs[index].1 = if existing.is_empty() {
            scope_options
        } else {
            format!("{existing} {scope_options}")
        };
    } else {
        query_pairs.push(("options".to_string(), scope_options));
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
    // The Any compatibility pool must honor the same TLS policy as the typed pool: when the
    // URL carries no sslmode and `SDKWORK_DATABASE_SSL_MODE` (or the resolved default) is
    // stricter than the sqlx default (`prefer`), the mode is materialized into the URL so a
    // misconfigured plaintext fallback is impossible.
    let url = if config.engine == DatabaseEngine::Postgres {
        ensure_url_ssl_mode(&config.url, config.postgres.ssl_mode)?
    } else {
        config.url
    };
    sqlx::any::AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&url)
        .await
        .map_err(PoolError::from)
}

/// Appends `sslmode=<mode>` to a PostgreSQL URL when it does not already declare one, so
/// connection pools built from raw URLs (Any driver) cannot silently fall back to plaintext.
fn ensure_url_ssl_mode(database_url: &str, mode: PgSslMode) -> Result<String, PoolError> {
    let mut url = Url::parse(database_url)
        .map_err(|error| PoolError::InvalidUrl(format!("invalid PostgreSQL URL: {error}")))?;
    if url
        .query_pairs()
        .any(|(key, _)| key.eq_ignore_ascii_case("sslmode"))
    {
        return Ok(url.into());
    }
    let mode_name = match mode {
        PgSslMode::Disable => "disable",
        PgSslMode::Allow => "allow",
        PgSslMode::Prefer => "prefer",
        PgSslMode::Require => "require",
        PgSslMode::VerifyCa => "verify-ca",
        PgSslMode::VerifyFull => "verify-full",
    };
    url.query_pairs_mut()
        .append_pair("sslmode", mode_name);
    Ok(url.into())
}

pub async fn connect_knowledgebase_pool_from_env() -> Result<KnowledgebaseDatabasePool, PoolError> {
    let mut config = DatabaseConfig::from_env("KNOWLEDGEBASE")?;
    if config.engine == DatabaseEngine::Postgres {
        config.url = postgres_url_with_deployment_scope(
            &config.url,
            require_postgres_rls_tenant_id()?,
            require_postgres_rls_organization_id()?,
        )?;
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
    use super::{
        ensure_url_ssl_mode, postgres_url_with_deployment_scope, process_pool_budget,
        resolve_postgres_ssl_mode, KnowledgebaseProcessPoolBudget,
    };
    use sdkwork_database_config::{DatabaseEngine, PgSslMode};
    use url::Url;

    #[test]
    fn postgres_process_pool_budget_is_bounded_and_prefers_the_typed_pool() {
        assert_eq!(
            process_pool_budget(
                DatabaseEngine::Postgres,
                "postgresql://localhost/db",
                Some(5)
            )
            .expect("allocate pool budget"),
            KnowledgebaseProcessPoolBudget {
                any_max_connections: 2,
                postgres_max_connections: Some(3),
            }
        );
        assert!(process_pool_budget(
            DatabaseEngine::Postgres,
            "postgresql://localhost/db",
            Some(1)
        )
        .is_err());
    }

    #[test]
    fn in_memory_sqlite_pool_is_forced_to_one_connection() {
        assert_eq!(
            process_pool_budget(DatabaseEngine::Sqlite, "sqlite::memory:", Some(100))
                .expect("allocate SQLite pool budget"),
            KnowledgebaseProcessPoolBudget {
                any_max_connections: 1,
                postgres_max_connections: None,
            }
        );
    }

    #[test]
    fn postgres_tenant_option_preserves_existing_connection_options() {
        let configured = postgres_url_with_deployment_scope(
            "postgresql://app:secret@localhost/sdkwork_ai_dev?sslmode=verify-full&options=-c%20search_path%3Dsdkwork_ai_dev%2Cpublic",
            42,
            7,
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
            "-c search_path=sdkwork_ai_dev,public -c app.current_tenant_id=42 -c app.current_organization_id=7"
        );
        assert_eq!(
            resolve_postgres_ssl_mode(&configured).expect("SSL mode"),
            PgSslMode::VerifyFull
        );
    }

    #[test]
    fn postgres_tenant_option_is_added_when_options_are_absent() {
        let configured =
            postgres_url_with_deployment_scope("postgresql://app@localhost/sdkwork_ai_dev", 7, 11)
                .expect("tenant-scoped URL");
        let parsed = Url::parse(&configured).expect("valid URL");
        assert!(parsed.query_pairs().any(|(key, value)| key == "options"
            && value == "-c app.current_tenant_id=7 -c app.current_organization_id=11"));
    }

    #[test]
    fn caller_owned_postgres_tenant_option_is_rejected() {
        let error = postgres_url_with_deployment_scope(
            "postgresql://app@localhost/sdkwork_ai_dev?options=-c%20app.current_tenant_id%3D99",
            7,
            11,
        )
        .expect_err("caller tenant option must fail closed");
        assert!(error.to_string().contains("deployment-owned"));
    }

    #[test]
    fn duplicate_postgres_options_are_rejected() {
        let error = postgres_url_with_deployment_scope(
            "postgresql://app@localhost/sdkwork_ai_dev?options=-c%20timezone%3DUTC&options=-c%20search_path%3Dsdkwork_ai_dev",
            7,
            11,
        )
        .expect_err("duplicate options must fail closed");
        assert!(error.to_string().contains("duplicate options"));
    }

    #[test]
    fn any_pool_url_materializes_ssl_mode_when_absent() {
        let configured =
            ensure_url_ssl_mode("postgresql://app@localhost/kb", PgSslMode::VerifyFull)
                .expect("SSL mode materialized");
        let parsed = Url::parse(&configured).expect("valid URL");
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(key, _)| key == "sslmode")
                .map(|(_, value)| value.into_owned())
                .as_deref(),
            Some("verify-full")
        );
    }

    #[test]
    fn any_pool_url_preserves_explicit_ssl_mode() {
        let configured =
            ensure_url_ssl_mode("postgresql://app@localhost/kb?sslmode=require", PgSslMode::Disable)
                .expect("explicit mode preserved");
        let parsed = Url::parse(&configured).expect("valid URL");
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(key, _)| key == "sslmode")
                .map(|(_, value)| value.into_owned())
                .as_deref(),
            Some("require")
        );
    }
}
