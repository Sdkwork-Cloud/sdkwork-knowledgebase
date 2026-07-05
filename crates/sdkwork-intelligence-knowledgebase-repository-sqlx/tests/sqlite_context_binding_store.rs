use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteContextBindingStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    context_binding::KnowledgeContextBindingService,
    ports::knowledge_context_binding_store::KnowledgeContextBindingStore,
    ports::knowledge_space_store::{CreateKnowledgeSpaceRecord, KnowledgeSpaceStore},
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeContextType,
    ListKnowledgeSpaceContextBindingsRequest,
};
use sqlx::AnyPool;

#[tokio::test]
async fn context_binding_store_round_trips_after_migration_install() {
    let pool = sqlite_pool().await;
    let tenant_id = 100001_u64;
    let organization_id = 7001_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Binding Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .expect("create space");

    let store = SqliteContextBindingStore::new(pool);
    let service = KnowledgeContextBindingService::new(&store);
    let binding = service
        .bind_context(
            tenant_id,
            "operator-1",
            "drive-space-1",
            CreateKnowledgeSpaceContextBindingRequest {
                space_id: space.id,
                context_type: KnowledgeContextType::Organization,
                context_id: "org_knowledgebase_dev".to_string(),
                context_name: Some("Dev Org".to_string()),
                access_level: None,
            },
        )
        .await
        .expect("bind context");

    let listed = service
        .list_space_bindings(tenant_id, space.id, None, None, None)
        .await
        .expect("list bindings");
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].id, binding.id);
}

#[tokio::test]
async fn context_binding_store_pages_by_id_cursor() {
    let pool = sqlite_pool().await;
    let tenant_id = 100003_u64;
    let organization_id = 7003_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Paged Binding Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .expect("create space");

    let store = SqliteContextBindingStore::new(pool);
    for context_id in ["ctx-a", "ctx-b", "ctx-c"] {
        store
            .create_binding(
                tenant_id,
                "operator-1",
                CreateKnowledgeSpaceContextBindingRequest {
                    space_id: space.id,
                    context_type: KnowledgeContextType::Team,
                    context_id: context_id.to_string(),
                    context_name: None,
                    access_level: None,
                },
            )
            .await
            .expect("bind context");
    }

    let first_page = store
        .list_space_bindings(
            tenant_id,
            ListKnowledgeSpaceContextBindingsRequest {
                space_id: space.id,
                context_type: None,
                cursor: None,
                page_size: Some(2),
            },
        )
        .await
        .expect("first page");
    assert_eq!(2, first_page.items.len());
    assert!(first_page.next_cursor.is_some());

    let cursor = first_page.next_cursor.clone().expect("next cursor");
    let second_page = store
        .list_space_bindings(
            tenant_id,
            ListKnowledgeSpaceContextBindingsRequest {
                space_id: space.id,
                context_type: None,
                cursor: Some(cursor),
                page_size: Some(2),
            },
        )
        .await
        .expect("second page");
    assert_eq!(1, second_page.items.len());
    assert!(second_page.next_cursor.is_none());
    assert!(first_page.items[0].id < first_page.items[1].id);
    assert!(first_page.items[1].id < second_page.items[0].id);
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}
