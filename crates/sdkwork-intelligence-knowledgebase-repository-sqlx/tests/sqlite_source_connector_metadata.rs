use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, SqliteKnowledgeSourceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore,
};
use sdkwork_knowledgebase_contract::source::KnowledgeSourceType;
use sqlx::AnyPool;

#[tokio::test]
async fn sqlite_source_store_persists_connector_metadata_json() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeSourceStore::new(pool, 9002);
    let created = store
        .create_source(CreateKnowledgeSourceRecord {
            space_id: 42,
            source_type: KnowledgeSourceType::Connector,
            provider: Some("dify".to_string()),
            drive_bucket: None,
            drive_prefix: None,
            connector_metadata_json: Some(r#"{"origin":"external_import"}"#.to_string()),
        })
        .await
        .expect("create source");

    assert_eq!(
        created.connector_metadata_json.as_deref(),
        Some(r#"{"origin":"external_import"}"#)
    );

    let listed = store
        .list_sources_for_space(42)
        .await
        .expect("list sources");
    assert_eq!(listed.len(), 1);
    assert_eq!(
        listed[0].connector_metadata_json.as_deref(),
        Some(r#"{"origin":"external_import"}"#)
    );
}

async fn sqlite_pool() -> AnyPool {
    connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("connect sqlite pool")
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
