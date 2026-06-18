use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, SqliteKnowledgeOutboxStore,
};
use sdkwork_intelligence_knowledgebase_service::outbox::KnowledgeOutboxPublisherService;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::{
    AppendOutboxEventRecord, KnowledgeOutboxStore,
};

#[tokio::test]
async fn sqlite_outbox_store_appends_pending_events() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1);
    store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "ingestion_job".to_string(),
            aggregate_id: 42,
            event_type: "knowledge.ingest.succeeded".to_string(),
            payload_json: r#"{"spaceId":1}"#.to_string(),
        })
        .await
        .expect("append outbox event");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0",
    )
    .fetch_one(&pool)
    .await
    .expect("count outbox rows");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn sqlite_outbox_store_marks_pending_events_published() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1);
    store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "ingestion_job".to_string(),
            aggregate_id: 7,
            event_type: "knowledge.ingest.succeeded".to_string(),
            payload_json: r#"{"spaceId":1}"#.to_string(),
        })
        .await
        .expect("append outbox event");

    let published = KnowledgeOutboxPublisherService::new(&store)
        .publish_pending(10)
        .await
        .expect("publish outbox batch");
    assert_eq!(published.published, 1);

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");
    assert_eq!(pending, 0);
}
