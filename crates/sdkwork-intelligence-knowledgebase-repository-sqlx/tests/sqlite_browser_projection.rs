use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeBrowserProjectionStore, SqliteKnowledgeDocumentStore,
    SqliteKnowledgeDocumentVersionStore, SqliteKnowledgeDriveObjectRefStore,
    SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_browser_projection_store::KnowledgeBrowserProjectionStore;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::{
    CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::KnowledgeSpaceStore;
use sdkwork_knowledgebase_contract::OkfConceptPublishState;
use sqlx::{AnyPool, Row};

#[tokio::test]
async fn sqlite_space_store_persists_drive_space_binding() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeSpaceStore::new(pool.clone(), 9001, 7001);

    let space = store
        .create_space(sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::CreateKnowledgeSpaceRecord {
            name: "Research Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(space.drive_space_id, None);

    let bound = store
        .mark_drive_space_bound(space.id, "drv-kb-001".to_string())
        .await
        .unwrap();
    assert_eq!(bound.drive_space_id.as_deref(), Some("drv-kb-001"));

    let row = sqlx::query("SELECT drive_space_id FROM kb_space WHERE id = $1")
        .bind(space.id as i64)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        row.get::<Option<String>, _>("drive_space_id").as_deref(),
        Some("drv-kb-001")
    );
}

#[tokio::test]
async fn sqlite_space_store_deleted_space_releases_active_drive_space_binding() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeSpaceStore::new(pool.clone(), 9001, 7001);

    let first = store
        .create_space(sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::CreateKnowledgeSpaceRecord {
            name: "Failed Initialization".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .unwrap();
    store
        .mark_drive_space_bound(first.id, "drv-kb-001".to_string())
        .await
        .unwrap();
    store.mark_space_deleted(first.id).await.unwrap();

    let second = store
        .create_space(sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::CreateKnowledgeSpaceRecord {
            name: "Retried Initialization".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .unwrap();
    let rebound = store
        .mark_drive_space_bound(second.id, "drv-kb-001".to_string())
        .await
        .unwrap();

    assert_eq!(rebound.drive_space_id.as_deref(), Some("drv-kb-001"));
    assert!(store.get_space(first.id).await.is_err());

    let statuses = sqlx::query("SELECT id, status FROM kb_space ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.get::<i64, _>("id"), row.get::<i64, _>("status")))
        .collect::<Vec<_>>();
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0].1, 3);
    assert_eq!(statuses[1].1, 1);
}

#[tokio::test]
async fn sqlite_browser_projection_batches_document_status_by_drive_node_id() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let versions = SqliteKnowledgeDocumentVersionStore::new(pool.clone(), tenant_id);
    let projections = SqliteKnowledgeBrowserProjectionStore::new(pool, tenant_id);

    let object_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: 7,
            drive_space_id: Some("drv-kb-001".to_string()),
            drive_node_id: Some("node-pdf".to_string()),
            logical_path: Some("raw/documents/doc-1/original/report.pdf".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "raw/documents/doc-1/original/report.pdf".to_string(),
            drive_object_version: None,
            drive_etag: None,
            content_type: Some("application/pdf".to_string()),
            size_bytes: 42,
            checksum_sha256_hex: Some("checksum".to_string()),
            object_role: "original_document".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .unwrap();
    let document = documents
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: 7,
            collection_id: 0,
            source_id: None,
            identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
            original_file_drive_node_id: Some("node-pdf".to_string()),
            title: "Report".to_string(),
            mime_type: Some("application/pdf".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .unwrap();
    let version = versions
        .create_document_version(CreateKnowledgeDocumentVersionRecord {
            document_id: document.id,
            version_no: 1,
            original_object_ref_id: object_ref.id,
            checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            size_bytes: object_ref.size_bytes,
            mime_type: object_ref.content_type.clone(),
        })
        .await
        .unwrap();

    let batch = projections
        .batch_document_projections(7, vec!["node-folder".to_string(), "node-pdf".to_string()])
        .await
        .unwrap();

    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].drive_node_id, "node-pdf");
    assert_eq!(batch[0].document_id, document.id);
    assert_eq!(batch[0].current_version_id, Some(version.id));
    assert_eq!(batch[0].ingest_state, "pending");
    assert_eq!(batch[0].parse_state, "pending");
    assert_eq!(batch[0].index_state, "pending");
}

#[tokio::test]
async fn sqlite_browser_projection_batches_okf_concept_status_by_logical_path() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let tenant_id = 9001;
    insert_space(&pool, tenant_id, 7).await;
    let concepts =
        sdkwork_intelligence_knowledgebase_repository_sqlx::SqliteKnowledgeOkfConceptStore::new(
            pool.clone(),
            tenant_id,
        );
    let projections = SqliteKnowledgeBrowserProjectionStore::new(pool, tenant_id);

    let concept = concepts
        .upsert_concept(UpsertKnowledgeOkfConceptRecord {
            space_id: 7,
            concept_id: "entities/entity-name".to_string(),
            title: "Entity Name".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/entities/entity-name.md".to_string(),
            description: "Entity summary.".to_string(),
            source_count: 1,
            tags: vec![],
            publish_state: OkfConceptPublishState::Published,
        })
        .await
        .unwrap();

    let batch = projections
        .batch_okf_concept_projections(
            7,
            vec![
                "okf/index.md".to_string(),
                "okf/entities/entity-name.md".to_string(),
            ],
        )
        .await
        .unwrap();

    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].logical_path, concept.logical_path);
    assert_eq!(batch[0].concept_row_id, concept.id);
    assert_eq!(batch[0].publish_state, OkfConceptPublishState::Published);
}

#[tokio::test]
async fn sqlite_browser_projection_rejects_unbounded_document_projection_batches() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let projections = SqliteKnowledgeBrowserProjectionStore::new(pool, 9001);

    let error = projections
        .batch_document_projections(
            7,
            (0..201)
                .map(|index| format!("drive-node-{index}"))
                .collect(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("batch size"));
}

#[tokio::test]
async fn sqlite_browser_projection_rejects_unbounded_okf_projection_batches() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let projections = SqliteKnowledgeBrowserProjectionStore::new(pool, 9001);

    let error = projections
        .batch_okf_concept_projections(
            7,
            (0..201)
                .map(|index| format!("okf/entities/entity-{index}.md"))
                .collect(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("batch size"));
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
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

async fn apply_sqlite_migration(_pool: &AnyPool) {}
