use sqlx::{Acquire, AnyPool, Row};

use crate::migrations::{
    SQLITE_GROUP_ARCHIVE_SAGA_MIGRATION, SQLITE_GROUP_ARCHIVE_SAGA_SCOPE_TRIGGERS_MIGRATION,
    SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION, SQLITE_MIGRATIONS,
};

const GROUP_BINDING_TABLE: &str = "kb_group_knowledge_space_binding";
const GROUP_BINDING_REBUILD_TABLE: &str = "kb_group_knowledge_space_binding__lifecycle_upgrade";
const GROUP_BINDING_REBUILD_COLUMNS: &[&str] = &[
    "id",
    "uuid",
    "tenant_id",
    "organization_id",
    "conversation_id",
    "space_id",
    "space_uuid",
    "group_name",
    "lifecycle_state",
    "acl_projection_state",
    "provisioning_idempotency_key_sha256_hex",
    "provisioning_lease_token",
    "provisioning_lease_until",
    "membership_epoch",
    "last_source_event_id",
    "last_error_code",
    "last_error_at",
    "archived_at",
    "archived_by",
    "deleted_at",
    "deleted_by",
    "created_by",
    "updated_by",
    "created_at",
    "updated_at",
    "version",
    "upstream_link_generation",
    "archive_source_event_id",
    "archive_payload_sha256_hex",
    "archive_lease_token",
    "archive_lease_until",
    "archive_acl_cursor",
    "archive_acl_pages_processed",
    "archive_acl_cleanup_completed_at",
];

pub async fn connect_sqlite_pool(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    crate::db::bootstrap::connect_sqlite_pool_via_framework(database_url).await
}

pub async fn install_sqlite_core_schema(pool: &AnyPool) -> Result<(), sqlx::Error> {
    install_sqlite_schema(pool).await
}

pub async fn install_sqlite_schema(pool: &AnyPool) -> Result<(), sqlx::Error> {
    for migration in SQLITE_MIGRATIONS {
        apply_sqlite_migration(pool, migration).await?;
    }
    if group_binding_lifecycle_rebuild_required(pool).await? {
        rebuild_group_binding_lifecycle_constraint(pool).await?;
        // Rebuilding a SQLite table drops its indexes and table-owned triggers. Reapply the
        // authored, idempotent sources rather than duplicating those artifacts in the upgrader.
        apply_sqlite_migration(pool, SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION).await?;
        apply_sqlite_migration(pool, SQLITE_GROUP_ARCHIVE_SAGA_MIGRATION).await?;
        apply_sqlite_migration(pool, SQLITE_GROUP_ARCHIVE_SAGA_SCOPE_TRIGGERS_MIGRATION).await?;
    }
    Ok(())
}

async fn apply_sqlite_migration(pool: &AnyPool, migration: &str) -> Result<(), sqlx::Error> {
    if migration_contains_trigger_program(migration) {
        // SQLite trigger bodies contain statement delimiters between BEGIN and END. The migration
        // source is compile-time-owned, so execute it as one raw program rather than splitting a
        // valid trigger into incomplete SQL fragments.
        sqlx::raw_sql(migration).execute(pool).await?;
        return Ok(());
    }
    for statement in migration.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            execute_idempotent_sqlite_statement(pool, statement).await?;
        }
    }
    Ok(())
}

async fn group_binding_lifecycle_rebuild_required(pool: &AnyPool) -> Result<bool, sqlx::Error> {
    let table_sql = sqlx::query_scalar::<_, Option<String>>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = $1",
    )
    .bind(GROUP_BINDING_TABLE)
    .fetch_one(pool)
    .await?;
    Ok(table_sql.is_some_and(|sql| {
        let normalized = sql.to_ascii_lowercase();
        normalized.contains("lifecycle_state") && !normalized.contains("archiving")
    }))
}

async fn rebuild_group_binding_lifecycle_constraint(pool: &AnyPool) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    let foreign_keys_enabled = sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
        .fetch_one(&mut *connection)
        .await?
        != 0;
    if foreign_keys_enabled {
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *connection)
            .await?;
    }

    let rebuild_result = async {
        validate_group_binding_rebuild_columns(&mut connection).await?;
        let mut transaction = connection.begin().await?;
        sqlx::raw_sql(&group_binding_rebuild_create_sql()?)
            .execute(&mut *transaction)
            .await?;
        for statement in SQLITE_GROUP_ARCHIVE_SAGA_MIGRATION.split(';') {
            let statement = statement.trim();
            let Some(statement) = statement.find("ALTER TABLE").map(|index| &statement[index..]) else {
                continue;
            };
            let statement = statement.replace(GROUP_BINDING_TABLE, GROUP_BINDING_REBUILD_TABLE);
            sqlx::query(&statement).execute(&mut *transaction).await?;
        }
        let columns = GROUP_BINDING_REBUILD_COLUMNS.join(", ");
        let copy_sql = format!(
            "INSERT INTO {GROUP_BINDING_REBUILD_TABLE} ({columns}) SELECT {columns} FROM {GROUP_BINDING_TABLE}"
        );
        sqlx::query(&copy_sql).execute(&mut *transaction).await?;
        drop_group_space_triggers(&mut transaction).await?;
        sqlx::query(&format!("DROP TABLE {GROUP_BINDING_TABLE}"))
            .execute(&mut *transaction)
            .await?;
        sqlx::query(&format!(
            "ALTER TABLE {GROUP_BINDING_REBUILD_TABLE} RENAME TO {GROUP_BINDING_TABLE}"
        ))
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await
    }
    .await;

    let restore_foreign_keys_result = async {
        if foreign_keys_enabled {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *connection)
                .await?;
        }
        Ok::<(), sqlx::Error>(())
    }
    .await;
    rebuild_result?;
    restore_foreign_keys_result?;

    let foreign_key_violation = sqlx::query("PRAGMA foreign_key_check")
        .fetch_optional(&mut *connection)
        .await?;
    if foreign_key_violation.is_some() {
        return Err(sqlx::Error::Configuration(
            "group binding lifecycle upgrade left a foreign-key violation".into(),
        ));
    }
    Ok(())
}

async fn drop_group_space_triggers(
    transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
) -> Result<(), sqlx::Error> {
    let trigger_rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'trg_kb_group_space_%'",
    )
    .fetch_all(&mut **transaction)
    .await?;
    for row in trigger_rows {
        let trigger_name: String = row.try_get("name")?;
        if !trigger_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(sqlite_upgrade_configuration_error(
                "group binding lifecycle upgrade found an unsafe trigger name",
            ));
        }
        sqlx::query(&format!("DROP TRIGGER {trigger_name}"))
            .execute(&mut **transaction)
            .await?;
    }
    Ok(())
}

async fn validate_group_binding_rebuild_columns(
    connection: &mut sqlx::AnyConnection,
) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(&format!("PRAGMA table_info({GROUP_BINDING_TABLE})"))
        .fetch_all(&mut *connection)
        .await?;
    let columns = rows
        .iter()
        .map(|row| row.try_get::<String, _>("name"))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let expected = GROUP_BINDING_REBUILD_COLUMNS
        .iter()
        .map(|column| (*column).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    if columns != expected {
        return Err(sqlx::Error::Configuration(
            "group binding lifecycle upgrade encountered an unsupported table shape".into(),
        ));
    }
    Ok(())
}

fn group_binding_rebuild_create_sql() -> Result<String, sqlx::Error> {
    const CREATE_PREFIX: &str = "CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_binding";
    const CREATE_END_MARKER: &str =
        "\n);\n\nCREATE UNIQUE INDEX IF NOT EXISTS uk_kb_group_knowledge_space_binding_uuid";

    let start = SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION
        .find(CREATE_PREFIX)
        .ok_or_else(|| {
            sqlite_upgrade_configuration_error("group binding create source is missing")
        })?;
    let source = &SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION[start..];
    let end = source.find(CREATE_END_MARKER).ok_or_else(|| {
        sqlite_upgrade_configuration_error("group binding create source has an unexpected shape")
    })?;
    Ok(source[..end + 3].replacen(
        CREATE_PREFIX,
        &format!("CREATE TABLE {GROUP_BINDING_REBUILD_TABLE}"),
        1,
    ))
}

fn sqlite_upgrade_configuration_error(message: &str) -> sqlx::Error {
    sqlx::Error::Configuration(message.to_string().into())
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

fn migration_contains_trigger_program(migration: &str) -> bool {
    migration.contains("CREATE TRIGGER")
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

#[cfg(test)]
mod tests {
    use super::{connect_sqlite_pool, install_sqlite_schema};
    use crate::migrations::SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION;
    use sqlx::Row;
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn installs_sqlite_trigger_migrations_without_fragmenting_trigger_bodies() {
        let pool = connect_sqlite_pool("sqlite::memory:")
            .await
            .expect("sqlite pool");

        install_sqlite_schema(&pool)
            .await
            .expect("install schema with group trigger migration");
        install_sqlite_schema(&pool)
            .await
            .expect("schema install remains idempotent");

        let installed_triggers = sqlx::query_scalar::<_, String>(
            "SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'trg_kb_group_space_%' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .expect("list group triggers")
        .into_iter()
        .collect::<BTreeSet<_>>();
        let expected_triggers = [
            "trg_kb_group_space_active_acl_insert",
            "trg_kb_group_space_active_acl_update",
            "trg_kb_group_space_binding_lifecycle_insert",
            "trg_kb_group_space_binding_lifecycle_update",
            "trg_kb_group_space_binding_organization_insert",
            "trg_kb_group_space_binding_organization_update",
            "trg_kb_group_space_binding_tenant_insert",
            "trg_kb_group_space_binding_tenant_update",
            "trg_kb_group_space_event_inbox_organization_insert",
            "trg_kb_group_space_event_inbox_organization_update",
            "trg_kb_group_space_event_scope_binding_insert",
            "trg_kb_group_space_event_scope_binding_update",
            "trg_kb_group_space_event_inbox_tenant_insert",
            "trg_kb_group_space_event_inbox_tenant_update",
            "trg_kb_group_space_member_organization_insert",
            "trg_kb_group_space_member_organization_update",
            "trg_kb_group_space_member_role_access_insert",
            "trg_kb_group_space_member_role_access_update",
            "trg_kb_group_space_member_scope_binding_insert",
            "trg_kb_group_space_member_scope_binding_update",
            "trg_kb_group_space_member_tenant_insert",
            "trg_kb_group_space_member_tenant_update",
            "trg_kb_group_space_membership_projection_organization_insert",
            "trg_kb_group_space_membership_projection_organization_update",
            "trg_kb_group_space_membership_projection_tenant_insert",
            "trg_kb_group_space_membership_projection_tenant_update",
            "trg_kb_group_space_projection_scope_binding_insert",
            "trg_kb_group_space_projection_scope_binding_update",
            "trg_kb_group_space_projection_state_insert",
            "trg_kb_group_space_projection_state_update",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
        assert_eq!(installed_triggers, expected_triggers);
    }

    #[tokio::test]
    async fn upgrades_historic_group_lifecycle_check_without_losing_bound_children() {
        let pool = connect_sqlite_pool("sqlite::memory:")
            .await
            .expect("sqlite pool");
        let mut connection = pool.acquire().await.expect("sqlite connection");
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *connection)
            .await
            .expect("temporarily disable foreign keys for the historic pre-core fixture");
        let historic_group_migration = SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION.replacen(
            "'provisioning', 'active', 'failed', 'archiving', 'archived', 'deleted'",
            "'provisioning', 'active', 'failed', 'archived', 'deleted'",
            1,
        );
        sqlx::raw_sql(&historic_group_migration)
            .execute(&mut *connection)
            .await
            .expect("install historic group binding table");
        sqlx::query(
            r#"
            INSERT INTO kb_group_knowledge_space_binding (
                id, uuid, tenant_id, organization_id, conversation_id, group_name,
                lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
                membership_epoch, created_by, updated_by, created_at, updated_at, version
            ) VALUES (
                1, 'legacy-binding', 100, 200, 'legacy-conversation', 'Legacy Group',
                'provisioning', 'pending', 'legacy-key', 0, 'owner', 'owner',
                '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
            )
            "#,
        )
        .execute(&mut *connection)
        .await
        .expect("insert historic binding");
        sqlx::query(
            r#"
            INSERT INTO kb_group_knowledge_space_member (
                id, uuid, tenant_id, organization_id, binding_id, principal_kind, actor_id,
                member_role, access_level, membership_epoch, status, created_at, updated_at, version
            ) VALUES (
                2, 'legacy-member', 100, 200, 1, 'user', 'owner', 'owner', 'owner', 0, 1,
                '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0
            )
            "#,
        )
        .execute(&mut *connection)
        .await
        .expect("insert historic member");
        sqlx::query(
            r#"
            INSERT INTO kb_group_knowledge_space_event_inbox (
                id, uuid, tenant_id, organization_id, source_event_id, event_type, binding_id,
                payload_sha256_hex, applied_at
            ) VALUES (
                3, 'legacy-event', 100, 200, 'legacy-event', 'group.members.synchronized', 1,
                'legacy-payload', '2026-07-13T00:00:00Z'
            )
            "#,
        )
        .execute(&mut *connection)
        .await
        .expect("insert historic event");
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *connection)
            .await
            .expect("restore foreign keys for schema upgrade");
        drop(connection);

        install_sqlite_schema(&pool)
            .await
            .expect("upgrade historic group lifecycle check");

        let table_sql: String = sqlx::query_scalar(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'kb_group_knowledge_space_binding'",
        )
        .fetch_one(&pool)
        .await
        .expect("load upgraded binding DDL");
        assert!(table_sql.contains("'archiving'"));
        assert!(table_sql.contains("CHECK (tenant_id > 0)"));
        for table in [
            "kb_group_knowledge_space_binding",
            "kb_group_knowledge_space_member",
            "kb_group_knowledge_space_event_inbox",
        ] {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&pool)
                .await
                .expect("preserved group rows");
            assert_eq!(count, 1, "{table} rows must survive the lifecycle upgrade");
        }
        sqlx::query(
            "UPDATE kb_group_knowledge_space_binding SET lifecycle_state = 'archiving' WHERE id = 1",
        )
        .execute(&pool)
        .await
        .expect("upgraded lifecycle check accepts archiving");
        let tenant_error = sqlx::query(
            r#"
            INSERT INTO kb_group_knowledge_space_binding (
                id, uuid, tenant_id, organization_id, conversation_id, group_name,
                lifecycle_state, acl_projection_state, provisioning_idempotency_key_sha256_hex,
                membership_epoch, created_by, updated_by, created_at, updated_at, version,
                upstream_link_generation, archive_acl_pages_processed
            ) VALUES (
                4, 'invalid-tenant', 0, 200, 'invalid-tenant-conversation', 'Invalid Tenant',
                'provisioning', 'pending', 'invalid-tenant-key', 0, 'owner', 'owner',
                '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0, 0, 0
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect_err("upgraded binding must reject tenant_id = 0");
        assert!(tenant_error.to_string().contains("tenant_id"));

        let foreign_key_violation = sqlx::query("PRAGMA foreign_key_check")
            .fetch_optional(&pool)
            .await
            .expect("check rebuilt foreign keys");
        assert!(foreign_key_violation.is_none());
        let member_binding_id: i64 =
            sqlx::query("SELECT binding_id FROM kb_group_knowledge_space_member")
                .fetch_one(&pool)
                .await
                .expect("load preserved member")
                .try_get("binding_id")
                .expect("member binding id");
        assert_eq!(member_binding_id, 1);
    }
}
