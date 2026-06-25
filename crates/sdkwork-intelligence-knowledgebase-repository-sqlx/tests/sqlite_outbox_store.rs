use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, SqliteKnowledgeOutboxStore,
};
use sdkwork_intelligence_knowledgebase_service::outbox::{
    KnowledgeOutboxPublisherService, LoggingKnowledgeOutboxDispatcher,
};
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

    let published =
        KnowledgeOutboxPublisherService::new(1, &store, &LoggingKnowledgeOutboxDispatcher)
            .publish_pending(10)
            .await
            .expect("publish outbox batch");
    assert_eq!(published.published, 1);
    assert_eq!(published.failed, 0);

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn sqlite_outbox_store_claim_prevents_duplicate_publish() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1);
    store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "ingestion_job".to_string(),
            aggregate_id: 99,
            event_type: "knowledge.ingest.succeeded".to_string(),
            payload_json: r#"{"spaceId":1}"#.to_string(),
        })
        .await
        .expect("append outbox event");

    let first_claim = store.claim_pending_events(10).await.expect("first claim");
    assert_eq!(first_claim.len(), 1);

    let second_claim = store.claim_pending_events(10).await.expect("second claim");
    assert!(second_claim.is_empty());
}

#[tokio::test]
async fn sqlite_outbox_store_requeues_failed_events_under_retry_limit() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1);
    store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "ingestion_job".to_string(),
            aggregate_id: 9,
            event_type: "knowledge.ingest.succeeded".to_string(),
            payload_json: r#"{"spaceId":1}"#.to_string(),
        })
        .await
        .expect("append outbox event");

    let event_id: i64 = sqlx::query_scalar("SELECT id FROM kb_outbox_event WHERE tenant_id = 1")
        .fetch_one(&pool)
        .await
        .expect("event id");

    store
        .mark_failed(event_id as u64, "dispatch failed")
        .await
        .expect("mark failed");

    let requeued = store
        .requeue_failed_events(10, 5)
        .await
        .expect("requeue failed events");
    assert_eq!(requeued, 1);

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND status = 0",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");
    assert_eq!(pending, 1);
}
