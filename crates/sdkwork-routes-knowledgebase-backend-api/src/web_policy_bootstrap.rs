//! Idempotent dev/staging seeds for tenant-scoped web policy tables.

use sdkwork_utils_rust::is_blank;
use sdkwork_web_core::{limits_for_tier, rate_limit_tier_key, RateLimitTier};
use sdkwork_web_store_sqlx::WebStoreDatabaseHost;

const DEFAULT_TENANT_ID: &str = "100001";

pub async fn seed_default_tenant_web_policies(host: &WebStoreDatabaseHost) -> Result<(), String> {
    let tenant_id = std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| DEFAULT_TENANT_ID.to_string());
    let environment = std::env::var("SDKWORK_ENVIRONMENT")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| "dev".to_string());

    if let Some(pool) = host.pool().as_sqlite() {
        seed_sqlite(pool, &tenant_id, &environment).await
    } else if let Some(pool) = host.pool().as_postgres() {
        seed_postgres(pool, &tenant_id, &environment).await
    } else {
        Ok(())
    }
}

async fn seed_sqlite(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    tenant_id: &str,
    environment: &str,
) -> Result<(), String> {
    for tier in [
        RateLimitTier::AuthCritical,
        RateLimitTier::OpenApiDefault,
        RateLimitTier::Upload,
        RateLimitTier::Search,
        RateLimitTier::Bulk,
        RateLimitTier::Worker,
        RateLimitTier::Internal,
    ] {
        let limits = limits_for_tier(tier);
        let tier_key = rate_limit_tier_key(Some(tier));
        sqlx::query(
            "INSERT OR IGNORE INTO web_rate_limit_policy \
             (tenant_id, environment, tier_key, max_requests, window_secs, enabled, version) \
             VALUES (?, ?, ?, ?, ?, 1, 1)",
        )
        .bind(tenant_id)
        .bind(environment)
        .bind(tier_key)
        .bind(i64::from(limits.max_requests))
        .bind(i64::try_from(limits.window_secs).unwrap_or(i64::MAX))
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    }

    sqlx::query(
        "INSERT OR IGNORE INTO web_tenant_runtime_profile \
         (tenant_id, environment, rate_limit_enabled, max_content_length, max_concurrent_requests, version) \
         VALUES (?, ?, 1, NULL, NULL, 1)",
    )
    .bind(tenant_id)
    .bind(environment)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

async fn seed_postgres(
    pool: &sqlx::Pool<sqlx::Postgres>,
    tenant_id: &str,
    environment: &str,
) -> Result<(), String> {
    for tier in [
        RateLimitTier::AuthCritical,
        RateLimitTier::OpenApiDefault,
        RateLimitTier::Upload,
        RateLimitTier::Search,
        RateLimitTier::Bulk,
        RateLimitTier::Worker,
        RateLimitTier::Internal,
    ] {
        let limits = limits_for_tier(tier);
        let tier_key = rate_limit_tier_key(Some(tier));
        sqlx::query(
            "INSERT INTO web_rate_limit_policy \
             (tenant_id, environment, tier_key, max_requests, window_secs, enabled, version) \
             VALUES ($1, $2, $3, $4, $5, TRUE, 1) \
             ON CONFLICT (tenant_id, environment, tier_key) DO NOTHING",
        )
        .bind(tenant_id)
        .bind(environment)
        .bind(tier_key)
        .bind(i64::from(limits.max_requests))
        .bind(i64::try_from(limits.window_secs).unwrap_or(i64::MAX))
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    }

    sqlx::query(
        "INSERT INTO web_tenant_runtime_profile \
         (tenant_id, environment, rate_limit_enabled, max_content_length, max_concurrent_requests, version) \
         VALUES ($1, $2, TRUE, NULL, NULL, 1) \
         ON CONFLICT (tenant_id, environment) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(environment)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}
