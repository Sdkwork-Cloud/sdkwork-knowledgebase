use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    db::connect_knowledgebase_any_pool_from_url, is_postgres_database_url,
    SqlxKnowledgeEngineProviderBindingReadinessStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_readiness_store::{
    KnowledgeEngineProviderBindingReadinessStore,
    ListKnowledgeEngineProviderBindingReadinessGapsRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::KnowledgeEngineProviderScope;
use sdkwork_knowledgebase_contract::{
    parse_canonical_nonnegative_signed_i64, parse_canonical_positive_signed_i64,
};

#[tokio::test]
async fn postgres_readiness_query_is_read_only_and_dialect_compatible_when_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!(
            "skipping PostgreSQL Provider Binding readiness query: configure a PostgreSQL SDKWORK_KNOWLEDGEBASE_DATABASE_URL and tenant scope"
        );
        return;
    };
    let tenant_id = parse_canonical_positive_signed_i64(
        &std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
            .expect("PostgreSQL readiness test requires SDKWORK_KNOWLEDGEBASE_TENANT_ID"),
    )
    .expect("canonical PostgreSQL readiness tenant id");
    let organization_id = std::env::var("SDKWORK_KNOWLEDGEBASE_ORGANIZATION_ID")
        .ok()
        .map(|value| parse_canonical_nonnegative_signed_i64(&value))
        .transpose()
        .expect("canonical PostgreSQL readiness organization id")
        .unwrap_or(0);
    let scope = KnowledgeEngineProviderScope {
        tenant_id,
        organization_id,
    };
    let pool = connect_knowledgebase_any_pool_from_url(&database_url)
        .await
        .expect("connect read-only PostgreSQL readiness pool");
    let store = SqlxKnowledgeEngineProviderBindingReadinessStore::new(pool.clone());

    let first = store
        .list_spaces_missing_active_binding(
            scope,
            ListKnowledgeEngineProviderBindingReadinessGapsRequest {
                cursor: None,
                page_size: Some(1),
            },
        )
        .await
        .expect("execute bounded PostgreSQL readiness query");
    assert!(first.items.len() <= 1);
    if let Some(cursor) = first.next_cursor {
        let second = store
            .list_spaces_missing_active_binding(
                scope,
                ListKnowledgeEngineProviderBindingReadinessGapsRequest {
                    cursor: Some(cursor),
                    page_size: Some(1),
                },
            )
            .await
            .expect("execute PostgreSQL readiness continuation query");
        assert!(second.items.len() <= 1);
    }
    pool.close().await;
}

fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|url| is_postgres_database_url(url))
        .filter(|_| std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID").is_ok())
}
