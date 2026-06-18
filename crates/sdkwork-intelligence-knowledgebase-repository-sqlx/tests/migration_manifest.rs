use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::{
    POSTGRES_ACCESS_MODE_MIGRATION, POSTGRES_AGENT_IMPLEMENTATION_MIGRATION,
    POSTGRES_CORE_MIGRATION, SQLITE_ACCESS_MODE_MIGRATION,
    SQLITE_AGENT_IMPLEMENTATION_MIGRATION, SQLITE_CORE_MIGRATION,
};
use std::collections::BTreeSet;

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
    "kb_wiki_page",
    "kb_wiki_page_revision",
    "kb_wiki_file_entry",
    "kb_wiki_schema_profile",
    "kb_wiki_log_entry",
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
    "uk_kb_wiki_page_uuid",
    "uk_kb_wiki_page_slug",
    "uk_kb_wiki_page_path",
    "idx_kb_wiki_page_state",
    "uk_kb_wiki_page_revision_uuid",
    "uk_kb_wiki_page_revision_no",
    "uk_kb_wiki_file_entry_uuid",
    "uk_kb_wiki_file_entry_path",
    "uk_kb_wiki_schema_profile_uuid",
    "uk_kb_wiki_log_entry_uuid",
    "uk_kb_local_mirror_package_uuid",
];

#[test]
fn core_migrations_include_required_knowledgebase_tables() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("description"));
        assert!(migration.contains("llm_wiki_initialized"));

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
            "uk_kb_wiki_page_slug",
            "uk_kb_wiki_page_revision_no",
            "uk_kb_wiki_file_entry_path",
            "uk_kb_wiki_log_entry_sequence",
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
    assert!(POSTGRES_CORE_MIGRATION.contains("wiki_log_sequence_counter BIGINT NOT NULL DEFAULT 0"));
    assert!(SQLITE_CORE_MIGRATION.contains("wiki_log_sequence_counter INTEGER NOT NULL DEFAULT 0"));
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

fn defined_database_objects(migration: &'static str, prefix: &str) -> BTreeSet<&'static str> {
    migration
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .filter_map(|tail| tail.split_whitespace().next())
        .map(|object_name| object_name.trim_matches('"'))
        .collect()
}
