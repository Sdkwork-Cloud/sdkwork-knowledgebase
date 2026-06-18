pub const POSTGRES_CORE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606010001__knowledgebase_core.sql");

pub const POSTGRES_ACCESS_MODE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606170001__knowledge_access_mode.sql");

pub const SQLITE_CORE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606010001__knowledgebase_core.sql");

pub const SQLITE_ACCESS_MODE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606170001__knowledge_access_mode.sql");

pub const SQLITE_MIGRATIONS: &[&str] = &[SQLITE_CORE_MIGRATION, SQLITE_ACCESS_MODE_MIGRATION];

pub const POSTGRES_MIGRATIONS: &[&str] = &[POSTGRES_CORE_MIGRATION, POSTGRES_ACCESS_MODE_MIGRATION];
