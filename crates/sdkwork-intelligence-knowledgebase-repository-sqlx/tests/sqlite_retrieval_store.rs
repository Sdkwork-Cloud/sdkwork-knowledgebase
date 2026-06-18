use sdkwork_intelligence_knowledgebase_repository_sqlx::migrations::SQLITE_CORE_MIGRATION;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    KnowledgeIdGenerator, KnowledgeIdGeneratorError, SqliteKnowledgeChunkRetrievalStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_trace_store::{
    CreateKnowledgeRetrievalHitRecord, CreateKnowledgeRetrievalTraceRecord,
    KnowledgeRetrievalTraceStore,
};
use sdkwork_knowledgebase_contract::rag::{KnowledgeRetrievalBinding, KnowledgeRetrievalMethod};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use std::sync::{Arc, Mutex};

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
        WHERE id = ?
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
        WHERE retrieval_trace_id = ?
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

async fn sqlite_pool() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap()
}

async fn apply_sqlite_migration(pool: &SqlitePool) {
    for statement in SQLITE_CORE_MIGRATION.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await.unwrap();
        }
    }
}

async fn seed_documents_and_chunks(pool: &SqlitePool) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_document (
            id, uuid, tenant_id, space_id, collection_id, identity_scope, title,
            visibility, content_state, index_state, status, created_at, updated_at, version
        )
        VALUES
            (201, 'doc-201', 9001, 7, 0, 'source_and_original_drive_node', 'Support Playbook', 1, 1, 2, 1, ?, ?, 0),
            (202, 'doc-202', 9001, 8, 0, 'source_and_original_drive_node', 'Billing Playbook', 1, 1, 2, 1, ?, ?, 0)
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
            (301, 'ver-301', 9001, 201, 1, 401, 100, 2, 2, ?, 1, ?, ?, 0),
            (302, 'ver-302', 9001, 202, 1, 402, 100, 2, 2, ?, 1, ?, ?, 0)
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
            (101, 'chunk-101', 9001, 7, 0, 201, 301, 1, 'enterprise renewal support playbook for premium accounts', 'hash-101', 7, 'section:renewal', 1, ?, ?, 0),
            (102, 'chunk-102', 9001, 7, 0, 201, 301, 2, 'support workflow for customer renewal escalations', 'hash-102', 6, 'section:workflow', 1, ?, ?, 0),
            (103, 'chunk-103', 9001, 8, 0, 202, 302, 1, 'billing support escalation for invoices', 'hash-103', 5, 'section:billing', 1, ?, ?, 0),
            (104, 'chunk-104', 9002, 7, 0, 201, 301, 1, 'other tenant enterprise renewal support', 'hash-104', 5, 'section:other', 1, ?, ?, 0),
            (105, 'chunk-105', 9001, 7, 2, 201, 301, 3, 'billing collection scoped note', 'hash-105', 4, 'section:collection', 1, ?, ?, 0)
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
}

async fn seed_embedding_for_chunk(pool: &SqlitePool, chunk_id: i64, index_id: i64) {
    let now = "2026-06-09T00:00:00Z";
    sqlx::query(
        r#"
        INSERT INTO kb_index (
            id, uuid, tenant_id, space_id, collection_id, index_kind, schema_version, status, created_at, updated_at, version
        )
        VALUES (?, ?, 9001, 7, 0, 'vector', 'v1', 1, ?, ?, 0)
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
            id, uuid, tenant_id, index_id, chunk_id, embedding_hash, vector_ref, dimension,
            provider_id, model, metadata, status, created_at, updated_at, version
        )
        VALUES (?, ?, 9001, ?, ?, 'hash-emb', 'drive://vectors/chunk-101', 1536, 'openai', 'text-embedding-3-small', NULL, 1, ?, ?, 0)
        "#,
    )
    .bind(index_id + 1000)
    .bind(format!("embedding-{chunk_id}"))
    .bind(index_id)
    .bind(chunk_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .unwrap();
}
