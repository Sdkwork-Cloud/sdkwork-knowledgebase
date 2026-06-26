//! Postgres session tenant context for RLS policies (Phase 2.1/2.2).

use sqlx::Executor;

/// Session variable read by RLS policies on tenant-scoped tables.
pub const POSTGRES_TENANT_SESSION_KEY: &str = "app.current_tenant_id";

/// Sets `app.current_tenant_id` on the active Postgres connection (not transaction-local).
///
/// Call after pool checkout before tenant-scoped queries when multiple tenants share one database.
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
    use super::POSTGRES_TENANT_SESSION_KEY;

    #[test]
    fn tenant_session_key_matches_adr() {
        assert_eq!(POSTGRES_TENANT_SESSION_KEY, "app.current_tenant_id");
    }
}
