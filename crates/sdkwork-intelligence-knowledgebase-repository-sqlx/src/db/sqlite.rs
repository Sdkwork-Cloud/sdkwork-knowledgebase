use sqlx::AnyPool;

use crate::migrations::SQLITE_MIGRATIONS;

pub async fn connect_sqlite_pool(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    crate::db::bootstrap::connect_sqlite_pool_via_framework(database_url).await
}

pub async fn install_sqlite_core_schema(pool: &AnyPool) -> Result<(), sqlx::Error> {
    install_sqlite_schema(pool).await
}

pub async fn install_sqlite_schema(pool: &AnyPool) -> Result<(), sqlx::Error> {
    for migration in SQLITE_MIGRATIONS {
        for statement in migration.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                execute_idempotent_sqlite_statement(pool, statement).await?;
            }
        }
    }
    Ok(())
}

async fn execute_idempotent_sqlite_statement(
    pool: &AnyPool,
    statement: &str,
) -> Result<(), sqlx::Error> {
    match sqlx::query(statement).execute(pool).await {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(error)) if is_idempotent_sqlite_schema_error(error.message()) => {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn is_idempotent_sqlite_schema_error(message: &str) -> bool {
    message.contains("duplicate column name") || message.contains("already exists")
}

async fn bootstrap_sqlite_file_database(database_url: &str) -> Result<(), sqlx::Error> {
    let pool = crate::db::bootstrap::connect_knowledgebase_pool_from_url(database_url)
        .await
        .map_err(|error| sqlx::Error::Configuration(error.to_string().into()))?;
    crate::db::bootstrap::bootstrap_knowledgebase_database(pool)
        .await
        .map_err(|error| sqlx::Error::Configuration(error.into()))?;
    Ok(())
}

fn is_memory_sqlite_database_url(database_url: &str) -> bool {
    let normalized = database_url.trim().to_ascii_lowercase();
    normalized == "sqlite::memory:" || normalized.contains("mode=memory")
}

pub async fn connect_sqlite_and_install_schema(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    if is_memory_sqlite_database_url(database_url) {
        let pool = connect_sqlite_pool(database_url).await?;
        install_sqlite_core_schema(&pool).await?;
        return Ok(pool);
    }
    bootstrap_sqlite_file_database(database_url).await?;
    connect_sqlite_pool(database_url).await
}

pub async fn sqlite_health_check(pool: &AnyPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}
