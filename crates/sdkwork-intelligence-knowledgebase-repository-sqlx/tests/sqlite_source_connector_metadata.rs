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
    let store = SqliteKnowledgeSourceStore::new(pool, 9002, 0);
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

#[tokio::test]
async fn source_store_isolates_organizations_within_one_tenant() {
    let pool = sqlite_pool().await;
    let first_organization = SqliteKnowledgeSourceStore::new(pool.clone(), 9003, 41);
    let second_organization = SqliteKnowledgeSourceStore::new(pool, 9003, 42);
    let record = CreateKnowledgeSourceRecord {
        space_id: 88,
        source_type: KnowledgeSourceType::Connector,
        provider: Some("dify".to_string()),
        drive_bucket: None,
        drive_prefix: None,
        connector_metadata_json: None,
    };

    let first = first_organization
        .create_or_get_source(record.clone())
        .await
        .expect("create first organization source");
    assert!(second_organization
        .list_sources_for_space(88)
        .await
        .expect("list second organization")
        .is_empty());

    let second = second_organization
        .create_or_get_source(record)
        .await
        .expect("create isolated second organization source");
    assert_ne!(first.id, second.id);
}

async fn sqlite_pool() -> AnyPool {
    connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("connect sqlite pool")
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}
