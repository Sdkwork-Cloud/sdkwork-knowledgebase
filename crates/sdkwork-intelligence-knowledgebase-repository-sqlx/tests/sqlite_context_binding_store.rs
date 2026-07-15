use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteContextBindingStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::{
    context_binding::KnowledgeContextBindingService,
    ports::knowledge_context_binding_store::{
        KnowledgeContextBindingStore, KnowledgeContextBindingStoreError,
    },
    ports::knowledge_space_store::{CreateKnowledgeSpaceRecord, KnowledgeSpaceStore},
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeContextType, ListContextBoundSpacesRequest,
    ListKnowledgeSpaceContextBindingsRequest, UpdateKnowledgeSpaceContextBindingRequest,
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

#[tokio::test]
async fn generic_context_binding_store_hides_legacy_chat_group_rows_before_pagination() {
    let pool = sqlite_pool().await;
    let tenant_id = 100004_u64;
    let organization_id = 7004_u64;
    let spaces = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id);
    let space = spaces
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Legacy Binding Isolation Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode: Default::default(),
        })
        .await
        .expect("create space");
    let store = SqliteContextBindingStore::new(pool.clone());

    assert!(matches!(
        store
            .create_binding(
                tenant_id,
                "operator-1",
                CreateKnowledgeSpaceContextBindingRequest {
                    space_id: space.id,
                    context_type: KnowledgeContextType::ChatGroup,
                    context_id: "new-group-must-use-dedicated-aggregate".to_string(),
                    context_name: None,
                    access_level: None,
                },
            )
            .await,
        Err(KnowledgeContextBindingStoreError::InvalidRequest(_))
    ));

    for (context_type, context_id) in [
        (KnowledgeContextType::Organization, "org-current"),
        (KnowledgeContextType::Team, "team-current"),
    ] {
        store
            .create_binding(
                tenant_id,
                "operator-1",
                CreateKnowledgeSpaceContextBindingRequest {
                    space_id: space.id,
                    context_type,
                    context_id: context_id.to_string(),
                    context_name: None,
                    access_level: None,
                },
            )
            .await
            .expect("create supported binding");
    }

    for (id, context_id) in [(1_i64, "legacy-group-a"), (2_i64, "legacy-group-b")] {
        sqlx::query(
            r#"
            INSERT INTO kb_space_context_binding (
                id, tenant_id, space_id, context_type, context_id, context_name,
                access_level, status, created_by, created_at, updated_at, version
            ) VALUES ($1, $2, $3, 'chat_group', $4, NULL, 'reader', 1, 'legacy',
                      '2026-07-13T00:00:00Z', '2026-07-13T00:00:00Z', 0)
            "#,
        )
        .bind(id)
        .bind(tenant_id as i64)
        .bind(space.id as i64)
        .bind(context_id)
        .execute(&pool)
        .await
        .expect("insert legacy chat_group binding");
    }

    let listed = store
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
        .expect("list generic bindings");
    assert_eq!(listed.items.len(), 2);
    assert!(listed.next_cursor.is_none());
    assert!(listed
        .items
        .iter()
        .all(|binding| binding.context_type != KnowledgeContextType::ChatGroup));

    assert!(matches!(
        store.get_binding(tenant_id, 1).await,
        Err(KnowledgeContextBindingStoreError::NotFound(1))
    ));
    assert!(matches!(
        store
            .update_binding(
                tenant_id,
                1,
                UpdateKnowledgeSpaceContextBindingRequest {
                    context_name: Some("mutate legacy".to_string()),
                    access_level: None,
                },
            )
            .await,
        Err(KnowledgeContextBindingStoreError::NotFound(1))
    ));
    assert!(matches!(
        store.delete_binding(tenant_id, 1).await,
        Err(KnowledgeContextBindingStoreError::NotFound(1))
    ));
    assert!(store
        .list_context_bound_spaces(
            tenant_id,
            ListContextBoundSpacesRequest {
                context_type: KnowledgeContextType::ChatGroup,
                context_id: "legacy-group-a".to_string(),
                cursor: None,
                page_size: Some(20),
            },
        )
        .await
        .expect("legacy group lookup must be hidden")
        .is_empty());
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}
