use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    SqliteCommerceStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::{
    commerce_store::{KnowledgeMarketStore, KnowledgeMarketStoreError},
    knowledge_space_store::{CreateKnowledgeSpaceRecord, KnowledgeSpaceStore},
};
use sqlx::AnyPool;

#[tokio::test]
async fn empty_catalog_does_not_implicitly_publish_private_spaces() {
    let pool = sqlite_pool().await;
    let tenant_id = 110001_u64;
    let organization_id = 7101_u64;
    SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_id)
        .create_space(space_record("Private Space"))
        .await
        .expect("create private space");

    let store = SqliteCommerceStore::new(pool.clone(), organization_id);
    let (items, next_cursor, has_more) = store
        .list_catalog_page(tenant_id, None, None, 20)
        .await
        .expect("list empty catalog");

    assert!(items.is_empty());
    assert!(next_cursor.is_none());
    assert!(!has_more);
    let listing_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM kb_market_listing WHERE tenant_id = $1 AND organization_id = $2",
    )
    .bind(tenant_id as i64)
    .bind(organization_id as i64)
    .fetch_one(&pool)
    .await
    .expect("count market listings");
    assert_eq!(listing_count, 0);
}

#[tokio::test]
async fn market_catalog_and_subscriptions_are_isolated_by_organization() {
    let pool = sqlite_pool().await;
    let tenant_id = 110002_u64;
    let organization_a = 7102_u64;
    let organization_b = 7103_u64;
    let actor_id = 91001_u64;
    let listing_a = 81001_u64;
    let listing_b = 81002_u64;

    let space_a = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_a)
        .create_space(space_record("Organization A Space"))
        .await
        .expect("create organization A space");
    let space_b = SqliteKnowledgeSpaceStore::new(pool.clone(), tenant_id, organization_b)
        .create_space(space_record("Organization B Space"))
        .await
        .expect("create organization B space");
    insert_listing(
        &pool,
        listing_a,
        tenant_id,
        organization_a,
        space_a.id,
        "Organization A Listing",
    )
    .await;
    insert_listing(
        &pool,
        listing_b,
        tenant_id,
        organization_b,
        space_b.id,
        "Organization B Listing",
    )
    .await;

    let store_a = SqliteCommerceStore::new(pool.clone(), organization_a);
    let store_b = SqliteCommerceStore::new(pool.clone(), organization_b);
    let (items_a, _, _) = store_a
        .list_catalog_page(tenant_id, Some(actor_id), None, 20)
        .await
        .expect("list organization A catalog");
    let (items_b, _, _) = store_b
        .list_catalog_page(tenant_id, Some(actor_id), None, 20)
        .await
        .expect("list organization B catalog");
    assert_eq!(items_a.len(), 1);
    assert_eq!(items_a[0].id, listing_a.to_string());
    assert_eq!(items_b.len(), 1);
    assert_eq!(items_b[0].id, listing_b.to_string());

    assert_eq!(
        store_a.subscribe(tenant_id, actor_id, listing_b).await,
        Err(KnowledgeMarketStoreError::NotFound)
    );
    store_a
        .subscribe(tenant_id, actor_id, listing_a)
        .await
        .expect("subscribe organization A listing");
    assert!(matches!(
        store_a.subscribe(tenant_id, actor_id, listing_a).await,
        Err(KnowledgeMarketStoreError::InvalidRequest(_))
    ));
    assert_listing_state(&pool, tenant_id, organization_a, listing_a, 1, 1).await;
    assert_listing_state(&pool, tenant_id, organization_b, listing_b, 0, 0).await;

    let (subscribed_items, _, _) = store_a
        .list_catalog_page(tenant_id, Some(actor_id), None, 20)
        .await
        .expect("list subscribed organization A catalog");
    assert!(subscribed_items[0].is_subscribed);
    assert_eq!(subscribed_items[0].subscribers_count, 1);

    assert_eq!(
        store_b.unsubscribe(tenant_id, actor_id, listing_a).await,
        Err(KnowledgeMarketStoreError::NotFound)
    );
    store_a
        .unsubscribe(tenant_id, actor_id, listing_a)
        .await
        .expect("unsubscribe organization A listing");
    assert_eq!(
        store_a.unsubscribe(tenant_id, actor_id, listing_a).await,
        Err(KnowledgeMarketStoreError::NotFound)
    );
    assert_listing_state(&pool, tenant_id, organization_a, listing_a, 0, 0).await;
}

fn space_record(name: &str) -> CreateKnowledgeSpaceRecord {
    CreateKnowledgeSpaceRecord {
        name: name.to_string(),
        description: None,
        okf_bundle_initialized: false,
        knowledge_mode: Default::default(),
    }
}

async fn insert_listing(
    pool: &AnyPool,
    listing_id: u64,
    tenant_id: u64,
    organization_id: u64,
    space_id: u64,
    title: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO kb_market_listing (
            id, tenant_id, organization_id, space_id, title, icon, description, author,
            tags_json, provider, model_name, subscribers_count, documents_count,
            status, created_at, updated_at, version
        ) VALUES ($1, $2, $3, $4, $5, 'book', 'Published description', 'Publisher',
                  '["published"]', 'Configured provider', 'Configured model', 0, 1,
                  1, '2026-07-31T00:00:00Z', '2026-07-31T00:00:00Z', 0)
        "#,
    )
    .bind(listing_id as i64)
    .bind(tenant_id as i64)
    .bind(organization_id as i64)
    .bind(space_id as i64)
    .bind(title)
    .execute(pool)
    .await
    .expect("insert explicit market listing");
}

async fn assert_listing_state(
    pool: &AnyPool,
    tenant_id: u64,
    organization_id: u64,
    listing_id: u64,
    expected_subscribers: i64,
    expected_active_subscriptions: i64,
) {
    let subscribers = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT subscribers_count
        FROM kb_market_listing
        WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
        "#,
    )
    .bind(tenant_id as i64)
    .bind(organization_id as i64)
    .bind(listing_id as i64)
    .fetch_one(pool)
    .await
    .expect("read listing subscriber count");
    let subscriptions = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM kb_market_subscription
        WHERE tenant_id = $1 AND organization_id = $2 AND listing_id = $3 AND status = 1
        "#,
    )
    .bind(tenant_id as i64)
    .bind(organization_id as i64)
    .bind(listing_id as i64)
    .fetch_one(pool)
    .await
    .expect("count active subscriptions");
    assert_eq!(subscribers, expected_subscribers);
    assert_eq!(subscriptions, expected_active_subscriptions);
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .expect("install SQLite schema")
}
