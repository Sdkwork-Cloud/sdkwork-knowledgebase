use sdkwork_database_config::{DatabaseConfig, DatabaseEngine as SdkDatabaseEngine};
use sdkwork_database_sqlx::{create_any_pool_from_config, PoolError};
use sdkwork_drive_config::DatabaseEngine as DriveDatabaseEngine;
use sdkwork_drive_workspace_service::infrastructure::sql::install_any_schema;
use sqlx::AnyPool;

const DEFAULT_DRIVE_PROVIDER_ID: &str = "sdkwork-knowledgebase-local";
const DEFAULT_DRIVE_BUCKET: &str = "knowledgebase";

const KNOWLEDGEBASE_DRIVE_POOL_MAX_CONNECTIONS: u32 = 5;

pub async fn connect_knowledgebase_drive_pool(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    let (database_config, drive_engine) = drive_database_config_from_url(database_url)?;
    let pool = create_any_pool_from_config(database_config)
        .await
        .map_err(map_pool_error)?;
    install_any_schema(&pool, drive_engine)
        .await
        .map_err(|error| sqlx::Error::Protocol(error.to_string()))?;
    seed_default_drive_storage_provider(&pool, drive_engine).await?;
    Ok(pool)
}

pub async fn knowledgebase_drive_health_check(pool: &AnyPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

fn drive_database_config_from_url(
    database_url: &str,
) -> Result<(DatabaseConfig, DriveDatabaseEngine), sqlx::Error> {
    let normalized = database_url.trim();
    let engine = SdkDatabaseEngine::from_url(normalized).ok_or_else(|| {
        sqlx::Error::Configuration(
            format!("unsupported knowledgebase drive database url: {normalized}").into(),
        )
    })?;
    let drive_engine = match engine {
        SdkDatabaseEngine::Sqlite => DriveDatabaseEngine::Sqlite,
        SdkDatabaseEngine::Postgres => DriveDatabaseEngine::Postgresql,
    };
    Ok((
        DatabaseConfig {
            engine,
            url: normalized.to_string(),
            max_connections: KNOWLEDGEBASE_DRIVE_POOL_MAX_CONNECTIONS,
            ..DatabaseConfig::default()
        },
        drive_engine,
    ))
}

fn map_pool_error(error: PoolError) -> sqlx::Error {
    sqlx::Error::Configuration(error.to_string().into())
}

async fn seed_default_drive_storage_provider(
    pool: &AnyPool,
    engine: DriveDatabaseEngine,
) -> Result<(), sqlx::Error> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM dr_drive_storage_provider WHERE id = $1")
            .bind(DEFAULT_DRIVE_PROVIDER_ID)
            .fetch_optional(pool)
            .await?;
    if exists.is_some() {
        return Ok(());
    }

    let sql = match engine {
        DriveDatabaseEngine::Sqlite => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 'local_filesystem', $2, 'file://localhost', 'local', $2, 1, 1,
            'plain:local:local', NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
        DriveDatabaseEngine::Postgresql => {
            "INSERT INTO dr_drive_storage_provider (
            id, provider_kind, name, endpoint_url, region, bucket, path_style,
            strict_tls, credential_ref, server_side_encryption_mode, default_storage_class,
            status, version, created_by, updated_by
        ) VALUES (
            $1, 'local_filesystem', $2, 'file://localhost', 'local', $2, TRUE, TRUE,
            'plain:local:local', NULL, NULL, 'active', 1, 'system', 'system'
        )"
        }
    };

    sqlx::query(sql)
        .bind(DEFAULT_DRIVE_PROVIDER_ID)
        .bind(DEFAULT_DRIVE_BUCKET)
        .execute(pool)
        .await?;
    Ok(())
}
