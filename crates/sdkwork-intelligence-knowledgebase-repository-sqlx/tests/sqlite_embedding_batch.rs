use sdkwork_database_config::DatabaseEngine;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_sqlite_and_install_schema, KnowledgeIdGenerator, KnowledgeIdGeneratorError,
    SqliteKnowledgeEmbeddingStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_embedding_store::ChunkEmbeddingUpsertRequest;
use sqlx::AnyPool;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sqlite_embedding_store_batch_upserts_and_updates_in_place() {
    let pool = sqlite_pool().await;
    seed_index_and_chunks(&pool).await;
    let store = SqliteKnowledgeEmbeddingStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([9101, 9102, 9103, 9104]),
        DatabaseEngine::Sqlite,
    );

    let vector_a = vec![0.1_f32, 0.2, 0.3];
    let vector_b = vec![0.4_f32, 0.5, 0.6];
    store
        .upsert_chunk_embeddings_batch(&[
            ChunkEmbeddingUpsertRequest {
                tenant_id: 9001,
                index_id: 501,
                chunk_id: 101,
                vector: vector_a.clone(),
                provider_id: None,
                model: None,
            },
            ChunkEmbeddingUpsertRequest {
                tenant_id: 9001,
                index_id: 501,
                chunk_id: 102,
                vector: vector_b.clone(),
                provider_id: None,
                model: None,
            },
        ])
        .await
        .unwrap();

    let count = embedding_count(&pool, 9001).await;
    assert_eq!(count, 2);

    let version_101 = embedding_version(&pool, 9001, 101).await;
    assert_eq!(version_101, 0);

    let vector_a_updated = vec![0.9_f32, 0.8, 0.7];
    store
        .upsert_chunk_embeddings_batch(&[ChunkEmbeddingUpsertRequest {
            tenant_id: 9001,
            index_id: 501,
            chunk_id: 101,
            vector: vector_a_updated,
            provider_id: Some("custom-provider".to_string()),
            model: Some("custom-model".to_string()),
        }])
        .await
        .unwrap();

    assert_eq!(embedding_count(&pool, 9001).await, 2);
    assert_eq!(embedding_version(&pool, 9001, 101).await, 1);

    let provider = sqlx::query_scalar::<_, String>(
        r#"
        SELECT provider_id
        FROM kb_embedding
        WHERE tenant_id = $1 AND chunk_id = $2
        "#,
    )
    .bind(9001_i64)
    .bind(101_i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(provider, "custom-provider");
}

async fn sqlite_pool() -> AnyPool {
    connect_sqlite_and_install_schema("sqlite::memory:")
        .await
        .unwrap()
}

async fn seed_index_and_chunks(pool: &AnyPool) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_index (
            id, uuid, tenant_id, space_id, collection_id, index_kind, schema_version, status, created_at, updated_at, version
        )
        VALUES (501, 'index-501', 9001, 7, 0, 'vector', 'v1', 1, $1, $2, 0)
        "#,
    )
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO kb_chunk (
            id, uuid, tenant_id, space_id, collection_id, document_id, document_version_id,
            chunk_index, content_text, content_hash, token_count, locator, status, created_at, updated_at, version
        )
        VALUES
            (101, 'chunk-101', 9001, 7, 0, 201, 301, 1, 'first chunk', 'hash-101', 2, 'loc-1', 1, $1, $2, 0),
            (102, 'chunk-102', 9001, 7, 0, 201, 301, 2, 'second chunk', 'hash-102', 2, 'loc-2', 1, $3, $4, 0)
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();
}

async fn embedding_count(pool: &AnyPool, tenant_id: i64) -> i64 {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM kb_embedding
        WHERE tenant_id = $1
        "#,
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn embedding_version(pool: &AnyPool, tenant_id: i64, chunk_id: i64) -> i64 {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT version
        FROM kb_embedding
        WHERE tenant_id = $1 AND chunk_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(chunk_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

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

fn fixed_id_generator(ids: impl IntoIterator<Item = u64>) -> Arc<dyn KnowledgeIdGenerator> {
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.reverse();
    Arc::new(FixedIdGenerator {
        ids: Mutex::new(ids),
    })
}
