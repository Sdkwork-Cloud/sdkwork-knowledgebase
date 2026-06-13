use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::SQLITE_CORE_MIGRATION;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeWikiPageStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::{
    AppendKnowledgeWikiLogEntryRecord, CreateKnowledgeWikiPageRevisionRecord,
    KnowledgeWikiPageStore, MarkKnowledgeWikiCurrentRevisionRecord, UpsertKnowledgeWikiPageRecord,
};
use sdkwork_knowledgebase_contract::wiki::{
    WikiLogEventType, WikiPagePublishState, WikiPageType, WikiRevisionReviewState,
};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

#[tokio::test]
async fn sqlite_wiki_page_store_publishes_pages_revisions_logs_and_projections() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let wiki_pages = SqliteKnowledgeWikiPageStore::new(pool, tenant_id);

    let page = wiki_pages
        .upsert_page(UpsertKnowledgeWikiPageRecord {
            space_id: 7,
            slug: "entity-name".to_string(),
            title: "Entity Name".to_string(),
            page_type: WikiPageType::Entity,
            logical_path: "wiki/pages/entities/entity-name/current.md".to_string(),
            summary: "Entity summary.".to_string(),
            source_count: 2,
            tags: vec!["entity".to_string(), "research".to_string()],
            publish_state: WikiPagePublishState::CandidateReady,
        })
        .await
        .unwrap();
    assert_eq!(page.publish_state, WikiPagePublishState::CandidateReady);
    assert_eq!(page.current_revision_id, None);

    let next_revision_no = wiki_pages.next_revision_no(page.id).await.unwrap();
    assert_eq!(next_revision_no, 1);

    let markdown_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-entity-current".to_string()),
            logical_path: Some("wiki/pages/entities/entity-name/current.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "wiki/pages/entities/entity-name/current.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-current".to_string()),
            object_role: "wiki_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    let revision = wiki_pages
        .create_revision(CreateKnowledgeWikiPageRevisionRecord {
            page_id: page.id,
            revision_no: next_revision_no,
            markdown_object_ref_id: markdown_ref.id,
            content_hash: "content-hash".to_string(),
            review_state: WikiRevisionReviewState::Approved,
        })
        .await
        .unwrap();
    assert_eq!(revision.revision_no, 1);
    assert_eq!(revision.review_state, WikiRevisionReviewState::Approved);

    let published = wiki_pages
        .mark_current_revision(MarkKnowledgeWikiCurrentRevisionRecord {
            page_id: page.id,
            revision_id: revision.id,
            publish_state: WikiPagePublishState::Published,
        })
        .await
        .unwrap();
    assert_eq!(published.current_revision_id, Some(revision.id));
    assert_eq!(published.publish_state, WikiPagePublishState::Published);

    let log_entry = wiki_pages
        .append_log_entry(AppendKnowledgeWikiLogEntryRecord {
            space_id: 7,
            event_type: WikiLogEventType::Publish.as_str().to_string(),
            event_time: "2026-06-04T12:00:00Z".to_string(),
            title: "Published Entity Name".to_string(),
            actor: "system".to_string(),
            affected_pages: vec!["Entity Name".to_string()],
            audit_event_id: Some("audit-1".to_string()),
            warnings: vec![],
            privacy_level: "internal".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(log_entry.event_type, WikiLogEventType::Publish);
    assert_eq!(log_entry.actor, "system");

    let summaries = wiki_pages.list_page_summaries(7).await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].logical_path, published.logical_path);
    assert_eq!(summaries[0].summary, "Entity summary.");
    assert_eq!(summaries[0].source_count, 2);
    assert_eq!(
        summaries[0].tags,
        vec!["entity".to_string(), "research".to_string()]
    );

    let logs = wiki_pages.list_log_entries(7).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].affected_pages, vec!["Entity Name".to_string()]);

    let projections = wiki_pages
        .batch_page_projections_by_paths(7, vec![published.logical_path.clone()])
        .await
        .unwrap();
    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].page_id, page.id);
    assert_eq!(projections[0].current_revision_id, Some(revision.id));
    assert_eq!(
        projections[0].publish_state,
        WikiPagePublishState::Published
    );
}

#[tokio::test]
async fn sqlite_wiki_page_store_reserves_revision_numbers_before_revision_insert() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    insert_space(&pool, 9001, 7).await;
    let wiki_pages = SqliteKnowledgeWikiPageStore::new(pool, 9001);

    let page = wiki_pages
        .upsert_page(UpsertKnowledgeWikiPageRecord {
            space_id: 7,
            slug: "concurrent-topic".to_string(),
            title: "Concurrent Topic".to_string(),
            page_type: WikiPageType::Topic,
            logical_path: "wiki/pages/topics/concurrent-topic/current.md".to_string(),
            summary: "Concurrency summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: WikiPagePublishState::CandidateReady,
        })
        .await
        .unwrap();

    let first_reserved = wiki_pages.next_revision_no(page.id).await.unwrap();
    let second_reserved = wiki_pages.next_revision_no(page.id).await.unwrap();

    assert_eq!(first_reserved, 1);
    assert_eq!(second_reserved, 2);
}

#[tokio::test]
async fn sqlite_wiki_page_store_rejects_duplicate_revision_number_for_same_page() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let wiki_pages = SqliteKnowledgeWikiPageStore::new(pool, tenant_id);

    let page = wiki_pages
        .upsert_page(UpsertKnowledgeWikiPageRecord {
            space_id: 7,
            slug: "duplicate-revision-topic".to_string(),
            title: "Duplicate Revision Topic".to_string(),
            page_type: WikiPageType::Topic,
            logical_path: "wiki/pages/topics/duplicate-revision-topic/current.md".to_string(),
            summary: "Duplicate revision summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: WikiPagePublishState::CandidateReady,
        })
        .await
        .unwrap();

    let first_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-revision-1".to_string()),
            logical_path: Some(
                "wiki/pages/topics/duplicate-revision-topic/revisions/r1.md".to_string(),
            ),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "wiki/pages/topics/duplicate-revision-topic/revisions/r1.md"
                .to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-r1".to_string()),
            object_role: "wiki_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();
    let second_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-revision-1-duplicate".to_string()),
            logical_path: Some(
                "wiki/pages/topics/duplicate-revision-topic/revisions/r1-copy.md".to_string(),
            ),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "wiki/pages/topics/duplicate-revision-topic/revisions/r1-copy.md"
                .to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-r1-copy".to_string()),
            object_role: "wiki_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    wiki_pages
        .create_revision(CreateKnowledgeWikiPageRevisionRecord {
            page_id: page.id,
            revision_no: 1,
            markdown_object_ref_id: first_ref.id,
            content_hash: "content-hash-1".to_string(),
            review_state: WikiRevisionReviewState::Approved,
        })
        .await
        .unwrap();

    let error = wiki_pages
        .create_revision(CreateKnowledgeWikiPageRevisionRecord {
            page_id: page.id,
            revision_no: 1,
            markdown_object_ref_id: second_ref.id,
            content_hash: "content-hash-duplicate".to_string(),
            review_state: WikiRevisionReviewState::Approved,
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("UNIQUE"));
}

#[tokio::test]
async fn sqlite_wiki_page_store_rejects_unbounded_projection_batches() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    insert_space(&pool, 9001, 7).await;
    let wiki_pages = SqliteKnowledgeWikiPageStore::new(pool, 9001);

    let error = wiki_pages
        .batch_page_projections_by_paths(
            7,
            (0..201)
                .map(|index| format!("wiki/pages/entities/entity-{index}/current.md"))
                .collect(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("batch size"));
}

async fn sqlite_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap()
}

async fn apply_sqlite_migration(pool: &SqlitePool) {
    for statement in SQLITE_CORE_MIGRATION.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await.unwrap();
        }
    }
}

async fn insert_space(pool: &SqlitePool, tenant_id: u64, space_id: u64) {
    sqlx::query(
        r#"
        INSERT INTO kb_space (
            id,
            uuid,
            tenant_id,
            organization_id,
            name,
            status,
            llm_wiki_initialized,
            created_at,
            updated_at,
            version
        )
        VALUES (?, ?, ?, 0, ?, 1, 1, ?, ?, 0)
        "#,
    )
    .bind(space_id as i64)
    .bind(format!("space-{space_id}"))
    .bind(tenant_id as i64)
    .bind(format!("Knowledge Space {space_id}"))
    .bind("2026-06-05T00:00:00Z")
    .bind("2026-06-05T00:00:00Z")
    .execute(pool)
    .await
    .unwrap();
}
