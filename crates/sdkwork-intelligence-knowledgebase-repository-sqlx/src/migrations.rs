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

pub const POSTGRES_OKF_LINK_CANDIDATE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606210001__okf_link_and_candidate.sql");

pub const POSTGRES_OUTBOX_DELIVERY_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606220001__knowledgebase_outbox_delivery.sql");

pub const POSTGRES_OUTBOX_CLAIM_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606220003__knowledgebase_outbox_claim.sql");

pub const POSTGRES_CHUNK_FTS_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606220002__knowledgebase_chunk_fts.sql");

pub const POSTGRES_PERFORMANCE_INDEXES_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606230001__knowledgebase_performance_indexes.sql");

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

pub const SQLITE_OKF_LINK_CANDIDATE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606210001__okf_link_and_candidate.sql");

pub const SQLITE_OUTBOX_DELIVERY_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606220001__knowledgebase_outbox_delivery.sql");

pub const SQLITE_OUTBOX_CLAIM_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606220003__knowledgebase_outbox_claim.sql");

pub const SQLITE_CHUNK_FTS_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606220002__knowledgebase_chunk_fts.sql");

pub const SQLITE_PERFORMANCE_INDEXES_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606230001__knowledgebase_performance_indexes.sql");

pub const SQLITE_MARKET_SITE_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606240001__knowledge_market_and_site_deployment.sql");

pub const POSTGRES_MARKET_SITE_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606240001__knowledge_market_and_site_deployment.sql");

pub const SQLITE_AUDIT_EVENT_MIGRATION: &str =
    include_str!("../migrations/sqlite/V202606250001__knowledgebase_audit_event.sql");

pub const POSTGRES_AUDIT_EVENT_MIGRATION: &str =
    include_str!("../migrations/postgres/V202606250001__knowledgebase_audit_event.sql");

pub const SQLITE_MIGRATIONS: &[&str] = &[
    SQLITE_CORE_MIGRATION,
    SQLITE_CONTEXT_BINDING_MIGRATION,
    SQLITE_ACCESS_MODE_MIGRATION,
    SQLITE_AGENT_IMPLEMENTATION_MIGRATION,
    SQLITE_OUTBOX_MIGRATION,
    SQLITE_OKF_LINK_CANDIDATE_MIGRATION,
    SQLITE_OUTBOX_DELIVERY_MIGRATION,
    SQLITE_CHUNK_FTS_MIGRATION,
    SQLITE_OUTBOX_CLAIM_MIGRATION,
    SQLITE_PERFORMANCE_INDEXES_MIGRATION,
    SQLITE_MARKET_SITE_MIGRATION,
    SQLITE_AUDIT_EVENT_MIGRATION,
];

pub const POSTGRES_MIGRATIONS: &[&str] = &[
    POSTGRES_CORE_MIGRATION,
    POSTGRES_CONTEXT_BINDING_MIGRATION,
    POSTGRES_ACCESS_MODE_MIGRATION,
    POSTGRES_AGENT_IMPLEMENTATION_MIGRATION,
    POSTGRES_PGVECTOR_MIGRATION,
    POSTGRES_OUTBOX_MIGRATION,
    POSTGRES_OKF_LINK_CANDIDATE_MIGRATION,
    POSTGRES_OUTBOX_DELIVERY_MIGRATION,
    POSTGRES_CHUNK_FTS_MIGRATION,
    POSTGRES_OUTBOX_CLAIM_MIGRATION,
    POSTGRES_PERFORMANCE_INDEXES_MIGRATION,
    POSTGRES_MARKET_SITE_MIGRATION,
    POSTGRES_AUDIT_EVENT_MIGRATION,
];

// Legacy migration SQL retained for contract tests only. Runtime PostgreSQL bootstrap uses
// application-root `database/` via `sdkwork-knowledgebase-database-host`.
