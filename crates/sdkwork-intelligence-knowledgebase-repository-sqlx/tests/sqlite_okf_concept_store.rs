use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeOkfConceptStore,
    SqliteOkfConceptRevisionMetadataStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::UpsertKnowledgeOkfCandidateRecord;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, CreateKnowledgeOkfConceptRevisionRecord,
    KnowledgeOkfConceptStore, MarkKnowledgeOkfConceptCurrentRevisionRecord,
    UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::okf_concept_revision_metadata_store::{
    OkfConceptRevisionMetadataStore, StageOkfConceptRevisionMetadataRecord,
};
use sdkwork_knowledgebase_contract::okf::OkfCandidateType;
use sdkwork_knowledgebase_contract::okf::OkfLogEventType;
use sdkwork_knowledgebase_contract::{OkfConceptPublishState, OkfRevisionReviewState};
use sqlx::AnyPool;

#[tokio::test]
async fn sqlite_okf_concept_store_publishes_concepts_revisions_logs_and_projections() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool, tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "entities/entity-name".to_string(),
            title: "Entity Name".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/entity-name.md".to_string(),
            description: "Entity summary.".to_string(),
            source_count: 2,
            tags: vec!["entity".to_string(), "research".to_string()],
            publish_state: OkfConceptPublishState::CandidateReady,
        })
        .await
        .unwrap();
    assert_eq!(
        concept.publish_state,
        OkfConceptPublishState::CandidateReady
    );
    assert_eq!(concept.current_revision_id, None);

    let next_revision_no = concepts.next_revision_no(concept.id).await.unwrap();
    assert_eq!(next_revision_no, 1);

    let markdown_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-entity-current".to_string()),
            logical_path: Some("okf/entities/entity-name.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "okf/entities/entity-name.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-current".to_string()),
            object_role: "concept_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    let revision = concepts
        .create_revision(CreateKnowledgeOkfConceptRevisionRecord {
            concept_row_id: concept.id,
            revision_no: next_revision_no,
            markdown_object_ref_id: markdown_ref.id,
            content_hash: "content-hash".to_string(),
            review_state: OkfRevisionReviewState::Approved,
        })
        .await
        .unwrap();
    assert_eq!(revision.revision_no, 1);
    assert_eq!(revision.review_state, OkfRevisionReviewState::Approved);

    let published = concepts
        .mark_current_revision(MarkKnowledgeOkfConceptCurrentRevisionRecord {
            concept_row_id: concept.id,
            revision_id: revision.id,
            publish_state: OkfConceptPublishState::Published,
        })
        .await
        .unwrap();
    assert_eq!(published.current_revision_id, Some(revision.id));
    assert_eq!(published.publish_state, OkfConceptPublishState::Published);

    let log_entry = concepts
        .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
            space_id: 7,
            event_type: OkfLogEventType::Publish.as_str().to_string(),
            event_time: "2026-06-04T12:00:00Z".to_string(),
            title: "Published Entity Name".to_string(),
            actor: "system".to_string(),
            affected_concepts: vec!["Entity Name".to_string()],
            audit_event_id: Some("audit-1".to_string()),
            warnings: vec![],
            privacy_level: "internal".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(log_entry.event_type, OkfLogEventType::Publish);
    assert_eq!(log_entry.actor, "system");

    let summaries = concepts.list_concept_summaries(7).await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].logical_path, published.logical_path);
    assert_eq!(summaries[0].description, "Entity summary.");
    assert_eq!(summaries[0].source_count, 2);
    assert_eq!(
        summaries[0].tags,
        vec!["entity".to_string(), "research".to_string()]
    );

    let logs = concepts.list_log_entries(7).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].affected_concepts, vec!["Entity Name".to_string()]);

    let projections = concepts
        .batch_concept_projections_by_paths(7, vec![published.logical_path.clone()])
        .await
        .unwrap();
    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].concept_row_id, concept.id);
    assert_eq!(projections[0].current_revision_id, Some(revision.id));
    assert_eq!(
        projections[0].publish_state,
        OkfConceptPublishState::Published
    );
}

#[tokio::test]
async fn sqlite_okf_concept_store_reserves_revision_numbers_before_revision_insert() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    insert_space(&pool, 9001, 7).await;
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool, 9001);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "topics/concurrent-topic".to_string(),
            title: "Concurrent Topic".to_string(),
            concept_type: "Topic".to_string(),
            logical_path: "okf/topics/concurrent-topic.md".to_string(),
            description: "Concurrency summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: OkfConceptPublishState::CandidateReady,
        })
        .await
        .unwrap();

    let first_reserved = concepts.next_revision_no(concept.id).await.unwrap();
    let second_reserved = concepts.next_revision_no(concept.id).await.unwrap();

    assert_eq!(first_reserved, 1);
    assert_eq!(second_reserved, 2);
}

#[tokio::test]
async fn sqlite_okf_concept_store_rejects_duplicate_revision_number_for_same_concept() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool, tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "topics/duplicate-revision-topic".to_string(),
            title: "Duplicate Revision Topic".to_string(),
            concept_type: "Topic".to_string(),
            logical_path: "okf/topics/duplicate-revision-topic.md".to_string(),
            description: "Duplicate revision summary.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: OkfConceptPublishState::CandidateReady,
        })
        .await
        .unwrap();

    let first_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-revision-1".to_string()),
            logical_path: Some(
                ".sdkwork/governance/revisions/topics/duplicate-revision-topic/r1.md".to_string(),
            ),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: ".sdkwork/governance/revisions/topics/duplicate-revision-topic/r1.md"
                .to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-r1".to_string()),
            object_role: "concept_revision".to_string(),
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
                ".sdkwork/governance/revisions/topics/duplicate-revision-topic/r1-copy.md"
                    .to_string(),
            ),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key:
                ".sdkwork/governance/revisions/topics/duplicate-revision-topic/r1-copy.md"
                    .to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some("checksum-r1-copy".to_string()),
            object_role: "concept_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    concepts
        .create_revision(CreateKnowledgeOkfConceptRevisionRecord {
            concept_row_id: concept.id,
            revision_no: 1,
            markdown_object_ref_id: first_ref.id,
            content_hash: "content-hash-1".to_string(),
            review_state: OkfRevisionReviewState::Approved,
        })
        .await
        .unwrap();

    let error = concepts
        .create_revision(CreateKnowledgeOkfConceptRevisionRecord {
            concept_row_id: concept.id,
            revision_no: 1,
            markdown_object_ref_id: second_ref.id,
            content_hash: "content-hash-duplicate".to_string(),
            review_state: OkfRevisionReviewState::Approved,
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("UNIQUE"));
}

#[tokio::test]
async fn sqlite_okf_concept_store_rejects_unbounded_projection_batches() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    insert_space(&pool, 9001, 7).await;
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool, 9001);

    let error = concepts
        .batch_concept_projections_by_paths(
            7,
            (0..201)
                .map(|index| format!("okf/entities/entity-{index}.md"))
                .collect(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("batch size"));
}

#[tokio::test]
async fn sqlite_okf_concept_revision_metadata_stages_object_ref_revision_and_current_pointer_atomically(
) {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool.clone(), tenant_id);
    let metadata = SqliteOkfConceptRevisionMetadataStore::new(pool.clone(), tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "entities/atomic-entity".to_string(),
            title: "Atomic Entity".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/atomic-entity.md".to_string(),
            description: "Atomic summary.".to_string(),
            source_count: 1,
            tags: vec!["entity".to_string()],
            publish_state: OkfConceptPublishState::CandidateReady,
        })
        .await
        .unwrap();

    let object_ref_record =
        |logical_path: &str, object_key: &str| CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: None,
            logical_path: Some(logical_path.to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: object_key.to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 128,
            checksum_sha256_hex: Some(format!("checksum-{object_key}")),
            object_role: "concept_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        };

    let staged = metadata
        .stage_concept_revision_metadata(StageOkfConceptRevisionMetadataRecord {
            revision_object_ref: object_ref_record(
                ".sdkwork/governance/revisions/entities/atomic-entity/r1.md",
                ".sdkwork/governance/revisions/entities/atomic-entity/r1.md",
            ),
            published_object_ref: Some(object_ref_record(
                "okf/entities/atomic-entity.md",
                "okf/entities/atomic-entity.md",
            )),
            concept_row_id: concept.id,
            revision_no: 1,
            content_hash: "content-hash-atomic".to_string(),
            review_state: OkfRevisionReviewState::Approved,
            publish_state: OkfConceptPublishState::Published,
            candidate: None,
        })
        .await
        .unwrap();

    assert_eq!(staged.revision.revision_no, 1);
    assert_eq!(staged.concept.current_revision_id, Some(staged.revision.id));
    assert_eq!(
        staged.concept.publish_state,
        OkfConceptPublishState::Published
    );

    let object_ref_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_drive_object_ref")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(object_ref_count, 2);

    let revision_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kb_okf_concept_revision WHERE concept_row_id = $1",
    )
    .bind(concept.id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(revision_count, 1);
}

#[tokio::test]
async fn sqlite_okf_concept_revision_metadata_stages_candidate_with_revision_atomically() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool.clone(), tenant_id);
    let metadata = SqliteOkfConceptRevisionMetadataStore::new(pool.clone(), tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "entities/candidate-entity".to_string(),
            title: "Candidate Entity".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/candidate-entity.md".to_string(),
            description: "Candidate summary.".to_string(),
            source_count: 1,
            tags: vec!["entity".to_string()],
            publish_state: OkfConceptPublishState::Draft,
        })
        .await
        .unwrap();

    let revision_object_ref = CreateKnowledgeDriveObjectRefRecord {
        space_id: 7,
        drive_space_id: Some("drv-kb-001".to_string()),
        drive_node_id: None,
        logical_path: Some(
            ".sdkwork/governance/revisions/entities/candidate-entity/r1.md".to_string(),
        ),
        drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
        drive_storage_provider_id: "provider-kb".to_string(),
        drive_bucket: "knowledgebase-test".to_string(),
        drive_object_key: ".sdkwork/governance/revisions/entities/candidate-entity/r1.md"
            .to_string(),
        drive_object_version: Some("v1".to_string()),
        drive_etag: None,
        content_type: Some("text/markdown; charset=utf-8".to_string()),
        size_bytes: 128,
        checksum_sha256_hex: Some("checksum-candidate-r1".to_string()),
        object_role: "concept_revision".to_string(),
        access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
    };

    let staged = metadata
        .stage_concept_revision_metadata(StageOkfConceptRevisionMetadataRecord {
            revision_object_ref,
            published_object_ref: None,
            concept_row_id: concept.id,
            revision_no: 1,
            content_hash: "content-hash-candidate".to_string(),
            review_state: OkfRevisionReviewState::Pending,
            publish_state: OkfConceptPublishState::CandidateReady,
            candidate: Some(UpsertKnowledgeOkfCandidateRecord {
                space_id: 7,
                concept_row_id: concept.id,
                concept_id: "entities/candidate-entity".to_string(),
                candidate_type: OkfCandidateType::ConceptCreate,
                state: OkfConceptPublishState::CandidateReady,
                markdown_object_ref_id: 0,
            }),
        })
        .await
        .unwrap();

    assert_eq!(staged.revision.revision_no, 1);
    assert_eq!(
        staged.concept.publish_state,
        OkfConceptPublishState::CandidateReady
    );

    let candidate_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_okf_candidate WHERE concept_id = $1")
            .bind("entities/candidate-entity")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(candidate_count, 1);

    let candidate_object_ref_id: i64 = sqlx::query_scalar(
        "SELECT markdown_object_ref_id FROM kb_okf_candidate WHERE concept_id = $1",
    )
    .bind("entities/candidate-entity")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        candidate_object_ref_id,
        staged.revision_object_ref.id as i64
    );
}

#[tokio::test]
async fn sqlite_okf_concept_revision_slot_prepares_concept_and_revision_number_atomically() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let metadata = SqliteOkfConceptRevisionMetadataStore::new(pool.clone(), tenant_id);

    let concept_record = || UpsertKnowledgeOkfConceptRecord {
        space_id: 7,
        concept_id: "entities/slot-entity".to_string(),
        title: "Slot Entity".to_string(),
        concept_type: "Entity".to_string(),
        logical_path: "okf/entities/slot-entity.md".to_string(),
        description: "Slot summary.".to_string(),
        source_count: 1,
        tags: vec!["entity".to_string()],
        publish_state: OkfConceptPublishState::Draft,
    };

    let first = metadata
        .prepare_concept_revision_slot(concept_record())
        .await
        .unwrap();
    assert_eq!(first.revision_no, 1);

    let second = metadata
        .prepare_concept_revision_slot(concept_record())
        .await
        .unwrap();
    assert_eq!(second.revision_no, 2);
    assert_eq!(first.concept.id, second.concept.id);

    let revision_counter: i64 =
        sqlx::query_scalar("SELECT revision_counter FROM kb_okf_concept WHERE id = $1")
            .bind(first.concept.id as i64)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(revision_counter, 2);

    let revision_row_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kb_okf_concept_revision")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(revision_row_count, 0);
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}

async fn insert_space(pool: &AnyPool, tenant_id: u64, space_id: u64) {
    sqlx::query(
        r#"
        INSERT INTO kb_space (
            id,
            uuid,
            tenant_id,
            organization_id,
            name,
            status,
            okf_bundle_initialized,
            created_at,
            updated_at,
            version
        )
        VALUES ($1, $2, $3, 0, $4, 1, 1, $5, $6, 0)
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
