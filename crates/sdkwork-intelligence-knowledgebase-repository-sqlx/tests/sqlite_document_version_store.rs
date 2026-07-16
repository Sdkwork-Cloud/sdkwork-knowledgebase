use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeDocumentStore, SqliteKnowledgeDocumentVersionStore,
    SqliteKnowledgeDriveObjectRefStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::CreateNextKnowledgeDocumentVersionRecord;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore, MANAGED_DRIVE_ACCESS_MODE,
    SDKWORK_DRIVE_PROVIDER_KIND,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};
use std::sync::Arc;
use tokio::sync::Barrier;

#[tokio::test]
async fn latest_document_version_remains_correct_after_two_hundred_versions() {
    let fixture = version_fixture().await;

    for _ in 0..205 {
        fixture
            .versions
            .create_next_document_version(fixture.next_record())
            .await
            .expect("create next version");
    }

    let latest = fixture
        .versions
        .get_latest_version_for_document(fixture.document_id)
        .await
        .expect("get latest version")
        .expect("latest version");
    assert_eq!(latest.version_no, 205);

    let (first_page, next_cursor, has_more) = fixture
        .versions
        .list_versions_page_for_document(fixture.document_id, None, 200)
        .await
        .expect("first version page");
    assert_eq!(first_page.len(), 200);
    assert!(has_more);
    let cursor = next_cursor
        .expect("next cursor")
        .parse::<u64>()
        .expect("numeric cursor");
    let (second_page, _, second_has_more) = fixture
        .versions
        .list_versions_page_for_document(fixture.document_id, Some(cursor), 200)
        .await
        .expect("second version page");
    assert_eq!(second_page.len(), 5);
    assert!(!second_has_more);
}

#[tokio::test]
async fn concurrent_document_version_creation_allocates_unique_monotonic_numbers() {
    const CREATOR_COUNT: usize = 16;
    let fixture = version_fixture().await;
    let barrier = Arc::new(Barrier::new(CREATOR_COUNT + 1));
    let mut tasks = Vec::with_capacity(CREATOR_COUNT);

    for _ in 0..CREATOR_COUNT {
        let versions = fixture.versions.clone();
        let barrier = Arc::clone(&barrier);
        let record = fixture.next_record();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            versions.create_next_document_version(record).await
        }));
    }
    barrier.wait().await;

    let mut version_numbers = Vec::with_capacity(CREATOR_COUNT);
    for task in tasks {
        version_numbers.push(
            task.await
                .expect("join version creator")
                .expect("create concurrent version")
                .version_no,
        );
    }
    version_numbers.sort_unstable();
    assert_eq!(
        version_numbers,
        (1..=CREATOR_COUNT as u64).collect::<Vec<_>>()
    );

    let latest = fixture
        .versions
        .get_latest_version_for_document(fixture.document_id)
        .await
        .expect("get latest version")
        .expect("latest version");
    assert_eq!(latest.version_no, CREATOR_COUNT as u64);
}

struct VersionFixture {
    versions: SqliteKnowledgeDocumentVersionStore,
    document_id: u64,
    object_ref_id: u64,
}

impl VersionFixture {
    fn next_record(&self) -> CreateNextKnowledgeDocumentVersionRecord {
        CreateNextKnowledgeDocumentVersionRecord {
            document_id: self.document_id,
            original_object_ref_id: self.object_ref_id,
            checksum_sha256_hex: Some("checksum".to_string()),
            size_bytes: 42,
            mime_type: Some("text/markdown".to_string()),
        }
    }
}

async fn version_fixture() -> VersionFixture {
    let pool =
        sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
            "sqlite::memory:",
        )
        .await
        .expect("sqlite schema");
    let tenant_id = 100_001;
    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, 0);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Version Test Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .expect("create space");
    let documents = SqliteKnowledgeDocumentStore::new(pool.clone(), tenant_id);
    let document = documents
        .create_document(CreateKnowledgeDocumentRecord {
            space_id: space.id,
            collection_id: 0,
            source_id: None,
            identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
            original_file_drive_node_id: Some("version-test-node".to_string()),
            title: "Version Test Document".to_string(),
            mime_type: Some("text/markdown".to_string()),
            language: Some("en".to_string()),
        })
        .await
        .expect("create document");
    let object_refs = SqliteKnowledgeDriveObjectRefStore::new(pool.clone(), tenant_id);
    let object_ref = object_refs
        .create_object_ref(CreateKnowledgeDriveObjectRefRecord {
            space_id: space.id,
            drive_space_id: Some("kb-version-test".to_string()),
            drive_node_id: Some("version-test-node".to_string()),
            logical_path: Some("raw/version-test.md".to_string()),
            drive_provider_kind: SDKWORK_DRIVE_PROVIDER_KIND.to_string(),
            drive_storage_provider_id: "provider-kb".to_string(),
            drive_bucket: "knowledgebase-test".to_string(),
            drive_object_key: "raw/version-test.md".to_string(),
            drive_object_version: None,
            drive_etag: None,
            content_type: Some("text/markdown".to_string()),
            size_bytes: 42,
            checksum_sha256_hex: Some("checksum".to_string()),
            object_role: "original_document".to_string(),
            access_mode: MANAGED_DRIVE_ACCESS_MODE.to_string(),
        })
        .await
        .expect("create object ref");

    VersionFixture {
        versions: SqliteKnowledgeDocumentVersionStore::new(pool, tenant_id),
        document_id: document.id,
        object_ref_id: object_ref.id,
    }
}
