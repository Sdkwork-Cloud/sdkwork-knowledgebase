use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteKnowledgeDocumentStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};

#[tokio::test]
async fn document_store_pages_by_id_cursor() {
    let pool = sqlite_pool().await;
    let tenant_id = 100002_u64;
    let organization_id = 7002_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Document Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .expect("create space");

    let store = SqliteKnowledgeDocumentStore::new(pool, tenant_id);
    for (index, title) in ["Alpha", "Beta", "Gamma"].into_iter().enumerate() {
        store
            .create_document(CreateKnowledgeDocumentRecord {
                space_id: space.id,
                collection_id: 0,
                source_id: None,
                identity_scope: KnowledgeDocumentIdentityScope::SourceAndOriginalDriveNode,
                original_file_drive_node_id: Some(format!("node-{index}")),
                title: title.to_string(),
                mime_type: None,
                language: None,
            })
            .await
            .expect("create document");
    }

    let (first_page, next_cursor, has_more) = store
        .list_documents_page(space.id, None, 2)
        .await
        .expect("first page");
    assert_eq!(2, first_page.len());
    assert!(has_more);
    assert_eq!(Some(first_page[1].id.to_string()), next_cursor);

    let cursor_id = next_cursor
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .expect("cursor id");
    let (second_page, second_cursor, second_has_more) = store
        .list_documents_page(space.id, Some(cursor_id), 2)
        .await
        .expect("second page");
    assert_eq!(1, second_page.len());
    assert!(!second_has_more);
    assert_eq!(None, second_cursor);
    assert!(first_page[0].id < first_page[1].id);
    assert!(first_page[1].id < second_page[0].id);
}

async fn sqlite_pool() -> sqlx::AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}
