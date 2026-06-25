//! SQL storage support for SDKWork Knowledgebase.

mod agent_profile_store;
mod audit_event_store;
mod binding_scope_filters;
mod browser_projection_store;
mod chunk_transaction;
pub mod db;
mod drive_object_ref_store;
mod embedding_store;
mod id;
mod index_store;
mod keyword_search;
pub mod mapper;
pub mod migrations;
mod okf_candidate_store;
mod okf_concept_link_store;
mod okf_concept_store;
pub mod repository;
mod retrieval_profile_store;
mod retrieval_store;
mod sqlite_chunk_store;
mod sqlite_commerce_store;
mod sqlite_context_binding_store;
mod sqlite_drive_import_metadata_store;
mod sqlite_import_stores;
mod sqlite_knowledge_document_metadata_transaction;
mod sqlite_markdown_index_metadata_store;
mod sqlite_okf_candidate_transaction;
mod sqlite_okf_concept_revision_metadata_store;
mod sqlite_okf_concept_transaction;
mod sqlite_outbox_store;
mod sqlite_space_stores;

pub mod pgvector_layered_retrieval;
mod postgres_pgvector_retrieval;

pub use agent_profile_store::SqliteKnowledgeAgentProfileStore;
pub use audit_event_store::{
    KnowledgeAuditEventRecord, KnowledgeAuditEventStore, KnowledgeAuditEventStoreError,
    SqliteKnowledgeAuditEventStore,
};
pub use browser_projection_store::SqliteKnowledgeBrowserProjectionStore;
pub use db::{
    connect_knowledgebase_and_install_schema, connect_postgres_and_install_schema,
    connect_postgres_pool, connect_postgres_via_framework_lifecycle,
    connect_sqlite_and_install_schema, connect_sqlite_pool, install_sqlite_core_schema,
    install_sqlite_schema, is_postgres_database_url, knowledgebase_health_check,
    postgres_health_check, sqlite_health_check, PostgresRepositoryError,
};
pub use drive_object_ref_store::SqliteKnowledgeDriveObjectRefStore;
pub use embedding_store::SqliteKnowledgeEmbeddingStore;
pub use id::{
    default_knowledge_id_generator, KnowledgeIdGenerator, KnowledgeIdGeneratorError,
    SnowflakeKnowledgeIdGenerator,
};
pub use index_store::{KnowledgeIndexStoreError, SqliteKnowledgeIndexStore};
pub use keyword_search::{keyword_search_backend_for_database_url, KeywordSearchBackend};
pub use okf_candidate_store::SqliteKnowledgeOkfCandidateStore;
pub use okf_concept_link_store::SqliteKnowledgeOkfConceptLinkStore;
pub use okf_concept_store::SqliteKnowledgeOkfConceptStore;
pub use pgvector_layered_retrieval::PgVectorLayeredRetrievalBackend;
pub use postgres_pgvector_retrieval::PgVectorKnowledgeRetrievalBackend;
pub use retrieval_profile_store::{
    KnowledgeRetrievalProfileStoreError, SqliteKnowledgeRetrievalProfileStore,
};
pub use retrieval_store::SqliteKnowledgeChunkRetrievalStore;
pub use sqlite_chunk_store::SqliteKnowledgeChunkStore;
pub use sqlite_commerce_store::SqliteCommerceStore;
pub use sqlite_context_binding_store::SqliteContextBindingStore;
pub use sqlite_drive_import_metadata_store::SqliteDriveImportMetadataStore;
pub use sqlite_import_stores::{
    SqliteIngestionJobStore, SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeSourceStore,
};
pub use sqlite_markdown_index_metadata_store::SqliteMarkdownIndexMetadataStore;
pub use sqlite_okf_concept_revision_metadata_store::SqliteOkfConceptRevisionMetadataStore;
pub use sqlite_outbox_store::SqliteKnowledgeOutboxStore;
pub use sqlite_space_stores::{SqliteKnowledgeOkfBundleFileStore, SqliteKnowledgeSpaceStore};
