use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SqliteKnowledgeChunkRetrievalStore,
    SqliteKnowledgeChunkStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_chunk_store::KnowledgeChunkStore;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_trace_store::{
    CreateKnowledgeRetrievalHitRecord, CreateKnowledgeRetrievalTraceRecord,
    KnowledgeRetrievalTraceStore,
};
use sdkwork_knowledgebase_contract::rag::{KnowledgeRetrievalBinding, KnowledgeRetrievalMethod};
use sqlx::{AnyPool, Row};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn sqlite_chunk_store_lists_id_content_pairs_in_chunk_index_order() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    let store = SqliteKnowledgeChunkStore::new(pool, 9001);

    let pairs = store
        .list_chunk_id_content_for_document_version(301)
        .await
        .unwrap();

    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0].0, 101);
    assert_eq!(pairs[1].0, 102);
    assert_eq!(pairs[2].0, 105);
    assert!(pairs[0].1.contains("renewal support playbook"));
    assert!(pairs[1].1.contains("support workflow"));
    assert!(pairs[2].1.contains("billing collection"));
}

#[tokio::test]
async fn sqlite_retrieval_backend_searches_active_chunks_with_tenant_and_binding_scope() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([7001, 7002, 7003]),
    );

    let hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(2),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 2,
        })
        .await
        .unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].chunk_id, 101);
    assert_eq!(hits[0].document_id, 201);
    assert_eq!(hits[0].document_version_id, Some(301));
    assert_eq!(hits[0].title, "Support Playbook");
    assert_eq!(hits[0].locator.as_deref(), Some("section:renewal"));
    assert!(hits[0].score > hits[1].score);
    assert_eq!(hits[1].chunk_id, 102);

    let out_of_scope_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "billing".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: Some(3),
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();
    assert!(out_of_scope_hits.is_empty());
}

#[tokio::test]
async fn sqlite_retrieval_vector_method_requires_active_embedding_rows() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    seed_embedding_for_chunk(&pool, 101, 501).await;

    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([7001]),
    );

    let vector_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Vector,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();

    assert_eq!(vector_hits.len(), 1);
    assert_eq!(vector_hits[0].chunk_id, 101);

    let keyword_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();

    assert!(
        keyword_hits.len() > vector_hits.len(),
        "keyword search should return more hits than vector-only embedding search"
    );
    assert!(keyword_hits.iter().any(|hit| hit.chunk_id == 101));
    assert!(keyword_hits.iter().any(|hit| hit.chunk_id == 102));
}

#[tokio::test]
async fn sqlite_retrieval_vector_method_respects_collection_binding_scope() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    seed_chunk_embedding_with_vector(&pool, 101, 501, 1501, "[1.0,0.0,0.0]").await;
    seed_chunk_embedding_with_vector(&pool, 105, 502, 1505, "[1.0,0.0,0.0]").await;

    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([7001, 7002]),
    );

    let scoped_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "billing".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: Some(2),
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Vector,
            query_embedding: Some(vec![1.0, 0.0, 0.0]),
            top_k: 5,
        })
        .await
        .unwrap();

    assert_eq!(scoped_hits.len(), 1);
    assert_eq!(scoped_hits[0].chunk_id, 105);

    let out_of_scope_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "billing".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: Some(3),
                source_filter: None,
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Vector,
            query_embedding: Some(vec![1.0, 0.0, 0.0]),
            top_k: 5,
        })
        .await
        .unwrap();
    assert!(out_of_scope_hits.is_empty());
}

#[tokio::test]
async fn sqlite_retrieval_keyword_method_respects_document_language_filter() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    sqlx::query("UPDATE kb_document SET language = 'en' WHERE id = 201")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE kb_document SET language = 'fr' WHERE id = 202")
        .execute(&pool)
        .await
        .unwrap();

    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([7001, 7002]),
    );

    let english_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: None,
                document_filter: Some(vec![sdkwork_knowledgebase_contract::rag::KnowledgeFilter {
                    key: "language".to_string(),
                    value: "en".to_string(),
                }]),
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();

    assert!(!english_hits.is_empty());
    assert!(english_hits.iter().all(|hit| hit.document_id == 201));

    let french_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: None,
                document_filter: Some(vec![sdkwork_knowledgebase_contract::rag::KnowledgeFilter {
                    key: "language".to_string(),
                    value: "fr".to_string(),
                }]),
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();
    assert!(french_hits.is_empty());
}

#[tokio::test]
async fn sqlite_retrieval_keyword_method_respects_source_type_filter() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    seed_drive_source_binding(&pool, 601, 201).await;

    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([7001, 7002]),
    );

    let drive_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: Some(vec![sdkwork_knowledgebase_contract::rag::KnowledgeFilter {
                    key: "sourceType".to_string(),
                    value: "drive".to_string(),
                }]),
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();

    assert!(!drive_hits.is_empty());
    assert!(drive_hits.iter().all(|hit| hit.document_id == 201));

    let api_hits = store
        .search_chunks(KnowledgeChunkSearchRequest {
            tenant_id: 9001,
            query: "enterprise renewal support".to_string(),
            binding: KnowledgeRetrievalBinding {
                space_id: 7,
                collection_id: None,
                source_filter: Some(vec![sdkwork_knowledgebase_contract::rag::KnowledgeFilter {
                    key: "sourceType".to_string(),
                    value: "api".to_string(),
                }]),
                document_filter: None,
                priority: 0,
                top_k: Some(5),
                min_score: Some(0.0),
            },
            method: KnowledgeRetrievalMethod::Keyword,
            query_embedding: None,
            top_k: 5,
        })
        .await
        .unwrap();
    assert!(api_hits.is_empty());
}

#[tokio::test]
async fn sqlite_retrieval_trace_store_persists_trace_and_ranked_hits() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([8001, 8002, 8003]),
    );

    let trace_id = store
        .create_trace(CreateKnowledgeRetrievalTraceRecord {
            tenant_id: 9001,
            actor_id: Some(30001),
            retrieval_profile_id: Some(31),
            query_hash_sha256_hex: "hash-enterprise-renewal-support".to_string(),
            query_text_redacted: Some("enterprise renewal support".to_string()),
            request_payload_json: Some(r#"{"query":"enterprise renewal support"}"#.to_string()),
            latency_ms: Some(25),
            result_count: 2,
            status: "succeeded".to_string(),
        })
        .await
        .unwrap();

    store
        .create_hits(vec![
            CreateKnowledgeRetrievalHitRecord {
                tenant_id: 9001,
                retrieval_trace_id: trace_id,
                chunk_id: 101,
                document_id: 201,
                document_version_id: Some(301),
                score: Some(0.9),
                result_rank: 1,
                match_reason: Some("keyword".to_string()),
                citation_json: Some(r#"{"chunkId":"101"}"#.to_string()),
                metadata_json: None,
            },
            CreateKnowledgeRetrievalHitRecord {
                tenant_id: 9001,
                retrieval_trace_id: trace_id,
                chunk_id: 102,
                document_id: 201,
                document_version_id: Some(301),
                score: Some(0.7),
                result_rank: 2,
                match_reason: Some("keyword".to_string()),
                citation_json: Some(r#"{"chunkId":"102"}"#.to_string()),
                metadata_json: None,
            },
        ])
        .await
        .unwrap();

    assert_eq!(trace_id, 8001);

    let trace_row = sqlx::query(
        r#"
        SELECT tenant_id, actor_id, retrieval_profile_id, result_count, status
        FROM kb_retrieval_trace
        WHERE id = $1
        "#,
    )
    .bind(trace_id as i64)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(trace_row.get::<i64, _>("tenant_id"), 9001);
    assert_eq!(trace_row.get::<Option<i64>, _>("actor_id"), Some(30001));
    assert_eq!(
        trace_row.get::<Option<i64>, _>("retrieval_profile_id"),
        Some(31)
    );
    assert_eq!(trace_row.get::<i64, _>("result_count"), 2);
    assert_eq!(trace_row.get::<i64, _>("status"), 1);

    let hit_rows = sqlx::query(
        r#"
        SELECT chunk_id, result_rank, score, match_reason
        FROM kb_retrieval_hit
        WHERE retrieval_trace_id = $1
        ORDER BY result_rank
        "#,
    )
    .bind(trace_id as i64)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(hit_rows.len(), 2);
    assert_eq!(hit_rows[0].get::<i64, _>("chunk_id"), 101);
    assert_eq!(hit_rows[0].get::<i64, _>("result_rank"), 1);
    assert_eq!(hit_rows[0].get::<String, _>("match_reason"), "keyword");
    assert_eq!(hit_rows[1].get::<i64, _>("chunk_id"), 102);
}

#[tokio::test]
async fn sqlite_retrieval_trace_store_reconstructs_trace_and_hits() {
    let pool = sqlite_pool().await;
    apply_sqlite_migration(&pool).await;
    seed_documents_and_chunks(&pool).await;
    let store = SqliteKnowledgeChunkRetrievalStore::with_id_generator(
        pool.clone(),
        9001,
        fixed_id_generator([8101, 8102]),
    );

    let trace_id = store
        .create_trace(CreateKnowledgeRetrievalTraceRecord {
            tenant_id: 9001,
            actor_id: Some(30001),
            retrieval_profile_id: Some(31),
            query_hash_sha256_hex: "hash-enterprise-renewal-support".to_string(),
            query_text_redacted: Some("enterprise renewal support".to_string()),
            request_payload_json: Some(r#"{"query":"enterprise renewal support"}"#.to_string()),
            latency_ms: Some(25),
            result_count: 1,
            status: "succeeded".to_string(),
        })
        .await
        .unwrap();
    store
        .create_hits(vec![CreateKnowledgeRetrievalHitRecord {
            tenant_id: 9001,
            retrieval_trace_id: trace_id,
            chunk_id: 101,
            document_id: 201,
            document_version_id: Some(301),
            score: Some(0.9),
            result_rank: 1,
            match_reason: Some("hybrid".to_string()),
            citation_json: Some(r#"{"chunkId":"101"}"#.to_string()),
            metadata_json: None,
        }])
        .await
        .unwrap();

    let trace = store.retrieve_trace(9001, trace_id).await.unwrap();
    let hits = store.list_trace_hits(9001, trace_id).await.unwrap();

    assert_eq!(trace.retrieval_trace_id, 8101);
    assert_eq!(trace.tenant_id, 9001);
    assert_eq!(trace.retrieval_profile_id, Some(31));
    assert_eq!(trace.status, "succeeded");
    assert_eq!(trace.result_count, 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk_id, 101);
    assert_eq!(hits[0].document_id, 201);
    assert_eq!(hits[0].space_id, 7);
    assert_eq!(hits[0].title, "Support Playbook");
    assert_eq!(hits[0].result_rank, 1);
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

async fn sqlite_pool() -> AnyPool {
    sdkwork_intelligence_knowledgebase_repository_sqlx::connect_sqlite_and_install_schema(
        "sqlite::memory:",
    )
    .await
    .unwrap()
}

async fn apply_sqlite_migration(_pool: &AnyPool) {}

async fn seed_documents_and_chunks(pool: &AnyPool) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_document (
            id, uuid, tenant_id, space_id, collection_id, identity_scope, title,
            visibility, content_state, index_state, status, created_at, updated_at, version
        )
        VALUES
            (201, 'doc-201', 9001, 7, 0, 'source_and_original_drive_node', 'Support Playbook', 1, 1, 2, 1, $1, $2, 0),
            (202, 'doc-202', 9001, 8, 0, 'source_and_original_drive_node', 'Billing Playbook', 1, 1, 2, 1, $3, $4, 0)
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO kb_document_version (
            id, uuid, tenant_id, document_id, version_no, original_object_ref_id,
            size_bytes, parse_state, index_state, submitted_at, status, created_at, updated_at, version
        )
        VALUES
            (301, 'ver-301', 9001, 201, 1, 401, 100, 2, 2, $1, 1, $2, $3, 0),
            (302, 'ver-302', 9001, 202, 1, 402, 100, 2, 2, $4, 1, $5, $6, 0)
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query("UPDATE kb_document SET current_version_id = 301 WHERE id = 201")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE kb_document SET current_version_id = 302 WHERE id = 202")
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
            (101, 'chunk-101', 9001, 7, 0, 201, 301, 1, 'enterprise renewal support playbook for premium accounts', 'hash-101', 7, 'section:renewal', 1, $1, $2, 0),
            (102, 'chunk-102', 9001, 7, 0, 201, 301, 2, 'support workflow for customer renewal escalations', 'hash-102', 6, 'section:workflow', 1, $3, $4, 0),
            (103, 'chunk-103', 9001, 8, 0, 202, 302, 1, 'billing support escalation for invoices', 'hash-103', 5, 'section:billing', 1, $5, $6, 0),
            (104, 'chunk-104', 9002, 7, 0, 201, 301, 1, 'other tenant enterprise renewal support', 'hash-104', 5, 'section:other', 1, $7, $8, 0),
            (105, 'chunk-105', 9001, 7, 2, 201, 301, 3, 'billing collection scoped note', 'hash-105', 4, 'section:collection', 1, $9, $10, 0)
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO kb_chunk_fts (content_text, chunk_id, tenant_id, space_id, document_id)
        SELECT c.content_text, c.id, c.tenant_id, c.space_id, c.document_id
        FROM kb_chunk c
        WHERE c.tenant_id = 9001
        "#,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_drive_source_binding(pool: &AnyPool, source_id: i64, document_id: i64) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_source (
            id, uuid, tenant_id, space_id, source_type, provider, status, created_at, updated_at, version
        )
        VALUES ($1, $2, 9001, 7, 'drive', 'sdkwork-drive', 1, $3, $4, 0)
        "#,
    )
    .bind(source_id)
    .bind(format!("source-{source_id}"))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query("UPDATE kb_document SET source_id = $1 WHERE id = $2")
        .bind(source_id)
        .bind(document_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn seed_embedding_for_chunk(pool: &AnyPool, chunk_id: i64, index_id: i64) {
    seed_chunk_embedding_with_vector(pool, chunk_id, index_id, index_id + 1000, "[0.1,0.2,0.3]")
        .await;
}

async fn seed_chunk_embedding_with_vector(
    pool: &AnyPool,
    chunk_id: i64,
    index_id: i64,
    embedding_id: i64,
    vector_json: &str,
) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_index (
            id, uuid, tenant_id, space_id, collection_id, index_kind, schema_version, status, created_at, updated_at, version
        )
        VALUES ($1, $2, 9001, 7, 0, 'vector', 'v1', 1, $3, $4, 0)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(index_id)
    .bind(format!("index-{index_id}"))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO kb_embedding (
            id, uuid, tenant_id, index_id, chunk_id, embedding_hash, vector_ref, vector_json, dimension,
            provider_id, model, metadata, status, created_at, updated_at, version
        )
        VALUES ($1, $2, 9001, $3, $4, 'hash-emb', 'inline://vector_json', $5, 3, 'openai', 'text-embedding-3-small', NULL, 1, $6, $7, 0)
        "#,
    )
    .bind(embedding_id)
    .bind(format!("embedding-{chunk_id}"))
    .bind(index_id)
    .bind(chunk_id)
    .bind(vector_json)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();
}
