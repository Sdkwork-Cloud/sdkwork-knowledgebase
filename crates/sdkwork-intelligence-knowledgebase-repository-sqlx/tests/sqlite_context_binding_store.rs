use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteContextBindingStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    context_binding::KnowledgeContextBindingService,
    ports::knowledge_space_store::{CreateKnowledgeSpaceRecord, KnowledgeSpaceStore},
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeContextType,
};
use sqlx::AnyPool;

#[tokio::test]
async fn context_binding_store_round_trips_after_migration_install() {
    let pool = sqlite_pool().await;
    let tenant_id = 20001_u64;
    let organization_id = 7001_u64;

    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Binding Space".to_string(),
            description: None,
            llm_wiki_initialized: false,
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
        .list_space_bindings(tenant_id, space.id, None)
        .await
        .expect("list bindings");
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].id, binding.id);
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}
