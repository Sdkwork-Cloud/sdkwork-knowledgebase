pub const POSTGRES_CORE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606010001__knowledgebase_core.sql");

pub const POSTGRES_ACCESS_MODE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606170001__knowledge_access_mode.sql");

pub const POSTGRES_AGENT_IMPLEMENTATION_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606180001__agent_implementation.sql");

pub const POSTGRES_CONTEXT_BINDING_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606140001__knowledgebase_context_binding.sql");

pub const POSTGRES_PGVECTOR_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606190001__knowledgebase_pgvector.sql");

pub const POSTGRES_OUTBOX_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606200001__knowledgebase_outbox.sql");

pub const SQLITE_CORE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606010001__knowledgebase_core.sql");

pub const SQLITE_ACCESS_MODE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606170001__knowledge_access_mode.sql");

pub const SQLITE_AGENT_IMPLEMENTATION_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606180001__agent_implementation.sql");

pub const SQLITE_CONTEXT_BINDING_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606140001__knowledgebase_context_binding.sql");

pub const SQLITE_OUTBOX_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606200001__knowledgebase_outbox.sql");

pub const SQLITE_MIGRATIONS: &[&str] = &[
    SQLITE_CORE_MIGRATION,
    SQLITE_CONTEXT_BINDING_MIGRATION,
    SQLITE_ACCESS_MODE_MIGRATION,
    SQLITE_AGENT_IMPLEMENTATION_MIGRATION,
    SQLITE_OUTBOX_MIGRATION,
];

pub const POSTGRES_MIGRATIONS: &[&str] = &[
    POSTGRES_CORE_MIGRATION,
    POSTGRES_CONTEXT_BINDING_MIGRATION,
    POSTGRES_ACCESS_MODE_MIGRATION,
    POSTGRES_AGENT_IMPLEMENTATION_MIGRATION,
    POSTGRES_OUTBOX_MIGRATION,
];
