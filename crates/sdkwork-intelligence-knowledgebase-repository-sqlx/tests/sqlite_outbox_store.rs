use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, SqliteKnowledgeOutboxStore,
};
use sdkwork_intelligence_knowledgebase_service::outbox::{
    KnowledgeOutboxPublisherService, LoggingKnowledgeOutboxDispatcher,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_outbox_store::{
    AppendOutboxEventRecord, KnowledgeOutboxStore, KnowledgeOutboxStoreError,
    MAX_KNOWLEDGE_OUTBOX_PAYLOAD_BYTES,
};

#[tokio::test]
async fn sqlite_outbox_store_appends_pending_events() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
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
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND organization_id = 7 AND status = 0",
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
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
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
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND organization_id = 7 AND status = 0",
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
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
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
    let store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
    store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "ingestion_job".to_string(),
            aggregate_id: 9,
            event_type: "knowledge.ingest.succeeded".to_string(),
            payload_json: r#"{"spaceId":1}"#.to_string(),
        })
        .await
        .expect("append outbox event");

    let claimed = store
        .claim_pending_events(1)
        .await
        .expect("claim")
        .pop()
        .expect("claimed event");
    store
        .mark_failed(&claimed, "dispatch failed")
        .await
        .expect("mark failed");

    let requeued = store
        .requeue_failed_events(10, 5)
        .await
        .expect("requeue failed events");
    assert_eq!(requeued, 1);

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND organization_id = 7 AND status = 0",
    )
    .fetch_one(&pool)
    .await
    .expect("count pending");
    assert_eq!(pending, 1);
}

#[tokio::test]
async fn sqlite_outbox_store_rejects_invalid_or_oversized_payloads() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let store = SqliteKnowledgeOutboxStore::new(pool, 1, 7, "worker-a");

    for payload_json in [
        "not-json".to_string(),
        format!("\"{}\"", "x".repeat(MAX_KNOWLEDGE_OUTBOX_PAYLOAD_BYTES)),
    ] {
        let error = store
            .append_event(AppendOutboxEventRecord {
                aggregate_type: "knowledge_document".to_string(),
                aggregate_id: 42,
                event_type: "knowledge.document.changed.v1".to_string(),
                payload_json,
            })
            .await
            .expect_err("invalid payload must be rejected before persistence");

        assert!(matches!(
            error,
            KnowledgeOutboxStoreError::InvalidRequest(_)
        ));
    }
}

#[tokio::test]
async fn sqlite_outbox_store_fences_a_stale_worker_after_reclaim() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let first_store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
    let second_store = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-b");
    first_store
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "knowledge_document".to_string(),
            aggregate_id: 42,
            event_type: "knowledge.document.changed.v1".to_string(),
            payload_json: r#"{"documentId":42}"#.to_string(),
        })
        .await
        .expect("append");

    let stale_claim = first_store
        .claim_pending_events(1)
        .await
        .expect("first claim")
        .pop()
        .expect("claimed event");
    sqlx::query(
        "UPDATE kb_outbox_event SET claimed_at = '2000-01-01T00:00:00Z' WHERE tenant_id = 1 AND organization_id = 7",
    )
    .execute(&pool)
    .await
    .expect("expire claim");

    let current_claim = second_store
        .claim_pending_events(1)
        .await
        .expect("reclaim")
        .pop()
        .expect("reclaimed event");
    assert_ne!(stale_claim.claim.token, current_claim.claim.token);

    let stale_result = first_store.mark_published(&stale_claim).await;
    assert!(matches!(
        stale_result,
        Err(KnowledgeOutboxStoreError::InvalidRequest(_))
    ));
    second_store
        .mark_published(&current_claim)
        .await
        .expect("current owner publishes");
}

#[tokio::test]
async fn sqlite_outbox_store_isolates_organizations_and_dead_letters_exhausted_events() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .expect("schema install");
    let organization_seven = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 7, "worker-a");
    let organization_eight = SqliteKnowledgeOutboxStore::new(pool.clone(), 1, 8, "worker-b");
    organization_seven
        .append_event(AppendOutboxEventRecord {
            aggregate_type: "knowledge_document".to_string(),
            aggregate_id: 9,
            event_type: "knowledge.document.changed.v1".to_string(),
            payload_json: r#"{"documentId":9}"#.to_string(),
        })
        .await
        .expect("append");

    assert!(organization_eight
        .claim_pending_events(10)
        .await
        .expect("cross-organization claim")
        .is_empty());
    let claimed = organization_seven
        .claim_pending_events(1)
        .await
        .expect("claim")
        .pop()
        .expect("claimed event");
    organization_seven
        .mark_failed(&claimed, "permanent failure")
        .await
        .expect("mark failed");
    assert_eq!(
        organization_seven
            .requeue_failed_events(10, 1)
            .await
            .expect("dead letter exhausted event"),
        0
    );
    let dead_letter_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_outbox_event WHERE tenant_id = 1 AND organization_id = 7 AND status = 4 AND dead_lettered_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("dead letter count");
    assert_eq!(dead_letter_count, 1);
}
