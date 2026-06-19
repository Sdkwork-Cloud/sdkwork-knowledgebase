//! SQL storage support for SDKWork Knowledgebase.

mod agent_profile_store;
mod browser_projection_store;
pub mod db;
mod drive_object_ref_store;
mod embedding_store;
mod id;
mod index_store;
pub mod mapper;
pub mod migrations;
pub mod repository;
mod retrieval_profile_store;
mod retrieval_store;
mod sqlite_chunk_store;
mod sqlite_context_binding_store;
mod sqlite_import_stores;
mod sqlite_outbox_store;
mod sqlite_space_stores;
mod wiki_page_store;

pub mod postgres_pgvector_retrieval;

pub use agent_profile_store::SqliteKnowledgeAgentProfileStore;
pub use browser_projection_store::SqliteKnowledgeBrowserProjectionStore;
pub use db::{
    connect_knowledgebase_and_install_schema, connect_postgres_and_install_schema,
    connect_postgres_pool, connect_sqlite_and_install_schema, connect_sqlite_pool,
    install_postgres_schema, install_sqlite_core_schema, install_sqlite_schema,
    is_postgres_database_url, knowledgebase_health_check, postgres_health_check,
    sqlite_health_check, PostgresRepositoryError,
};
pub use drive_object_ref_store::SqliteKnowledgeDriveObjectRefStore;
pub use embedding_store::SqliteKnowledgeEmbeddingStore;
pub use id::{KnowledgeIdGenerator, KnowledgeIdGeneratorError, SnowflakeKnowledgeIdGenerator};
pub use index_store::{KnowledgeIndexStoreError, SqliteKnowledgeIndexStore};
pub use postgres_pgvector_retrieval::PgVectorKnowledgeRetrievalBackend;
pub use retrieval_profile_store::{
    KnowledgeRetrievalProfileStoreError, SqliteKnowledgeRetrievalProfileStore,
};
pub use retrieval_store::SqliteKnowledgeChunkRetrievalStore;
pub use sqlite_chunk_store::SqliteKnowledgeChunkStore;
pub use sqlite_context_binding_store::SqliteContextBindingStore;
pub use sqlite_import_stores::{
    SqliteIngestionJobStore, SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeSourceStore,
};
pub use sqlite_outbox_store::SqliteKnowledgeOutboxStore;
pub use sqlite_space_stores::{SqliteKnowledgeSpaceStore, SqliteKnowledgeWikiFileEntryStore};
pub use wiki_page_store::SqliteKnowledgeWikiPageStore;
