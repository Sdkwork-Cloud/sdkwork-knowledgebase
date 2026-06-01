use sdkwork_knowledgebase_storage_sqlx::migrations::{
    POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION,
};

#[test]
fn core_migrations_include_required_knowledgebase_tables() {
    for migration in [POSTGRES_CORE_MIGRATION, SQLITE_CORE_MIGRATION] {
        assert!(migration.contains("knowledge_space"));
        assert!(migration.contains("description"));
        assert!(migration.contains("llm_wiki_initialized"));
        assert!(migration.contains("knowledge_collection"));
        assert!(migration.contains("knowledge_source"));
        assert!(migration.contains("knowledge_drive_object_ref"));
        assert!(migration.contains("knowledge_document"));
        assert!(migration.contains("knowledge_document_version"));
        assert!(migration.contains("knowledge_ingestion_job"));
        assert!(migration.contains("knowledge_ingestion_job_item"));
        assert!(migration.contains("knowledge_wiki_page"));
        assert!(migration.contains("knowledge_wiki_file_entry"));
        assert!(migration.contains("knowledge_wiki_schema_profile"));
        assert!(migration.contains("knowledge_wiki_log_entry"));
        assert!(migration.contains("knowledge_local_mirror_package"));
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
        assert!(migration.contains("idx_knowledge_drive_object_locator"));
        assert!(migration.contains("idx_knowledge_drive_object_role"));

        let lowercase = migration.to_ascii_lowercase();
        assert!(!lowercase.contains("presigned"));
        assert!(!lowercase.contains("credential"));
        assert!(!lowercase.contains("secret"));
    }
}
