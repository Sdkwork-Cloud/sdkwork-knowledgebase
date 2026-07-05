//! Postgres session tenant context for RLS policies (Phase 2.1/2.2).

use sdkwork_database_sqlx::PoolError;
use sdkwork_knowledgebase_observability::{deployment_tenant_id, is_production_like_environment};
use sqlx::Executor;

/// Session variable read by RLS policies on tenant-scoped tables.
pub const POSTGRES_TENANT_SESSION_KEY: &str = "app.current_tenant_id";

/// Resolves the deployment-bound tenant id used for Postgres RLS session context.
pub fn resolve_postgres_rls_tenant_id() -> u64 {
    deployment_tenant_id()
}

/// Returns the tenant id required for Postgres pool checkout, failing closed in production-like envs.
pub fn require_postgres_rls_tenant_id() -> Result<u64, PoolError> {
    let tenant_id = resolve_postgres_rls_tenant_id();
    if tenant_id == 0 && is_production_like_environment() {
        return Err(PoolError::InvalidUrl(
            "SDKWORK_KNOWLEDGEBASE_TENANT_ID must be set for production-like Postgres deployments"
                .to_string(),
        ));
    }
    Ok(if tenant_id == 0 { 1 } else { tenant_id })
}

/// Sets `app.current_tenant_id` on the active Postgres connection.
///
/// For deployment-dedicated processes, `after_connect` sets the deployment tenant.
/// For shared Postgres pools, call this after every `acquire()` with the authenticated
/// request tenant before running tenant-scoped queries.
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

#[cfg(test)]
mod tests {
    use super::{require_postgres_rls_tenant_id, POSTGRES_TENANT_SESSION_KEY};
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
}
