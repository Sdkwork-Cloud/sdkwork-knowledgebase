use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SqliteKnowledgeIndexStore,
    SqliteKnowledgeRetrievalProfileStore,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeIndexRequest, KnowledgeRetrievalProfileRequest,
};
use sqlx::{AnyPool, Row};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sqlite_index_store_creates_and_retrieves_active_index() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store =
        SqliteKnowledgeIndexStore::with_id_generator(pool.clone(), 9001, fixed_id_generator([801]));

    let created = store
        .create_index(KnowledgeIndexRequest {
            tenant_id: 9001,
            space_id: 7,
            collection_id: Some(3),
            index_kind: "vector".to_string(),
            embedding_provider_id: Some("provider.embedding.openai".to_string()),
            embedding_model: Some("text-embedding-3-small".to_string()),
            dimension: Some(1536),
            metric: Some("cosine".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(created.index_id, 801);
    assert_eq!(created.tenant_id, 9001);
    assert_eq!(created.space_id, 7);
    assert_eq!(created.index_kind, "vector");
    assert_eq!(created.status, "active");

    let loaded = store.get_index(801).await.unwrap();
    assert_eq!(loaded, created);

    let row = sqlx::query(
        r#"
        SELECT tenant_id, space_id, collection_id, index_kind, status
        FROM kb_index
        WHERE id = ?
        "#,
    )
    .bind(801_i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.get::<i64, _>("tenant_id"), 9001);
    assert_eq!(row.get::<i64, _>("space_id"), 7);
    assert_eq!(row.get::<i64, _>("collection_id"), 3);
    assert_eq!(row.get::<String, _>("index_kind"), "vector");
    assert_eq!(row.get::<i64, _>("status"), 1);
}

#[tokio::test]
async fn sqlite_retrieval_profile_store_creates_and_updates_profile() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    let store = SqliteKnowledgeRetrievalProfileStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([901]),
    );

    let created = store
        .create_profile(KnowledgeRetrievalProfileRequest {
            tenant_id: 9001,
            name: "Default Hybrid".to_string(),
            strategy: "hybrid".to_string(),
            top_k: 8,
            min_score: Some(0.35),
            rerank_enabled: true,
            context_budget_tokens: 4096,
            status: "active".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(created.retrieval_profile_id, 901);
    assert_eq!(created.name, "Default Hybrid");
    assert_eq!(created.strategy, "hybrid");
    assert_eq!(created.top_k, 8);
    assert!(created.rerank_enabled);

    let updated = store
        .update_profile(
            901,
            KnowledgeRetrievalProfileRequest {
                tenant_id: 9001,
                name: "Tight Hybrid".to_string(),
                strategy: "hybrid".to_string(),
                top_k: 4,
                min_score: Some(0.5),
                rerank_enabled: false,
                context_budget_tokens: 2048,
                status: "active".to_string(),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.retrieval_profile_id, 901);
    assert_eq!(updated.name, "Tight Hybrid");
    assert_eq!(updated.top_k, 4);
    assert!(!updated.rerank_enabled);

    let loaded = store.get_profile(901).await.unwrap();
    assert_eq!(loaded, updated);
}

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}

fn fixed_id_generator(ids: impl IntoIterator<Item = u64>) -> Arc<dyn KnowledgeIdGenerator> {
    #[derive(Debug)]
    struct FixedIdGenerator {
        ids: Mutex<Vec<u64>>,
    }

    impl KnowledgeIdGenerator for FixedIdGenerator {
        fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError> {
            self.ids
                .lock()
                .expect("fixed id generator lock poisoned")
                .pop()
                .ok_or_else(|| {
                    KnowledgeIdGeneratorError::Internal("fixed id generator exhausted".into())
                })
        }
    }

    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.reverse();
    Arc::new(FixedIdGenerator {
        ids: Mutex::new(ids),
    })
}
