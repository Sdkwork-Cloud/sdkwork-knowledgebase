use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, SqliteKnowledgeDriveObjectRefStore,
    SqliteKnowledgeOkfCandidateStore, SqliteKnowledgeOkfConceptStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateStore, UpsertKnowledgeOkfCandidateRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_knowledgebase_contract::{OkfCandidateType, OkfConceptPublishState};
use sqlx::AnyPool;

#[tokio::test]
async fn sqlite_okf_candidate_store_lists_only_registered_open_candidates() {
    let pool = connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .unwrap();
    let tenant_id = 9002;
    insert_space(&pool, tenant_id, 8).await;

    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let concepts = SqliteKnowledgeOkfConceptStore::new(pool.clone(), tenant_id);
    let candidates = SqliteKnowledgeOkfCandidateStore::new(pool, tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 8,
            concept_id: "tables/users".to_string(),
            title: "Users".to_string(),
            concept_type: "BigQuery Table".to_string(),
            logical_path: "okf/tables/users.md".to_string(),
            description: "Users table.".to_string(),
            source_count: 0,
            tags: vec![],
            publish_state: OkfConceptPublishState::CandidateReady,
        })
        .await
        .unwrap();

    let markdown_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 8,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: None,
            logical_path: Some(".sdkwork/governance/revisions/tables/users/r1.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: ".sdkwork/governance/revisions/tables/users/r1.md".to_string(),
            drive_object_version: Some("v1".to_string()),
            drive_etag: None,
            content_type: Some("text/markdown; charset=utf-8".to_string()),
            size_bytes: 64,
            checksum_sha256_hex: Some("checksum-governance".to_string()),
            object_role: "concept_revision".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();

    candidates
        .upsert_candidate(UpsertKnowledgeOkfCandidateRecord {
            space_id: 8,
            concept_row_id: concept.id,
            concept_id: concept.concept_id.clone(),
            candidate_type: OkfCandidateType::ConceptCreate,
            state: OkfConceptPublishState::CandidateReady,
            markdown_object_ref_id: markdown_ref.id,
        })
        .await
        .unwrap();

    let listed = candidates.list_open_candidates(Some(8)).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].concept_row_id, concept.id);

    candidates
        .update_candidate_state_by_concept_row_id(
            concept.id,
            OkfConceptPublishState::Rejected,
            None,
            Some("needs more citations".to_string()),
        )
        .await
        .unwrap();

    assert!(candidates
        .list_open_candidates(Some(8))
        .await
        .unwrap()
        .is_empty());
}

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
