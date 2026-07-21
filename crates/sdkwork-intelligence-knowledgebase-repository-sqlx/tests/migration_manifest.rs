use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
    POSTGRES_ACCESS_MODE_MIGRATION, POSTGRES_AGENT_IMPLEMENTATION_MIGRATION,
    POSTGRES_CONTEXT_BINDING_MIGRATION, POSTGRES_CORE_MIGRATION,
    POSTGRES_GROUP_KNOWLEDGE_SPACE_MIGRATION, POSTGRES_GROUP_MEMBERSHIP_PROJECTION_MIGRATION,
    POSTGRES_OUTBOX_MIGRATION, POSTGRES_PGVECTOR_MIGRATION, SQLITE_ACCESS_MODE_MIGRATION,
    SQLITE_AGENT_IMPLEMENTATION_MIGRATION, SQLITE_CONTEXT_BINDING_MIGRATION, SQLITE_CORE_MIGRATION,
    SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION, SQLITE_GROUP_MEMBERSHIP_PROJECTION_MIGRATION,
    SQLITE_OUTBOX_MIGRATION,
};
use sqlx::Row;
use std::collections::BTreeSet;

const APP_ROOT_POSTGRES_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/postgres/0001_knowledgebase_baseline.sql");
const APP_ROOT_SQLITE_BASELINE: &str =
    include_str!("../../../database/ddl/baseline/sqlite/0001_knowledgebase_baseline.sql");
const APP_ROOT_POSTGRES_GROUP_SPACE_MIGRATION: &str =
    include_str!("../../../database/migrations/postgres/202607150001_group_knowledge_space.up.sql");
const APP_ROOT_SQLITE_GROUP_SPACE_MIGRATION: &str =
    include_str!("../../../database/migrations/sqlite/202607150001_group_knowledge_space.up.sql");
const APP_ROOT_POSTGRES_TENANT_SCOPE_MIGRATION: &str = include_str!(
    "../../../database/migrations/postgres/202607160001_group_knowledgebase_tenant_scope.up.sql"
);
const APP_ROOT_SQLITE_TENANT_SCOPE_MIGRATION: &str = include_str!(
    "../../../database/migrations/sqlite/202607160001_group_knowledgebase_tenant_scope.up.sql"
);
const APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION: &str =
    include_str!("../../../database/migrations/postgres/202607210001_live_wiki_publication.up.sql");
const APP_ROOT_SQLITE_LIVE_WIKI_MIGRATION: &str =
    include_str!("../../../database/migrations/sqlite/202607210001_live_wiki_publication.up.sql");
const APP_ROOT_POSTGRES_LIVE_WIKI_ROLLBACK: &str = include_str!(
    "../../../database/migrations/postgres/202607210001_live_wiki_publication.down.sql"
);
const APP_ROOT_SQLITE_LIVE_WIKI_ROLLBACK: &str =
    include_str!("../../../database/migrations/sqlite/202607210001_live_wiki_publication.down.sql");
const APP_ROOT_DATABASE_MANIFEST: &str = include_str!("../../../database/database.manifest.json");
const APP_ROOT_DATABASE_SCHEMA: &str = include_str!("../../../database/contract/schema.yaml");
const APP_ROOT_DATABASE_TABLE_REGISTRY: &str =
    include_str!("../../../database/contract/table-registry.json");
const AGENT_PROFILE_STORE_SOURCE: &str = include_str!("../src/agent_profile_store.rs");
const AUDIT_EVENT_STORE_SOURCE: &str = include_str!("../src/audit_event_store.rs");
const DRIVE_OBJECT_REF_STORE_SOURCE: &str = include_str!("../src/drive_object_ref_store.rs");
const INDEX_STORE_SOURCE: &str = include_str!("../src/index_store.rs");
const OKF_CONCEPT_LINK_STORE_SOURCE: &str = include_str!("../src/okf_concept_link_store.rs");
const OKF_CONCEPT_STORE_SOURCE: &str = include_str!("../src/okf_concept_store.rs");
const RETRIEVAL_PROFILE_STORE_SOURCE: &str = include_str!("../src/retrieval_profile_store.rs");
const RETRIEVAL_STORE_SOURCE: &str = include_str!("../src/retrieval_store.rs");
const SQLITE_COMMERCE_STORE_SOURCE: &str = include_str!("../src/sqlite_commerce_store.rs");
const SQLITE_CONTEXT_BINDING_STORE_SOURCE: &str =
    include_str!("../src/sqlite_context_binding_store.rs");
const SQLITE_DRIVE_IMPORT_METADATA_STORE_SOURCE: &str =
    include_str!("../src/sqlite_drive_import_metadata_store.rs");
const SQLITE_IMPORT_STORES_SOURCE: &str = include_str!("../src/sqlite_import_stores.rs");
const SQLITE_KNOWLEDGE_DOCUMENT_METADATA_TRANSACTION_SOURCE: &str =
    include_str!("../src/sqlite_knowledge_document_metadata_transaction.rs");
const SQLITE_OKF_CANDIDATE_TRANSACTION_SOURCE: &str =
    include_str!("../src/sqlite_okf_candidate_transaction.rs");
const SQLITE_OKF_CONCEPT_REVISION_METADATA_STORE_SOURCE: &str =
    include_str!("../src/sqlite_okf_concept_revision_metadata_store.rs");
const SQLITE_OKF_CONCEPT_TRANSACTION_SOURCE: &str =
    include_str!("../src/sqlite_okf_concept_transaction.rs");
const SQLITE_OUTBOX_STORE_SOURCE: &str = include_str!("../src/sqlite_outbox_store.rs");
const SQLITE_SPACE_STORES_SOURCE: &str = include_str!("../src/sqlite_space_stores.rs");
const WIKI_CHECKPOINT_STORE_SOURCE: &str = include_str!("../src/wiki_persistence/checkpoint.rs");
const WIKI_INBOX_STORE_SOURCE: &str = include_str!("../src/wiki_persistence/inbox.rs");
const WIKI_PROJECTION_STORE_SOURCE: &str = include_str!("../src/wiki_persistence/projection.rs");
const WIKI_PUBLICATION_STORE_SOURCE: &str = include_str!("../src/wiki_persistence/publication.rs");
const WIKI_RENDITION_STORE_SOURCE: &str = include_str!("../src/wiki_persistence/rendition.rs");

const REQUIRED_CORE_TABLES: [&str; 22] = [
    "kb_space",
    "kb_collection",
    "kb_source",
    "kb_drive_object_ref",
    "kb_document",
    "kb_document_version",
    "kb_chunk",
    "kb_index",
    "kb_embedding",
    "kb_retrieval_profile",
    "kb_retrieval_trace",
    "kb_retrieval_hit",
    "kb_agent_profile",
    "kb_agent_knowledge_binding",
    "kb_ingestion_job",
    "kb_ingestion_job_item",
    "kb_okf_concept",
    "kb_okf_concept_revision",
    "kb_okf_bundle_file",
    "kb_okf_schema_profile",
    "kb_okf_log_entry",
    "kb_local_mirror_package",
];

const REQUIRED_CORE_INDEXES: [&str; 49] = [
    "uk_kb_space_uuid",
    "uk_kb_space_drive_space",
    "uk_kb_collection_uuid",
    "uk_kb_source_uuid",
    "uk_kb_source_identity",
    "uk_kb_drive_object_ref_uuid",
    "idx_kb_drive_object_locator",
    "uk_kb_drive_object_ref_locator",
    "idx_kb_drive_object_role",
    "idx_kb_drive_object_drive_node",
    "uk_kb_document_uuid",
    "idx_kb_document_drive_node",
    "uk_kb_document_identity",
    "uk_kb_document_version_uuid",
    "uk_kb_document_version_no",
    "uk_kb_chunk_uuid",
    "idx_kb_chunk_document_version",
    "idx_kb_chunk_space_status",
    "uk_kb_index_uuid",
    "idx_kb_index_scope",
    "uk_kb_embedding_uuid",
    "uk_kb_embedding_index_chunk",
    "idx_kb_embedding_chunk",
    "uk_kb_retrieval_profile_uuid",
    "idx_kb_retrieval_profile_tenant_status",
    "uk_kb_retrieval_trace_uuid",
    "idx_kb_retrieval_trace_profile_created",
    "idx_kb_retrieval_trace_actor_created",
    "uk_kb_retrieval_hit_uuid",
    "idx_kb_retrieval_hit_trace_rank",
    "idx_kb_retrieval_hit_chunk",
    "uk_kb_agent_profile_uuid",
    "idx_kb_agent_profile_model",
    "uk_kb_agent_knowledge_binding_uuid",
    "idx_kb_agent_knowledge_binding_profile",
    "uk_kb_ingestion_job_uuid",
    "uk_kb_ingestion_job_idempotency",
    "uk_kb_ingestion_job_item_uuid",
    "uk_kb_okf_concept_uuid",
    "uk_kb_okf_concept_id",
    "uk_kb_okf_concept_path",
    "idx_kb_okf_concept_state",
    "uk_kb_okf_concept_revision_uuid",
    "uk_kb_okf_concept_revision_no",
    "uk_kb_okf_bundle_file_uuid",
    "uk_kb_okf_bundle_file_path",
    "uk_kb_okf_schema_profile_uuid",
    "uk_kb_okf_log_entry_uuid",
    "uk_kb_local_mirror_package_uuid",
];

const LIVE_WIKI_TABLES: [&str; 5] = [
    "kb_site_publication",
    "kb_source_file_projection",
    "kb_source_file_rendition",
    "kb_drive_source_checkpoint",
    "kb_drive_event_inbox",
];

const LIVE_WIKI_INDEXES: [&str; 28] = [
    "uk_kb_site_publication_uuid",
    "uk_kb_site_publication_scope_id",
    "uk_kb_site_publication_space",
    "uk_kb_site_publication_drive_space",
    "idx_kb_site_publication_state",
    "uk_kb_source_projection_uuid",
    "uk_kb_source_projection_scope_id",
    "uk_kb_source_projection_node",
    "uk_kb_source_projection_path",
    "uk_kb_source_projection_public_route",
    "idx_kb_source_projection_state",
    "idx_kb_source_projection_claimable",
    "idx_kb_source_projection_scheduled",
    "idx_kb_source_projection_public_lookup",
    "uk_kb_source_rendition_uuid",
    "uk_kb_source_rendition_identity",
    "idx_kb_source_rendition_claimable",
    "idx_kb_source_rendition_source_version",
    "uk_kb_drive_checkpoint_uuid",
    "uk_kb_drive_checkpoint_scope_id",
    "uk_kb_drive_checkpoint_publication",
    "uk_kb_drive_checkpoint_source_scope",
    "idx_kb_drive_checkpoint_reconcile",
    "uk_kb_drive_inbox_uuid",
    "uk_kb_drive_inbox_event",
    "uk_kb_drive_inbox_sequence",
    "idx_kb_drive_inbox_apply",
    "idx_kb_drive_inbox_retry",
];

#[test]
fn core_migrations_include_required_knowledgebase_tables() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("description"));
        assert!(migration.contains("okf_bundle_initialized"));

        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        for table in REQUIRED_CORE_TABLES {
            assert!(tables.contains(table), "missing required table: {table}");
        }
    }
}

#[test]
fn core_migrations_use_kb_prefix_for_defined_database_objects() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        let indexes = defined_database_objects(migration, "CREATE INDEX IF NOT EXISTS ")
            .into_iter()
            .chain(defined_database_objects(
                migration,
                "CREATE UNIQUE INDEX IF NOT EXISTS ",
            ))
            .collect::<BTreeSet<_>>();

        for table in tables {
            assert!(
                table.starts_with("kb_"),
                "knowledgebase table must use kb_ prefix: {table}"
            );
        }

        for index in indexes {
            assert!(
                index.starts_with("idx_kb_") || index.starts_with("uk_kb_"),
                "knowledgebase index must use idx_kb_ or uk_kb_ prefix: {index}"
            );
        }

        assert!(!migration.contains(" ON knowledge_"));
        assert!(!migration.contains("uk_knowledge_"));
        assert!(!migration.contains("idx_knowledge_"));
    }
}

#[test]
fn drive_object_ref_migrations_store_stable_locator_metadata_not_delivery_secrets() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("drive_provider_kind"));
        assert!(migration.contains("drive_bucket"));
        assert!(migration.contains("drive_object_key"));
        assert!(migration.contains("drive_object_version"));
        assert!(migration.contains("drive_etag"));
        assert!(migration.contains("drive_metadata"));
        assert!(migration.contains("object_role"));
        assert!(migration.contains("access_mode"));
        assert!(migration.contains("idx_kb_drive_object_locator"));
        assert!(migration.contains("idx_kb_drive_object_role"));

        let lowercase = migration.to_ascii_lowercase();
        assert!(!lowercase.contains("presigned"));
        assert!(!lowercase.contains("credential"));
        assert!(!lowercase.contains("secret"));
    }
}

#[test]
fn core_migrations_define_identity_and_idempotency_uniques() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        let indexes = defined_database_objects(migration, "CREATE UNIQUE INDEX IF NOT EXISTS ");
        for index in [
            "uk_kb_space_uuid",
            "uk_kb_source_identity",
            "uk_kb_drive_object_ref_locator",
            "uk_kb_document_identity",
            "uk_kb_document_version_no",
            "uk_kb_ingestion_job_idempotency",
            "uk_kb_okf_concept_id",
            "uk_kb_okf_concept_revision_no",
            "uk_kb_okf_bundle_file_path",
            "uk_kb_okf_log_entry_sequence",
        ] {
            assert!(
                indexes.contains(index),
                "missing required unique index: {index}"
            );
        }
    }
}

#[test]
fn core_migrations_define_uuid_unique_indexes_for_all_uuid_tables() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        let indexes = defined_database_objects(migration, "CREATE UNIQUE INDEX IF NOT EXISTS ");
        for table in REQUIRED_CORE_TABLES {
            let index = format!("uk_{table}_uuid");
            assert!(
                indexes.contains(index.as_str()),
                "missing uuid unique index for {table}: {index}"
            );
        }
    }
}

#[test]
fn core_migrations_define_document_identity_scope_strategy() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("identity_scope"));
        assert!(migration.contains("source_only"));
        assert!(migration.contains("source_and_original_drive_node"));
        assert!(migration.contains("identity_scope,"));
        assert!(migration.contains("WHEN identity_scope = 'source_only' THEN ''"));
        assert!(migration.contains("ELSE COALESCE(original_file_drive_node_id, '')"));
    }
}

#[test]
fn core_migrations_harden_nullable_identity_columns() {
    assert!(POSTGRES_CORE_MIGRATION.contains("idempotency_key VARCHAR(128) NOT NULL"));
    assert!(SQLITE_CORE_MIGRATION.contains("idempotency_key TEXT NOT NULL"));
    assert!(POSTGRES_CORE_MIGRATION.contains("okf_log_sequence_counter BIGINT NOT NULL DEFAULT 0"));
    assert!(SQLITE_CORE_MIGRATION.contains("okf_log_sequence_counter INTEGER NOT NULL DEFAULT 0"));
    assert!(POSTGRES_CORE_MIGRATION.contains("revision_counter BIGINT NOT NULL DEFAULT 0"));
    assert!(SQLITE_CORE_MIGRATION.contains("revision_counter INTEGER NOT NULL DEFAULT 0"));

    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("COALESCE(drive_object_version"));
    }
}

#[test]
fn core_migrations_require_runtime_generated_snowflake_ids() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        let lowercase = migration.to_ascii_lowercase();
        assert!(!lowercase.contains("autoincrement"));
        assert!(!lowercase.contains(" bigserial"));
        assert!(!lowercase.contains(" serial"));
        assert!(!lowercase.contains("generated by default as identity"));
        assert!(!lowercase.contains("generated always as identity"));
    }

    assert!(!SQLITE_CORE_MIGRATION.contains("id INTEGER PRIMARY KEY"));

    for table in REQUIRED_CORE_TABLES {
        let declaration = format!("CREATE TABLE IF NOT EXISTS {table}");
        assert!(
            POSTGRES_CORE_MIGRATION.contains(&declaration),
            "missing postgres table declaration for {table}"
        );
        assert!(
            SQLITE_CORE_MIGRATION.contains(&declaration),
            "missing sqlite table declaration for {table}"
        );
    }
}

#[test]
fn core_migrations_define_all_required_indexes_with_kb_prefix() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        let indexes = defined_database_objects(migration, "CREATE INDEX IF NOT EXISTS ")
            .into_iter()
            .chain(defined_database_objects(
                migration,
                "CREATE UNIQUE INDEX IF NOT EXISTS ",
            ))
            .collect::<BTreeSet<_>>();

        for index in REQUIRED_CORE_INDEXES {
            assert!(indexes.contains(index), "missing required index: {index}");
        }
    }
}

#[test]
fn rag_migrations_define_retrieval_index_trace_and_agent_binding_columns() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        for snippet in [
            "CREATE TABLE IF NOT EXISTS kb_chunk",
            "document_version_id",
            "chunk_index",
            "content_text",
            "token_count",
            "locator",
            "CREATE TABLE IF NOT EXISTS kb_index",
            "index_kind",
            "embedding_provider_id",
            "embedding_model",
            "dimension",
            "metric",
            "CREATE TABLE IF NOT EXISTS kb_embedding",
            "vector_ref",
            "embedding_hash",
            "CREATE TABLE IF NOT EXISTS kb_retrieval_profile",
            "strategy",
            "rerank_enabled",
            "context_budget_tokens",
            "CREATE TABLE IF NOT EXISTS kb_retrieval_trace",
            "query_text_redacted",
            "latency_ms",
            "result_count",
            "CREATE TABLE IF NOT EXISTS kb_retrieval_hit",
            "retrieval_trace_id",
            "match_reason",
            "citation",
            "CREATE TABLE IF NOT EXISTS kb_agent_profile",
            "model_provider_id",
            "model_id",
            "system_instruction",
            "CREATE TABLE IF NOT EXISTS kb_agent_knowledge_binding",
            "profile_id",
            "space_id",
            "source_filter",
            "document_filter",
            "min_score",
        ] {
            assert!(
                migration.contains(snippet),
                "RAG migration must include snippet: {snippet}"
            );
        }

        let lowercase = migration.to_ascii_lowercase();
        assert!(!lowercase.contains("presigned"));
        assert!(!lowercase.contains("access_token"));
        assert!(!lowercase.contains("refresh_token"));
        assert!(!lowercase.contains("api_key"));
    }
}

#[test]
fn access_mode_migrations_add_profile_space_mode_and_vector_json() {
    for migration in [SQLITE_ACCESS_MODE_MIGRATION, POSTGRES_ACCESS_MODE_MIGRATION] {
        for snippet in [
            "knowledge_mode",
            "vector_json",
            "idx_kb_agent_profile_knowledge_mode",
            "idx_kb_space_knowledge_mode",
        ] {
            assert!(
                migration.contains(snippet),
                "access mode migration must include snippet: {snippet}"
            );
        }
    }
}

#[test]
fn agent_implementation_migrations_add_profile_runtime_selector() {
    for migration in [
        SQLITE_AGENT_IMPLEMENTATION_MIGRATION,
        POSTGRES_AGENT_IMPLEMENTATION_MIGRATION,
    ] {
        for snippet in [
            "agent_implementation_id",
            "plugin.intelligence.rig",
            "idx_kb_agent_profile_agent_implementation",
        ] {
            assert!(
                migration.contains(snippet),
                "agent implementation migration must include snippet: {snippet}"
            );
        }
    }
}

#[test]
fn context_binding_migrations_define_space_context_binding_table() {
    for migration in [
        SQLITE_CONTEXT_BINDING_MIGRATION,
        POSTGRES_CONTEXT_BINDING_MIGRATION,
    ] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        assert!(tables.contains("kb_space_context_binding"));
        let indexes = defined_database_objects(migration, "CREATE INDEX IF NOT EXISTS ")
            .into_iter()
            .chain(defined_database_objects(
                migration,
                "CREATE UNIQUE INDEX IF NOT EXISTS ",
            ))
            .collect::<BTreeSet<_>>();
        for index in [
            "uk_kb_space_context",
            "idx_kb_space_context_lookup",
            "idx_kb_space_context_space",
        ] {
            assert!(
                indexes.contains(index),
                "missing context binding index: {index}"
            );
        }
    }
}

#[test]
fn outbox_migrations_define_kb_outbox_event_table() {
    for migration in [SQLITE_OUTBOX_MIGRATION, POSTGRES_OUTBOX_MIGRATION] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        assert!(tables.contains("kb_outbox_event"));
        let indexes = defined_database_objects(migration, "CREATE INDEX IF NOT EXISTS ")
            .into_iter()
            .chain(defined_database_objects(
                migration,
                "CREATE UNIQUE INDEX IF NOT EXISTS ",
            ))
            .collect::<BTreeSet<_>>();
        for index in [
            "uk_kb_outbox_event_uuid",
            "idx_kb_outbox_event_status_created",
        ] {
            assert!(indexes.contains(index), "missing outbox index: {index}");
        }
    }
}

#[test]
fn postgres_pgvector_migration_defines_vector_embedding_column() {
    for snippet in [
        "CREATE EXTENSION IF NOT EXISTS vector",
        "embedding_vector vector(1536)",
        "idx_kb_embedding_vector_hnsw",
    ] {
        assert!(
            POSTGRES_PGVECTOR_MIGRATION.contains(snippet),
            "pgvector migration must include snippet: {snippet}"
        );
    }
}

#[test]
fn okf_migrations_define_link_and_candidate_tables() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_OKF_LINK_CANDIDATE_MIGRATION, SQLITE_OKF_LINK_CANDIDATE_MIGRATION,
    };

    for migration in [
        POSTGRES_OKF_LINK_CANDIDATE_MIGRATION,
        SQLITE_OKF_LINK_CANDIDATE_MIGRATION,
    ] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        assert!(tables.contains("kb_okf_concept_link"));
        assert!(tables.contains("kb_okf_candidate"));
        let indexes = defined_database_objects(migration, "CREATE INDEX IF NOT EXISTS ")
            .into_iter()
            .chain(defined_database_objects(
                migration,
                "CREATE UNIQUE INDEX IF NOT EXISTS ",
            ))
            .collect::<BTreeSet<_>>();
        for index in [
            "uk_kb_okf_concept_link_uuid",
            "uk_kb_okf_concept_link_edge",
            "idx_kb_okf_concept_link_space_from",
            "idx_kb_okf_concept_link_space_to",
            "uk_kb_okf_candidate_uuid",
            "idx_kb_okf_candidate_space_state",
        ] {
            assert!(
                indexes.contains(index),
                "missing okf migration index: {index}"
            );
        }
    }
}

#[test]
fn outbox_delivery_migrations_add_retry_metadata_columns() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_OUTBOX_DELIVERY_MIGRATION, SQLITE_OUTBOX_DELIVERY_MIGRATION,
    };

    for migration in [
        SQLITE_OUTBOX_DELIVERY_MIGRATION,
        POSTGRES_OUTBOX_DELIVERY_MIGRATION,
    ] {
        for snippet in ["last_error", "retry_count", "kb_outbox_event"] {
            assert!(
                migration.contains(snippet),
                "outbox delivery migration must include snippet: {snippet}"
            );
        }
    }
}

#[test]
fn chunk_fts_migrations_define_keyword_search_primitives() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_CHUNK_FTS_MIGRATION, SQLITE_CHUNK_FTS_MIGRATION,
    };

    assert!(SQLITE_CHUNK_FTS_MIGRATION.contains("kb_chunk_fts"));
    assert!(SQLITE_CHUNK_FTS_MIGRATION.contains("fts5"));
    assert!(POSTGRES_CHUNK_FTS_MIGRATION.contains("search_vector"));
    assert!(POSTGRES_CHUNK_FTS_MIGRATION.contains("idx_kb_chunk_search_vector"));
}

#[test]
fn performance_index_migrations_target_outbox_event_table() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_PERFORMANCE_INDEXES_MIGRATION, SQLITE_PERFORMANCE_INDEXES_MIGRATION,
    };

    for migration in [
        SQLITE_PERFORMANCE_INDEXES_MIGRATION,
        POSTGRES_PERFORMANCE_INDEXES_MIGRATION,
    ] {
        assert!(migration.contains("idx_kb_ingestion_job_tenant_state_status"));
        assert!(migration.contains("idx_kb_outbox_stale_claim"));
        assert!(migration.contains("kb_outbox_event"));
        assert!(!migration.contains(" ON kb_outbox "));
    }
}

#[test]
fn market_migrations_define_market_tables() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_MARKET_MIGRATION, SQLITE_MARKET_MIGRATION,
    };

    for migration in [SQLITE_MARKET_MIGRATION, POSTGRES_MARKET_MIGRATION] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        for table in ["kb_market_listing", "kb_market_subscription"] {
            assert!(tables.contains(table), "missing market table: {table}");
        }
        assert!(!migration.contains("site_deployment"));
    }
}

#[test]
fn runtime_sql_value_bindings_are_generated_by_database_dialect() {
    for (file, source) in [
        ("agent_profile_store.rs", AGENT_PROFILE_STORE_SOURCE),
        ("audit_event_store.rs", AUDIT_EVENT_STORE_SOURCE),
        ("drive_object_ref_store.rs", DRIVE_OBJECT_REF_STORE_SOURCE),
        ("index_store.rs", INDEX_STORE_SOURCE),
        ("okf_concept_link_store.rs", OKF_CONCEPT_LINK_STORE_SOURCE),
        ("okf_concept_store.rs", OKF_CONCEPT_STORE_SOURCE),
        ("retrieval_profile_store.rs", RETRIEVAL_PROFILE_STORE_SOURCE),
        ("retrieval_store.rs", RETRIEVAL_STORE_SOURCE),
        ("sqlite_commerce_store.rs", SQLITE_COMMERCE_STORE_SOURCE),
        (
            "sqlite_context_binding_store.rs",
            SQLITE_CONTEXT_BINDING_STORE_SOURCE,
        ),
        (
            "sqlite_drive_import_metadata_store.rs",
            SQLITE_DRIVE_IMPORT_METADATA_STORE_SOURCE,
        ),
        ("sqlite_import_stores.rs", SQLITE_IMPORT_STORES_SOURCE),
        (
            "sqlite_knowledge_document_metadata_transaction.rs",
            SQLITE_KNOWLEDGE_DOCUMENT_METADATA_TRANSACTION_SOURCE,
        ),
        (
            "sqlite_okf_candidate_transaction.rs",
            SQLITE_OKF_CANDIDATE_TRANSACTION_SOURCE,
        ),
        (
            "sqlite_okf_concept_revision_metadata_store.rs",
            SQLITE_OKF_CONCEPT_REVISION_METADATA_STORE_SOURCE,
        ),
        (
            "sqlite_okf_concept_transaction.rs",
            SQLITE_OKF_CONCEPT_TRANSACTION_SOURCE,
        ),
        ("sqlite_outbox_store.rs", SQLITE_OUTBOX_STORE_SOURCE),
        ("sqlite_space_stores.rs", SQLITE_SPACE_STORES_SOURCE),
        (
            "wiki_persistence/checkpoint.rs",
            WIKI_CHECKPOINT_STORE_SOURCE,
        ),
        ("wiki_persistence/inbox.rs", WIKI_INBOX_STORE_SOURCE),
        (
            "wiki_persistence/projection.rs",
            WIKI_PROJECTION_STORE_SOURCE,
        ),
        (
            "wiki_persistence/publication.rs",
            WIKI_PUBLICATION_STORE_SOURCE,
        ),
        ("wiki_persistence/rendition.rs", WIKI_RENDITION_STORE_SOURCE),
    ] {
        assert!(
            !source.contains("AS TIMESTAMP)"),
            "{file} must use SqlTimestampDialect::sql_timestamp_expr instead of hard-coded PostgreSQL timestamp casts"
        );
        assert!(
            !source.contains("AS JSONB)"),
            "{file} must use SqlTimestampDialect::sql_json_expr instead of hard-coded PostgreSQL JSONB casts"
        );
    }

    assert!(
        [
            AGENT_PROFILE_STORE_SOURCE,
            AUDIT_EVENT_STORE_SOURCE,
            DRIVE_OBJECT_REF_STORE_SOURCE,
            INDEX_STORE_SOURCE,
            OKF_CONCEPT_LINK_STORE_SOURCE,
            OKF_CONCEPT_STORE_SOURCE,
            RETRIEVAL_PROFILE_STORE_SOURCE,
            RETRIEVAL_STORE_SOURCE,
            SQLITE_COMMERCE_STORE_SOURCE,
            SQLITE_CONTEXT_BINDING_STORE_SOURCE,
            SQLITE_DRIVE_IMPORT_METADATA_STORE_SOURCE,
            SQLITE_IMPORT_STORES_SOURCE,
            SQLITE_KNOWLEDGE_DOCUMENT_METADATA_TRANSACTION_SOURCE,
            SQLITE_OKF_CANDIDATE_TRANSACTION_SOURCE,
            SQLITE_OKF_CONCEPT_REVISION_METADATA_STORE_SOURCE,
            SQLITE_OKF_CONCEPT_TRANSACTION_SOURCE,
            SQLITE_OUTBOX_STORE_SOURCE,
            SQLITE_SPACE_STORES_SOURCE,
            WIKI_CHECKPOINT_STORE_SOURCE,
            WIKI_INBOX_STORE_SOURCE,
            WIKI_PROJECTION_STORE_SOURCE,
            WIKI_PUBLICATION_STORE_SOURCE,
            WIKI_RENDITION_STORE_SOURCE,
        ]
        .iter()
        .any(|source| source.contains("sql_timestamp_expr")),
        "runtime repositories must generate PostgreSQL timestamp casts through SqlTimestampDialect"
    );
    assert!(
        [
            AGENT_PROFILE_STORE_SOURCE,
            AUDIT_EVENT_STORE_SOURCE,
            OKF_CONCEPT_STORE_SOURCE,
            RETRIEVAL_STORE_SOURCE,
            SQLITE_DRIVE_IMPORT_METADATA_STORE_SOURCE,
            SQLITE_IMPORT_STORES_SOURCE,
            SQLITE_KNOWLEDGE_DOCUMENT_METADATA_TRANSACTION_SOURCE,
            SQLITE_OKF_CONCEPT_TRANSACTION_SOURCE,
            SQLITE_OUTBOX_STORE_SOURCE,
        ]
        .iter()
        .any(|source| source.contains("sql_json_expr")),
        "runtime repositories must generate PostgreSQL JSONB casts through SqlTimestampDialect"
    );
    for (file, source, projection) in [
        (
            "sqlite_import_stores.rs",
            SQLITE_IMPORT_STORES_SOURCE,
            "CAST(metadata AS TEXT) AS metadata",
        ),
        (
            "sqlite_knowledge_document_metadata_transaction.rs",
            SQLITE_KNOWLEDGE_DOCUMENT_METADATA_TRANSACTION_SOURCE,
            "CAST(metadata AS TEXT) AS metadata",
        ),
        (
            "sqlite_drive_import_metadata_store.rs",
            SQLITE_DRIVE_IMPORT_METADATA_STORE_SOURCE,
            "CAST(metadata AS TEXT) AS metadata",
        ),
        (
            "sqlite_okf_concept_transaction.rs",
            SQLITE_OKF_CONCEPT_TRANSACTION_SOURCE,
            "CAST(tags AS TEXT) AS tags",
        ),
        (
            "sqlite_okf_concept_revision_metadata_store.rs",
            SQLITE_OKF_CONCEPT_REVISION_METADATA_STORE_SOURCE,
            "CAST(tags AS TEXT) AS tags",
        ),
        (
            "okf_concept_store.rs",
            OKF_CONCEPT_STORE_SOURCE,
            "CAST(tags AS TEXT) AS tags",
        ),
        (
            "okf_concept_store.rs",
            OKF_CONCEPT_STORE_SOURCE,
            "CAST(metadata AS TEXT) AS metadata",
        ),
        (
            "retrieval_store.rs",
            RETRIEVAL_STORE_SOURCE,
            "CAST(h.citation AS TEXT) AS citation",
        ),
        (
            "sqlite_outbox_store.rs",
            SQLITE_OUTBOX_STORE_SOURCE,
            "CAST(payload AS TEXT) AS payload",
        ),
    ] {
        assert!(
            source.contains(projection),
            "{file} must project PostgreSQL JSONB values as text before decoding them as Rust String"
        );
    }
}

#[test]
fn audit_event_migrations_define_kb_audit_event_table() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_AUDIT_EVENT_MIGRATION, SQLITE_AUDIT_EVENT_MIGRATION,
    };

    for migration in [SQLITE_AUDIT_EVENT_MIGRATION, POSTGRES_AUDIT_EVENT_MIGRATION] {
        let tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        assert!(tables.contains("kb_audit_event"));
        assert!(migration.contains("idx_kb_audit_event_tenant_created"));
        assert!(migration.contains("idx_kb_audit_event_event_type"));
    }
}

#[test]
fn outbox_claim_migrations_add_claimed_at_column() {
    use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
        POSTGRES_OUTBOX_CLAIM_MIGRATION, SQLITE_OUTBOX_CLAIM_MIGRATION,
    };

    for migration in [
        SQLITE_OUTBOX_CLAIM_MIGRATION,
        POSTGRES_OUTBOX_CLAIM_MIGRATION,
    ] {
        assert!(migration.contains("claimed_at"));
        assert!(migration.contains("kb_outbox_event"));
    }
}

#[test]
fn app_root_database_baselines_are_engine_specific_single_snapshots() {
    for (needle, expected_count) in [
        ("CREATE TABLE IF NOT EXISTS kb_market_listing", 1),
        ("CREATE TABLE IF NOT EXISTS kb_market_subscription", 1),
        (
            "ALTER TABLE kb_outbox_event ADD COLUMN IF NOT EXISTS claimed_at",
            1,
        ),
        ("CREATE INDEX IF NOT EXISTS idx_kb_outbox_stale_claim", 1),
    ] {
        assert_eq!(
            count_occurrences(APP_ROOT_POSTGRES_BASELINE, needle),
            expected_count,
            "postgres baseline must contain {needle} exactly {expected_count} time(s)"
        );
    }

    assert!(APP_ROOT_POSTGRES_BASELINE.contains("expires_at BIGINT"));
    assert!(APP_ROOT_POSTGRES_BASELINE.contains("idx_web_audit_expires"));
    assert!(APP_ROOT_SQLITE_BASELINE.contains("expires_at INTEGER"));
    assert!(APP_ROOT_SQLITE_BASELINE.contains("idx_web_audit_expires"));
    for obsolete_table in ["kb_site", "kb_site_release", "kb_site_host_binding"] {
        let table_declaration = format!("CREATE TABLE IF NOT EXISTS {obsolete_table} (");
        assert!(!APP_ROOT_POSTGRES_BASELINE.contains(&table_declaration));
        assert!(!APP_ROOT_SQLITE_BASELINE.contains(&table_declaration));
    }

    for forbidden in [
        "ADD COLUMN IF NOT EXISTS",
        "USING GIN",
        "to_tsvector",
        "JSONB",
        "DOUBLE PRECISION",
    ] {
        assert!(
            !APP_ROOT_SQLITE_BASELINE.contains(forbidden),
            "sqlite baseline must not contain postgres-only syntax: {forbidden}"
        );
    }
}

#[test]
fn group_aggregate_baseline_preserves_postgres_rls_for_every_tenant_table() {
    let rls_section_start = APP_ROOT_POSTGRES_BASELINE
        .find("FOR table_name IN")
        .expect("postgres baseline must define the group aggregate RLS loop");
    let rls_section = &APP_ROOT_POSTGRES_BASELINE[rls_section_start..];

    for table in [
        "kb_group_knowledge_space_binding",
        "kb_group_knowledge_space_member",
        "kb_group_knowledge_space_event_inbox",
        "kb_group_knowledge_space_membership_projection",
    ] {
        assert!(
            rls_section.contains(&format!("'{table}'")),
            "postgres baseline RLS loop must include {table}"
        );
    }
    for required_statement in [
        "ALTER TABLE %I ENABLE ROW LEVEL SECURITY",
        "ALTER TABLE %I FORCE ROW LEVEL SECURITY",
        "CREATE POLICY tenant_isolation ON %I",
        "current_setting(''app.current_tenant_id'', true)::bigint",
    ] {
        assert!(
            rls_section.contains(required_statement),
            "postgres baseline RLS loop is missing {required_statement}"
        );
    }

    for table in [
        "kb_group_knowledge_space_binding",
        "kb_group_knowledge_space_member",
        "kb_group_knowledge_space_event_inbox",
    ] {
        assert!(
            POSTGRES_GROUP_KNOWLEDGE_SPACE_MIGRATION.contains(&format!("'{table}'")),
            "group aggregate migration must protect {table}"
        );
    }
    for required_statement in [
        "ALTER TABLE kb_group_knowledge_space_membership_projection ENABLE ROW LEVEL SECURITY",
        "ALTER TABLE kb_group_knowledge_space_membership_projection FORCE ROW LEVEL SECURITY",
        "CREATE POLICY tenant_isolation",
        "current_setting('app.current_tenant_id', true)::bigint",
    ] {
        assert!(
            POSTGRES_GROUP_MEMBERSHIP_PROJECTION_MIGRATION.contains(required_statement),
            "membership projection migration is missing {required_statement}"
        );
    }
}

#[test]
fn group_aggregate_accepts_the_canonical_tenant_scope_in_every_schema_path() {
    for (name, source, declaration, constraint) in [
        (
            "postgres baseline binding",
            APP_ROOT_POSTGRES_BASELINE,
            "organization_id BIGINT NOT NULL,",
            "ck_kb_group_knowledge_space_binding_organization",
        ),
        (
            "postgres migration binding",
            POSTGRES_GROUP_KNOWLEDGE_SPACE_MIGRATION,
            "organization_id BIGINT NOT NULL,",
            "ck_kb_group_knowledge_space_binding_organization",
        ),
        (
            "postgres baseline projection",
            APP_ROOT_POSTGRES_BASELINE,
            "organization_id BIGINT NOT NULL,",
            "ck_kb_group_knowledge_space_membership_projection_organization",
        ),
        (
            "postgres migration projection",
            POSTGRES_GROUP_MEMBERSHIP_PROJECTION_MIGRATION,
            "organization_id BIGINT NOT NULL,",
            "ck_kb_group_knowledge_space_membership_projection_organization",
        ),
    ] {
        assert!(
            source.contains(declaration),
            "{name} must not default organization_id to zero"
        );
        assert!(
            source.contains(constraint),
            "{name} is missing its named organization constraint"
        );
    }

    for (name, source) in [
        ("sqlite baseline", APP_ROOT_SQLITE_BASELINE),
        (
            "sqlite aggregate migration",
            SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION,
        ),
        (
            "sqlite membership projection migration",
            SQLITE_GROUP_MEMBERSHIP_PROJECTION_MIGRATION,
        ),
    ] {
        let group_section = if name == "sqlite baseline" {
            let start = source
                .find("CREATE TABLE IF NOT EXISTS kb_group_knowledge_space_binding")
                .expect("sqlite baseline must contain group aggregate tables");
            &source[start..]
        } else {
            source
        };
        assert!(
            !group_section.contains("organization_id INTEGER NOT NULL DEFAULT 0"),
            "{name} must not default a group organization_id to zero"
        );
        assert!(
            group_section.contains("CHECK (organization_id >= 0)"),
            "{name} must accept the zero tenant-scope organization sentinel in greenfield DDL"
        );
    }

    for table in ["binding", "member", "event_inbox", "membership_projection"] {
        let migration = if table == "membership_projection" {
            SQLITE_GROUP_MEMBERSHIP_PROJECTION_MIGRATION
        } else {
            SQLITE_GROUP_KNOWLEDGE_SPACE_MIGRATION
        };
        assert!(
            migration.contains(&format!("trg_kb_group_space_{table}_organization_insert")),
            "SQLite upgrade migration is missing the {table} organization insert guard"
        );
        assert!(
            migration.contains(&format!("trg_kb_group_space_{table}_organization_update")),
            "SQLite upgrade migration is missing the {table} organization update guard"
        );
    }

    for table in [
        "kb_group_knowledge_space_binding",
        "kb_group_knowledge_space_member",
        "kb_group_knowledge_space_event_inbox",
        "kb_group_knowledge_space_membership_projection",
    ] {
        assert!(
            APP_ROOT_POSTGRES_BASELINE.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "postgres baseline is missing {table}"
        );
        assert!(
            APP_ROOT_SQLITE_BASELINE.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "sqlite baseline is missing {table}"
        );
    }
}

#[test]
fn group_tenant_scope_upgrade_migrations_match_the_greenfield_contract() {
    assert_eq!(
        APP_ROOT_POSTGRES_TENANT_SCOPE_MIGRATION
            .matches("CHECK (organization_id >= 0)")
            .count(),
        4,
        "PostgreSQL upgrade must relax all four group aggregate scope constraints",
    );
    assert!(APP_ROOT_SQLITE_TENANT_SCOPE_MIGRATION.contains("CHECK (organization_id >= 0)"));
    for table in [
        "kb_group_knowledge_space_binding",
        "kb_group_knowledge_space_member",
        "kb_group_knowledge_space_event_inbox",
        "kb_group_knowledge_space_membership_projection",
    ] {
        assert!(
            APP_ROOT_SQLITE_TENANT_SCOPE_MIGRATION.contains(&format!("CREATE TABLE {table}")),
            "SQLite upgrade must rebuild {table} with the tenant-scope contract",
        );
    }
}

#[test]
fn runtime_group_space_expand_migrations_precede_dependent_scope_corrections() {
    assert!("202607150001" < "202607160001");

    for (engine, migration) in [
        ("postgres", APP_ROOT_POSTGRES_GROUP_SPACE_MIGRATION),
        ("sqlite", APP_ROOT_SQLITE_GROUP_SPACE_MIGRATION),
    ] {
        for table in [
            "kb_group_knowledge_space_binding",
            "kb_group_knowledge_space_member",
            "kb_group_knowledge_space_event_inbox",
            "kb_group_knowledge_space_membership_projection",
        ] {
            assert!(
                migration.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
                "{engine} runtime expand migration must create {table} before dependent corrections",
            );
        }
    }

    assert!(APP_ROOT_POSTGRES_GROUP_SPACE_MIGRATION
        .contains("ALTER TABLE %I ENABLE ROW LEVEL SECURITY"));
    assert!(
        APP_ROOT_POSTGRES_GROUP_SPACE_MIGRATION.contains("ALTER TABLE %I FORCE ROW LEVEL SECURITY")
    );
    assert!(APP_ROOT_POSTGRES_GROUP_SPACE_MIGRATION.contains("CREATE POLICY tenant_isolation"));
}

#[test]
fn live_wiki_migrations_and_greenfield_baselines_define_the_same_objects() {
    for (engine, baseline, migration) in [
        (
            "postgres",
            live_wiki_baseline_section(APP_ROOT_POSTGRES_BASELINE),
            APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION,
        ),
        (
            "sqlite",
            live_wiki_baseline_section(APP_ROOT_SQLITE_BASELINE),
            APP_ROOT_SQLITE_LIVE_WIKI_MIGRATION,
        ),
    ] {
        let baseline_tables = defined_database_objects(baseline, "CREATE TABLE IF NOT EXISTS ");
        let migration_tables = defined_database_objects(migration, "CREATE TABLE IF NOT EXISTS ");
        assert_eq!(baseline_tables, migration_tables, "{engine} table drift");

        let baseline_indexes = defined_indexes(baseline);
        let migration_indexes = defined_indexes(migration);
        assert_eq!(baseline_indexes, migration_indexes, "{engine} index drift");

        for table in LIVE_WIKI_TABLES {
            assert!(
                baseline_tables.contains(table),
                "{engine} live Wiki schema is missing {table}"
            );
        }
        for index in LIVE_WIKI_INDEXES {
            assert!(
                baseline_indexes.contains(index),
                "{engine} live Wiki schema is missing {index}"
            );
        }
    }
}

#[test]
fn live_wiki_postgres_tables_are_forced_behind_tenant_rls() {
    for table in LIVE_WIKI_TABLES {
        let table_literal = format!("'{table}'");
        assert!(
            APP_ROOT_POSTGRES_BASELINE.contains(&table_literal),
            "postgres baseline RLS inventory is missing {table}"
        );
        assert!(
            APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION.contains(&table_literal),
            "postgres migration RLS inventory is missing {table}"
        );
    }

    for source in [
        APP_ROOT_POSTGRES_BASELINE,
        APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION,
    ] {
        assert!(source.contains("ENABLE ROW LEVEL SECURITY"));
        assert!(source.contains("FORCE ROW LEVEL SECURITY"));
        assert!(source.contains("CREATE POLICY tenant_isolation"));
        assert!(source.contains("app.current_tenant_id"));
    }
}

#[test]
fn live_wiki_persistence_keeps_release_and_storage_locator_state_out_of_knowledgebase() {
    for (engine, source) in [
        ("postgres", APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION),
        ("sqlite", APP_ROOT_SQLITE_LIVE_WIKI_MIGRATION),
    ] {
        let lowercase = source.to_ascii_lowercase();
        for forbidden in [
            "kb_site_release",
            "kb_site_revision",
            "kb_deployment",
            "object_key",
            "signed_url",
            "presigned_url",
            "storage_bucket",
        ] {
            assert!(
                !lowercase.contains(forbidden),
                "{engine} live Wiki schema must not own {forbidden}"
            );
        }
        assert!(source.contains("rendition_key_sha256"));
        assert!(source.contains("drive_version_uuid"));
        assert!(source.contains("source_sequence_no"));
        assert!(source.contains("publication_state <> 'PUBLISHED'\n        OR (visibility"));
        assert!(!source.contains("source_state = 'READY' AND visibility"));
    }
}

#[test]
fn live_wiki_migrations_are_paired_and_registered_at_contract_version_1_1_0() {
    for migration in [
        APP_ROOT_POSTGRES_LIVE_WIKI_MIGRATION,
        APP_ROOT_SQLITE_LIVE_WIKI_MIGRATION,
        APP_ROOT_POSTGRES_LIVE_WIKI_ROLLBACK,
        APP_ROOT_SQLITE_LIVE_WIKI_ROLLBACK,
    ] {
        assert!(migration.contains("contract_version: 1.1.0"));
    }
    for rollback in [
        APP_ROOT_POSTGRES_LIVE_WIKI_ROLLBACK,
        APP_ROOT_SQLITE_LIVE_WIKI_ROLLBACK,
    ] {
        for table in LIVE_WIKI_TABLES {
            assert!(
                rollback.contains(&format!("DROP TABLE IF EXISTS {table}")),
                "rollback is missing {table}"
            );
        }
    }

    assert!(APP_ROOT_DATABASE_MANIFEST.contains("\"contractVersion\": \"1.1.0\""));
    assert!(APP_ROOT_DATABASE_SCHEMA.contains("contract_version: 1.1.0"));
    for table in LIVE_WIKI_TABLES {
        assert!(
            APP_ROOT_DATABASE_SCHEMA.contains(&format!("name: {table}")),
            "schema contract is missing {table}"
        );
        assert!(
            APP_ROOT_DATABASE_TABLE_REGISTRY.contains(&format!("\"table_name\": \"{table}\"")),
            "table registry is missing {table}"
        );
    }
}

#[tokio::test]
async fn live_wiki_sqlite_greenfield_baseline_bootstraps_with_foreign_keys_and_indexes() {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_pool("sqlite::memory:")
            .await
            .expect("connect empty SQLite database");
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .expect("enable SQLite foreign keys");
    sqlx::raw_sql(APP_ROOT_SQLITE_BASELINE)
        .execute(&pool)
        .await
        .expect("apply application-root SQLite baseline");

    let foreign_key_violations = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(&pool)
        .await
        .expect("check SQLite foreign keys");
    assert!(
        foreign_key_violations.is_empty(),
        "greenfield baseline contains foreign key violations"
    );

    for object_name in LIVE_WIKI_TABLES.into_iter().chain(LIVE_WIKI_INDEXES) {
        let row = sqlx::query("SELECT COUNT(*) AS object_count FROM sqlite_master WHERE name = $1")
            .bind(object_name)
            .fetch_one(&pool)
            .await
            .expect("query SQLite schema object");
        let object_count: i64 = row
            .try_get("object_count")
            .expect("decode schema object count");
        assert_eq!(object_count, 1, "SQLite baseline is missing {object_name}");
    }
}

fn live_wiki_baseline_section(source: &'static str) -> &'static str {
    let start = source
        .find("-- Canonical live Wiki publication authority (ADR-20260721).")
        .expect("baseline must contain the live Wiki section");
    let end = source[start..]
        .find("-- Provider Binding SPI v2 authority (ADR-20260720).")
        .map(|offset| start + offset)
        .expect("baseline must end the live Wiki section before provider bindings");
    &source[start..end]
}

fn defined_indexes(source: &'static str) -> BTreeSet<&'static str> {
    defined_database_objects(source, "CREATE INDEX IF NOT EXISTS ")
        .into_iter()
        .chain(defined_database_objects(
            source,
            "CREATE UNIQUE INDEX IF NOT EXISTS ",
        ))
        .collect()
}

fn defined_database_objects(migration: &'static str, prefix: &str) -> BTreeSet<&'static str> {
    migration
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .filter_map(|tail| tail.split_whitespace().next())
        .map(|object_name| object_name.trim_matches('"'))
        .collect()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}
