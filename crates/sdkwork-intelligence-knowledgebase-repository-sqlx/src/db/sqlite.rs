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

pub async fn connect_sqlite_and_install_schema(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    let pool = connect_sqlite_pool(database_url).await?;
    install_sqlite_core_schema(&pool).await?;
    Ok(pool)
}

pub async fn sqlite_health_check(pool: &AnyPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}
