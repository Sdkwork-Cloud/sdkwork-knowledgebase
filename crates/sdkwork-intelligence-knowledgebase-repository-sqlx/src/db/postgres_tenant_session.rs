//! Postgres session tenant context for RLS policies (Phase 2.1/2.2).

use sdkwork_database_sqlx::PoolError;
use sdkwork_knowledgebase_contract::{
    parse_canonical_nonnegative_signed_i64, parse_canonical_positive_signed_i64,
};
use sdkwork_knowledgebase_observability::{deployment_tenant_id, is_production_like_environment};
use sqlx::Executor;

/// Session variable read by RLS policies on tenant-scoped tables.
pub const POSTGRES_TENANT_SESSION_KEY: &str = "app.current_tenant_id";
pub const POSTGRES_ORGANIZATION_SESSION_KEY: &str = "app.current_organization_id";

/// Resolves the deployment-bound tenant id used for Postgres RLS session context.
pub fn resolve_postgres_rls_tenant_id() -> u64 {
    deployment_tenant_id()
}

/// Returns the tenant id required for Postgres pool checkout, failing closed in production-like envs.
pub fn require_postgres_rls_tenant_id() -> Result<u64, PoolError> {
    match std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID") {
        Ok(value) => parse_canonical_positive_signed_i64(&value).map_err(|_| {
            PoolError::InvalidUrl(
                "SDKWORK_KNOWLEDGEBASE_TENANT_ID must be a canonical positive signed BIGINT"
                    .to_string(),
            )
        }),
        Err(std::env::VarError::NotPresent) if !is_production_like_environment() => Ok(1),
        Err(_) => Err(PoolError::InvalidUrl(
            "SDKWORK_KNOWLEDGEBASE_TENANT_ID must be set for production-like Postgres deployments"
                .to_string(),
        )),
    }
}

pub fn require_postgres_rls_organization_id() -> Result<u64, PoolError> {
    match std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID") {
        Ok(value) => parse_canonical_nonnegative_signed_i64(&value).map_err(|_| {
            PoolError::InvalidUrl(
                "SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID must be a canonical nonnegative signed BIGINT"
                    .to_string(),
            )
        }),
        Err(std::env::VarError::NotPresent) if !is_production_like_environment() => Ok(0),
        Err(_) => Err(PoolError::InvalidUrl(
            "SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID must be set for production-like Postgres deployments"
                .to_string(),
        )),
    }
}

/// Sets `app.current_tenant_id` for explicit administrative or integration-test connections.
///
/// Deployable one-tenant-per-process runtimes inject this setting through the PostgreSQL
/// connection URL before the process-shared pool is created. Request-shared multi-tenant
/// checkout remains unsupported until transaction-local context is implemented.
pub async fn set_postgres_session_tenant_id<'e, E>(
    executor: E,
    tenant_id: u64,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, false)")
        .bind(POSTGRES_TENANT_SESSION_KEY)
        .bind(tenant_id.to_string())
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn set_postgres_session_organization_id<'e, E>(
    executor: E,
    organization_id: u64,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query("SELECT set_config($1, $2, false)")
        .bind(POSTGRES_ORGANIZATION_SESSION_KEY)
        .bind(organization_id.to_string())
        .execute(executor)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        require_postgres_rls_organization_id, require_postgres_rls_tenant_id,
        POSTGRES_ORGANIZATION_SESSION_KEY, POSTGRES_TENANT_SESSION_KEY,
    };
    use std::sync::{Mutex, MutexGuard};

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn env_test_guard() -> MutexGuard<'static, ()> {
        ENV_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn tenant_session_key_matches_adr() {
        assert_eq!(POSTGRES_TENANT_SESSION_KEY, "app.current_tenant_id");
        assert_eq!(
            POSTGRES_ORGANIZATION_SESSION_KEY,
            "app.current_organization_id"
        );
    }

    #[test]
    fn require_tenant_id_defaults_to_one_in_development() {
        let _guard = env_test_guard();
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID");
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
        assert_eq!(
            require_postgres_rls_tenant_id().expect("development default"),
            1
        );
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
    }

    #[test]
    fn require_tenant_id_fails_closed_in_production_like() {
        let _guard = env_test_guard();
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_TENANT_ID");
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "production");
        assert!(require_postgres_rls_tenant_id().is_err());
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
    }

    #[test]
    fn require_organization_id_is_explicit_in_production_like() {
        let _guard = env_test_guard();
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID");
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "production");
        assert!(require_postgres_rls_organization_id().is_err());
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID", "0");
        assert_eq!(
            require_postgres_rls_organization_id().expect("personal scope"),
            0
        );
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID");
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
    }
}
