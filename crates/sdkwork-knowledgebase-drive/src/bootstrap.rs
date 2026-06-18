use sdkwork_drive_config::DatabaseEngine;
use sdkwork_drive_workspace_service::infrastructure::sql::install_any_schema;
use sqlx::AnyPool;

const DEFAULT_DRIVE_PROVIDER_ID: &str = "sdkwork-knowledgebase-local";
const DEFAULT_DRIVE_BUCKET: &str = "knowledgebase";

pub async fn connect_sqlite_drive_pool(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    sqlx::any::install_default_drivers();
    let pool = sqlx::any::AnyPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    install_any_schema(&pool, DatabaseEngine::Sqlite)
        .await
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
    seed_default_drive_storage_provider(&pool).await?;
    Ok(pool)
}

pub async fn sqlite_drive_health_check(pool: &AnyPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

async fn seed_default_drive_storage_provider(pool: &AnyPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            ?1, 'local', ?1, 'file://localhost', 'local', ?2, 1, 1,
            'plain:local:local', NULL, NULL, 'active', 1, 'system', 'system'
        )",
    )
    .bind(DEFAULT_DRIVE_PROVIDER_ID)
    .bind(DEFAULT_DRIVE_BUCKET)
    .execute(pool)
    .await?;
    Ok(())
}
