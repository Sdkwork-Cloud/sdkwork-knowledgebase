use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_backend::{
    KnowledgeChunkSearchHit, KnowledgeChunkSearchRequest, KnowledgeRetrievalBackend,
    KnowledgeRetrievalBackendError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_retrieval_trace_store::{
    CreateKnowledgeRetrievalHitRecord, CreateKnowledgeRetrievalTraceRecord,
    KnowledgeRetrievalTraceHitRecord, KnowledgeRetrievalTraceRecord, KnowledgeRetrievalTraceStore,
    KnowledgeRetrievalTraceStoreError,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeRetrievalMethod;
use sqlx::{any::AnyRow, AnyPool, QueryBuilder, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const SUCCEEDED_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeChunkRetrievalStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeChunkRetrievalStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            id_generator,
        }
    }
}

impl SqliteKnowledgeChunkRetrievalStore {
    pub async fn list_trace_summaries(
        &self,
        limit: u32,
    ) -> Result<Vec<KnowledgeRetrievalTraceRecord>, KnowledgeRetrievalTraceStoreError> {
        let tenant_id = trace_to_i64("tenant_id", self.tenant_id)?;
        let limit = i64::from(limit.clamp(1, 200));
        let rows = sqlx::query(
            r#"
            SELECT
                tenant_id,
                id AS retrieval_trace_id,
                retrieval_profile_id,
                query_text_redacted,
                latency_ms,
                result_count,
                status
            FROM kb_retrieval_trace
            WHERE tenant_id = $1
            ORDER BY id DESC
            LIMIT $2
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(trace_sqlx_error)?;

        rows.into_iter().map(trace_record_from_row).collect()
    }
}

#[async_trait]
impl KnowledgeRetrievalBackend for SqliteKnowledgeChunkRetrievalStore {
    async fn search_chunks(
        &self,
        request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        if request.tenant_id != self.tenant_id {
            return Err(KnowledgeRetrievalBackendError::TenantMismatch);
        }

        match request.method {
            KnowledgeRetrievalMethod::Vector => {
                if request.query_embedding.is_some() {
                    self.search_vector_with_embedding(request).await
                } else {
                    self.search_embedding_indexed_chunks(request, TermMatchOperator::Any)
                        .await
                }
            }
            KnowledgeRetrievalMethod::Hybrid => {
                if request.query_embedding.is_some() {
                    self.search_hybrid_with_embedding(request).await
                } else {
                    self.search_chunks_with_term_operator(request, TermMatchOperator::Any)
                        .await
                }
            }
            KnowledgeRetrievalMethod::Graph
            | KnowledgeRetrievalMethod::External
            | KnowledgeRetrievalMethod::Structured
            | KnowledgeRetrievalMethod::LlmRerank => {
                return Err(KnowledgeRetrievalBackendError::UnsupportedMethod(
                    request.method,
                ));
            }
            KnowledgeRetrievalMethod::Exact => {
                self.search_chunks_with_term_operator(request, TermMatchOperator::All)
                    .await
            }
            _ => {
                self.search_chunks_with_term_operator(request, TermMatchOperator::Any)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TermMatchOperator {
    Any,
    All,
}

impl SqliteKnowledgeChunkRetrievalStore {
    async fn search_chunks_with_term_operator(
        &self,
        request: KnowledgeChunkSearchRequest,
        term_operator: TermMatchOperator,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        let tenant_id = backend_to_i64("tenant_id", self.tenant_id)?;
        let space_id = backend_to_i64("space_id", request.binding.space_id)?;
        let collection_id = request
            .binding
            .collection_id
            .map(|value| backend_to_i64("collection_id", value))
            .transpose()?;
        let top_k = i64::from(request.top_k.clamp(1, 64));
        let query_terms = normalized_query_terms(&request.query);

        if query_terms.is_empty() {
            return Ok(vec![]);
        }

        let mut query = QueryBuilder::new(
            r#"
            SELECT
                c.id AS chunk_id,
                c.document_id,
                c.document_version_id,
                c.space_id,
                c.collection_id,
                d.title,
                c.content_text,
                c.token_count,
                c.locator,
                "kb://documents/" || c.document_id AS source_uri,
            "#,
        );
        push_score_expression(&mut query, &query_terms);
        query.push(
            r#"
                AS score
            FROM kb_chunk c
            JOIN kb_document d
              ON d.tenant_id = c.tenant_id
             AND d.id = c.document_id
             AND d.status =
            "#,
        );
        query.push_bind(ACTIVE_STATUS);
        query.push(
            r#"
            WHERE c.tenant_id =
            "#,
        );
        query.push_bind(tenant_id);
        query.push(" AND c.space_id = ");
        query.push_bind(space_id);
        query.push(" AND c.status = ");
        query.push_bind(ACTIVE_STATUS);
        if let Some(collection_id) = collection_id {
            query.push(" AND c.collection_id = ");
            query.push_bind(collection_id);
        }
        query.push(" AND (");
        for (index, term) in query_terms.iter().enumerate() {
            if index > 0 {
                query.push(match term_operator {
                    TermMatchOperator::Any => " OR ",
                    TermMatchOperator::All => " AND ",
                });
            }
            query.push("LOWER(c.content_text) LIKE ");
            query.push_bind(format!("%{term}%"));
            query.push(" OR LOWER(d.title) LIKE ");
            query.push_bind(format!("%{term}%"));
        }
        query.push(") ORDER BY score DESC, c.id ASC LIMIT ");
        query.push_bind(top_k);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(backend_sqlx_error)?;

        rows.into_iter()
            .map(|row| chunk_hit_from_row(row, request.method, request.binding.min_score))
            .filter_map(Result::transpose)
            .collect()
    }

    async fn search_embedding_indexed_chunks(
        &self,
        request: KnowledgeChunkSearchRequest,
        term_operator: TermMatchOperator,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        let tenant_id = backend_to_i64("tenant_id", self.tenant_id)?;
        let space_id = backend_to_i64("space_id", request.binding.space_id)?;
        let collection_id = request
            .binding
            .collection_id
            .map(|value| backend_to_i64("collection_id", value))
            .transpose()?;
        let top_k = i64::from(request.top_k.clamp(1, 64));
        let query_terms = normalized_query_terms(&request.query);

        if query_terms.is_empty() {
            return Ok(vec![]);
        }

        let mut query = QueryBuilder::new(
            r#"
            SELECT
                c.id AS chunk_id,
                c.document_id,
                c.document_version_id,
                c.space_id,
                c.collection_id,
                d.title,
                c.content_text,
                c.token_count,
                c.locator,
                "kb://documents/" || c.document_id AS source_uri,
            "#,
        );
        push_score_expression(&mut query, &query_terms);
        query.push(
            r#"
                AS score
            FROM kb_chunk c
            JOIN kb_document d
              ON d.tenant_id = c.tenant_id
             AND d.id = c.document_id
             AND d.status =
            "#,
        );
        query.push_bind(ACTIVE_STATUS);
        query.push(
            r#"
            INNER JOIN kb_embedding e
              ON e.tenant_id = c.tenant_id
             AND e.chunk_id = c.id
             AND e.status =
            "#,
        );
        query.push_bind(ACTIVE_STATUS);
        query.push(
            r#"
            WHERE c.tenant_id =
            "#,
        );
        query.push_bind(tenant_id);
        query.push(" AND c.space_id = ");
        query.push_bind(space_id);
        query.push(" AND c.status = ");
        query.push_bind(ACTIVE_STATUS);
        if let Some(collection_id) = collection_id {
            query.push(" AND c.collection_id = ");
            query.push_bind(collection_id);
        }
        query.push(" AND (");
        for (index, term) in query_terms.iter().enumerate() {
            if index > 0 {
                query.push(match term_operator {
                    TermMatchOperator::Any => " OR ",
                    TermMatchOperator::All => " AND ",
                });
            }
            query.push("LOWER(c.content_text) LIKE ");
            query.push_bind(format!("%{term}%"));
            query.push(" OR LOWER(d.title) LIKE ");
            query.push_bind(format!("%{term}%"));
        }
        query.push(") ORDER BY score DESC, c.id ASC LIMIT ");
        query.push_bind(top_k);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(backend_sqlx_error)?;

        rows.into_iter()
            .map(|row| chunk_hit_from_row(row, request.method, request.binding.min_score))
            .filter_map(Result::transpose)
            .collect()
    }

    async fn search_vector_with_embedding(
        &self,
        request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        let query_embedding = request.query_embedding.as_ref().ok_or_else(|| {
            KnowledgeRetrievalBackendError::Internal(
                "vector search requires query_embedding".to_string(),
            )
        })?;
        let candidates = self
            .load_embedding_candidates(&request, TermMatchOperator::Any)
            .await?;
        let mut hits = score_embedding_candidates(
            candidates,
            query_embedding,
            request.method,
            request.binding.min_score,
        );
        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(request.top_k.clamp(1, 64) as usize);
        Ok(hits)
    }

    async fn search_hybrid_with_embedding(
        &self,
        request: KnowledgeChunkSearchRequest,
    ) -> Result<Vec<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
        let query_embedding = request.query_embedding.as_ref().ok_or_else(|| {
            KnowledgeRetrievalBackendError::Internal(
                "hybrid search requires query_embedding".to_string(),
            )
        })?;
        let embedding_dimension = query_embedding.len();
        let keyword_hits = self
            .search_chunks_with_term_operator(request.clone(), TermMatchOperator::Any)
            .await?;
        let vector_hits = self.search_vector_with_embedding(request).await?;
        Ok(merge_hybrid_hits(
            keyword_hits,
            vector_hits,
            embedding_dimension,
        ))
    }

    async fn load_embedding_candidates(
        &self,
        request: &KnowledgeChunkSearchRequest,
        term_operator: TermMatchOperator,
    ) -> Result<Vec<EmbeddingCandidateRow>, KnowledgeRetrievalBackendError> {
        let tenant_id = backend_to_i64("tenant_id", self.tenant_id)?;
        let space_id = backend_to_i64("space_id", request.binding.space_id)?;
        let collection_id = request
            .binding
            .collection_id
            .map(|value| backend_to_i64("collection_id", value))
            .transpose()?;
        let top_k = i64::from((request.top_k.clamp(1, 64) * 4).clamp(4, 256));
        let query_terms = normalized_query_terms(&request.query);

        let mut query = QueryBuilder::new(
            r#"
            SELECT
                c.id AS chunk_id,
                c.document_id,
                c.document_version_id,
                c.space_id,
                c.collection_id,
                d.title,
                c.content_text,
                c.token_count,
                c.locator,
                "kb://documents/" || c.document_id AS source_uri,
                e.vector_json AS vector_json
            FROM kb_chunk c
            JOIN kb_document d
              ON d.tenant_id = c.tenant_id
             AND d.id = c.document_id
             AND d.status =
            "#,
        );
        query.push_bind(ACTIVE_STATUS);
        query.push(
            r#"
            INNER JOIN kb_embedding e
              ON e.tenant_id = c.tenant_id
             AND e.chunk_id = c.id
             AND e.status =
            "#,
        );
        query.push_bind(ACTIVE_STATUS);
        query.push(
            r#"
            WHERE c.tenant_id =
            "#,
        );
        query.push_bind(tenant_id);
        query.push(" AND c.space_id = ");
        query.push_bind(space_id);
        query.push(" AND c.status = ");
        query.push_bind(ACTIVE_STATUS);
        query.push(" AND e.vector_json IS NOT NULL");
        if let Some(collection_id) = collection_id {
            query.push(" AND c.collection_id = ");
            query.push_bind(collection_id);
        }
        if !query_terms.is_empty() {
            query.push(" AND (");
            for (index, term) in query_terms.iter().enumerate() {
                if index > 0 {
                    query.push(match term_operator {
                        TermMatchOperator::Any => " OR ",
                        TermMatchOperator::All => " AND ",
                    });
                }
                query.push("LOWER(c.content_text) LIKE ");
                query.push_bind(format!("%{term}%"));
                query.push(" OR LOWER(d.title) LIKE ");
                query.push_bind(format!("%{term}%"));
            }
            query.push(")");
        }
        query.push(" ORDER BY c.id ASC LIMIT ");
        query.push_bind(top_k);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(backend_sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(EmbeddingCandidateRow {
                    chunk_id: u64_from_row(&row, "chunk_id")?,
                    document_id: u64_from_row(&row, "document_id")?,
                    document_version_id: optional_u64_from_row(&row, "document_version_id")?,
                    space_id: u64_from_row(&row, "space_id")?,
                    collection_id: optional_u64_from_row(&row, "collection_id")?,
                    title: row.try_get("title").map_err(backend_sqlx_error)?,
                    content: row.try_get("content_text").map_err(backend_sqlx_error)?,
                    token_count: optional_i64_from_row(&row, "token_count")?
                        .map(|value| value as u32),
                    locator: row.try_get("locator").map_err(backend_sqlx_error)?,
                    source_uri: row.try_get("source_uri").map_err(backend_sqlx_error)?,
                    vector_json: row.try_get("vector_json").map_err(backend_sqlx_error)?,
                })
            })
            .collect()
    }
}

#[async_trait]
impl KnowledgeRetrievalTraceStore for SqliteKnowledgeChunkRetrievalStore {
    async fn create_trace(
        &self,
        record: CreateKnowledgeRetrievalTraceRecord,
    ) -> Result<u64, KnowledgeRetrievalTraceStoreError> {
        if record.tenant_id != self.tenant_id {
            return Err(KnowledgeRetrievalTraceStoreError::Internal(
                "trace tenant_id must match store tenant scope".to_string(),
            ));
        }

        let id = next_i64_id(&self.id_generator).map_err(trace_id_error)?;
        let tenant_id = trace_to_i64("tenant_id", record.tenant_id)?;
        let actor_id = record
            .actor_id
            .map(|value| trace_to_i64("actor_id", value))
            .transpose()?;
        let retrieval_profile_id = record
            .retrieval_profile_id
            .map(|value| trace_to_i64("retrieval_profile_id", value))
            .transpose()?;
        let result_count = i64::from(record.result_count);
        let latency_ms = record.latency_ms.map(|value| value as i64);
        let status = trace_status_code(&record.status)?;
        let now = now_rfc3339().map_err(KnowledgeRetrievalTraceStoreError::Internal)?;

        sqlx::query(
            r#"
            INSERT INTO kb_retrieval_trace (
                id,
                uuid,
                tenant_id,
                actor_id,
                retrieval_profile_id,
                query_hash,
                query_text_redacted,
                request_payload,
                latency_ms,
                result_count,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(actor_id)
        .bind(retrieval_profile_id)
        .bind(record.query_hash_sha256_hex)
        .bind(record.query_text_redacted)
        .bind(record.request_payload_json)
        .bind(latency_ms)
        .bind(result_count)
        .bind(status)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .execute(&self.pool)
        .await
        .map_err(trace_sqlx_error)?;

        u64::try_from(id).map_err(|_| {
            KnowledgeRetrievalTraceStoreError::Internal(
                "generated retrieval trace id is negative".to_string(),
            )
        })
    }

    async fn create_hits(
        &self,
        records: Vec<CreateKnowledgeRetrievalHitRecord>,
    ) -> Result<(), KnowledgeRetrievalTraceStoreError> {
        for record in records {
            if record.tenant_id != self.tenant_id {
                return Err(KnowledgeRetrievalTraceStoreError::Internal(
                    "hit tenant_id must match store tenant scope".to_string(),
                ));
            }

            let id = next_i64_id(&self.id_generator).map_err(trace_id_error)?;
            let tenant_id = trace_to_i64("tenant_id", record.tenant_id)?;
            let retrieval_trace_id = trace_to_i64("retrieval_trace_id", record.retrieval_trace_id)?;
            let chunk_id = trace_to_i64("chunk_id", record.chunk_id)?;
            let document_id = trace_to_i64("document_id", record.document_id)?;
            let document_version_id = record
                .document_version_id
                .map(|value| trace_to_i64("document_version_id", value))
                .transpose()?;
            let result_rank = i64::from(record.result_rank);
            let now = now_rfc3339().map_err(KnowledgeRetrievalTraceStoreError::Internal)?;

            sqlx::query(
                r#"
                INSERT INTO kb_retrieval_hit (
                    id,
                    uuid,
                    tenant_id,
                    retrieval_trace_id,
                    chunk_id,
                    document_id,
                    document_version_id,
                    score,
                    result_rank,
                    match_reason,
                    citation,
                    metadata,
                    status,
                    created_at,
                    updated_at,
                    version
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(retrieval_trace_id)
            .bind(chunk_id)
            .bind(document_id)
            .bind(document_version_id)
            .bind(record.score)
            .bind(result_rank)
            .bind(record.match_reason)
            .bind(record.citation_json)
            .bind(record.metadata_json)
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .bind(now)
            .bind(INITIAL_VERSION)
            .execute(&self.pool)
            .await
            .map_err(trace_sqlx_error)?;
        }

        Ok(())
    }

    async fn retrieve_trace(
        &self,
        tenant_id: u64,
        retrieval_trace_id: u64,
    ) -> Result<KnowledgeRetrievalTraceRecord, KnowledgeRetrievalTraceStoreError> {
        if tenant_id != self.tenant_id {
            return Err(KnowledgeRetrievalTraceStoreError::NotFound(
                retrieval_trace_id,
            ));
        }

        let row = sqlx::query(
            r#"
            SELECT
                tenant_id,
                id AS retrieval_trace_id,
                retrieval_profile_id,
                query_text_redacted,
                latency_ms,
                result_count,
                status
            FROM kb_retrieval_trace
            WHERE tenant_id = $1 AND id = $2
            "#,
        )
        .bind(trace_to_i64("tenant_id", tenant_id)?)
        .bind(trace_to_i64("retrieval_trace_id", retrieval_trace_id)?)
        .fetch_optional(&self.pool)
        .await
        .map_err(trace_sqlx_error)?
        .ok_or(KnowledgeRetrievalTraceStoreError::NotFound(
            retrieval_trace_id,
        ))?;

        trace_record_from_row(row)
    }

    async fn list_trace_hits(
        &self,
        tenant_id: u64,
        retrieval_trace_id: u64,
    ) -> Result<Vec<KnowledgeRetrievalTraceHitRecord>, KnowledgeRetrievalTraceStoreError> {
        if tenant_id != self.tenant_id {
            return Err(KnowledgeRetrievalTraceStoreError::NotFound(
                retrieval_trace_id,
            ));
        }

        let rows = sqlx::query(
            r#"
            SELECT
                h.chunk_id,
                h.document_id,
                h.document_version_id,
                c.space_id,
                c.collection_id,
                d.title,
                c.content_text,
                h.score,
                h.result_rank,
                h.match_reason,
                h.citation,
                c.token_count
            FROM kb_retrieval_hit h
            JOIN kb_chunk c
              ON c.tenant_id = h.tenant_id
             AND c.id = h.chunk_id
            JOIN kb_document d
              ON d.tenant_id = h.tenant_id
             AND d.id = h.document_id
            WHERE h.tenant_id = $1 AND h.retrieval_trace_id = $2
            ORDER BY h.result_rank ASC, h.id ASC
            "#,
        )
        .bind(trace_to_i64("tenant_id", tenant_id)?)
        .bind(trace_to_i64("retrieval_trace_id", retrieval_trace_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(trace_sqlx_error)?;

        rows.into_iter().map(trace_hit_from_row).collect()
    }
}

#[derive(Debug, Clone)]
struct EmbeddingCandidateRow {
    chunk_id: u64,
    document_id: u64,
    document_version_id: Option<u64>,
    space_id: u64,
    collection_id: Option<u64>,
    title: String,
    content: String,
    token_count: Option<u32>,
    locator: Option<String>,
    source_uri: Option<String>,
    vector_json: String,
}

fn score_embedding_candidates(
    candidates: Vec<EmbeddingCandidateRow>,
    query_embedding: &[f32],
    method: KnowledgeRetrievalMethod,
    min_score: Option<f64>,
) -> Vec<KnowledgeChunkSearchHit> {
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let vector = parse_embedding_vector(&candidate.vector_json).ok()?;
            if vector.len() != query_embedding.len() {
                return None;
            }
            let score = cosine_similarity_f32(query_embedding, &vector);
            if min_score
                .map(|min_score| score < min_score)
                .unwrap_or(false)
            {
                return None;
            }
            Some(KnowledgeChunkSearchHit {
                chunk_id: candidate.chunk_id,
                document_id: candidate.document_id,
                document_version_id: candidate.document_version_id,
                space_id: candidate.space_id,
                collection_id: candidate.collection_id,
                title: candidate.title,
                content: candidate.content,
                score,
                token_count: candidate.token_count,
                locator: candidate.locator,
                source_uri: candidate.source_uri,
                retrieval_method: method,
                match_reason: Some("vector_embedding".to_string()),
            })
        })
        .collect()
}

fn merge_hybrid_hits(
    keyword_hits: Vec<KnowledgeChunkSearchHit>,
    vector_hits: Vec<KnowledgeChunkSearchHit>,
    _dimension: usize,
) -> Vec<KnowledgeChunkSearchHit> {
    let mut merged = std::collections::BTreeMap::<u64, KnowledgeChunkSearchHit>::new();

    for hit in keyword_hits {
        merged.insert(
            hit.chunk_id,
            KnowledgeChunkSearchHit {
                score: hit.score * 0.4,
                match_reason: Some("hybrid_keyword".to_string()),
                ..hit
            },
        );
    }

    for hit in vector_hits {
        merged
            .entry(hit.chunk_id)
            .and_modify(|existing| {
                existing.score += hit.score * 0.6;
                existing.match_reason = Some("hybrid_keyword_vector".to_string());
            })
            .or_insert(KnowledgeChunkSearchHit {
                score: hit.score * 0.6,
                match_reason: Some("hybrid_vector".to_string()),
                ..hit
            });
    }

    let mut hits = merged.into_values().collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

fn parse_embedding_vector(payload: &str) -> Result<Vec<f32>, KnowledgeRetrievalBackendError> {
    serde_json::from_str(payload).map_err(|error| {
        KnowledgeRetrievalBackendError::Internal(format!("invalid vector_json payload: {error}"))
    })
}

fn cosine_similarity_f32(left: &[f32], right: &[f32]) -> f64 {
    if left.is_empty() || right.is_empty() || left.len() != right.len() {
        return 0.0;
    }

    let mut dot = 0.0f64;
    let mut left_norm = 0.0f64;
    let mut right_norm = 0.0f64;
    for (a, b) in left.iter().zip(right.iter()) {
        let a = f64::from(*a);
        let b = f64::from(*b);
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    dot / (left_norm.sqrt() * right_norm.sqrt())
}

fn push_score_expression(query: &mut QueryBuilder<'_, sqlx::Any>, terms: &[String]) {
    query.push("(");
    for (index, term) in terms.iter().enumerate() {
        if index > 0 {
            query.push(" + ");
        }
        query.push("CASE WHEN LOWER(c.content_text) LIKE ");
        query.push_bind(format!("%{term}%"));
        query.push(" THEN 1.0 ELSE 0.0 END + CASE WHEN LOWER(d.title) LIKE ");
        query.push_bind(format!("%{term}%"));
        query.push(" THEN 0.5 ELSE 0.0 END");
    }
    query.push(")");
}

fn chunk_hit_from_row(
    row: AnyRow,
    method: KnowledgeRetrievalMethod,
    min_score: Option<f64>,
) -> Result<Option<KnowledgeChunkSearchHit>, KnowledgeRetrievalBackendError> {
    let score: f64 = row.try_get("score").map_err(backend_sqlx_error)?;
    if min_score
        .map(|min_score| score < min_score)
        .unwrap_or(false)
    {
        return Ok(None);
    }

    Ok(Some(KnowledgeChunkSearchHit {
        chunk_id: u64_from_row(&row, "chunk_id")?,
        document_id: u64_from_row(&row, "document_id")?,
        document_version_id: optional_u64_from_row(&row, "document_version_id")?,
        space_id: u64_from_row(&row, "space_id")?,
        collection_id: optional_u64_from_row(&row, "collection_id")?,
        title: row.try_get("title").map_err(backend_sqlx_error)?,
        content: row.try_get("content_text").map_err(backend_sqlx_error)?,
        score,
        token_count: optional_i64_from_row(&row, "token_count")?.map(|value| value as u32),
        locator: row.try_get("locator").map_err(backend_sqlx_error)?,
        source_uri: row.try_get("source_uri").map_err(backend_sqlx_error)?,
        retrieval_method: method,
        match_reason: Some(format!("{method:?}")),
    }))
}

fn normalized_query_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|term| {
            term.trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|term| !term.is_empty())
        .take(8)
        .collect()
}

fn trace_status_code(status: &str) -> Result<i64, KnowledgeRetrievalTraceStoreError> {
    match status {
        "succeeded" => Ok(SUCCEEDED_STATUS),
        value => Err(KnowledgeRetrievalTraceStoreError::Internal(format!(
            "unsupported retrieval trace status: {value}"
        ))),
    }
}

fn trace_status_name(status: i64) -> Result<String, KnowledgeRetrievalTraceStoreError> {
    match status {
        SUCCEEDED_STATUS => Ok("succeeded".to_string()),
        value => Err(KnowledgeRetrievalTraceStoreError::Internal(format!(
            "unsupported retrieval trace status code: {value}"
        ))),
    }
}

fn trace_method_from_match_reason(
    value: Option<String>,
) -> Result<KnowledgeRetrievalMethod, KnowledgeRetrievalTraceStoreError> {
    match value
        .as_deref()
        .unwrap_or("hybrid")
        .to_ascii_lowercase()
        .as_str()
    {
        "exact" => Ok(KnowledgeRetrievalMethod::Exact),
        "keyword" => Ok(KnowledgeRetrievalMethod::Keyword),
        "fulltext" | "full_text" => Ok(KnowledgeRetrievalMethod::FullText),
        "structured" => Ok(KnowledgeRetrievalMethod::Structured),
        "graph" => Ok(KnowledgeRetrievalMethod::Graph),
        "vector" => Ok(KnowledgeRetrievalMethod::Vector),
        "hybrid" => Ok(KnowledgeRetrievalMethod::Hybrid),
        "llmrerank" | "llm_rerank" => Ok(KnowledgeRetrievalMethod::LlmRerank),
        "external" => Ok(KnowledgeRetrievalMethod::External),
        value => Err(KnowledgeRetrievalTraceStoreError::Internal(format!(
            "unsupported retrieval hit match reason: {value}"
        ))),
    }
}

fn trace_record_from_row(
    row: AnyRow,
) -> Result<KnowledgeRetrievalTraceRecord, KnowledgeRetrievalTraceStoreError> {
    let result_count = row
        .try_get::<i64, _>("result_count")
        .map_err(trace_sqlx_error)?;
    Ok(KnowledgeRetrievalTraceRecord {
        tenant_id: trace_u64_from_row(&row, "tenant_id")?,
        retrieval_trace_id: trace_u64_from_row(&row, "retrieval_trace_id")?,
        retrieval_profile_id: trace_optional_u64_from_row(&row, "retrieval_profile_id")?,
        query_text_redacted: row
            .try_get("query_text_redacted")
            .map_err(trace_sqlx_error)?,
        latency_ms: trace_optional_i64_from_row(&row, "latency_ms")?.map(|value| value as u64),
        result_count: u32::try_from(result_count).map_err(|_| {
            KnowledgeRetrievalTraceStoreError::Internal(
                "result_count is out of u32 range".to_string(),
            )
        })?,
        status: trace_status_name(row.try_get("status").map_err(trace_sqlx_error)?)?,
    })
}

fn trace_hit_from_row(
    row: AnyRow,
) -> Result<KnowledgeRetrievalTraceHitRecord, KnowledgeRetrievalTraceStoreError> {
    let result_rank = row
        .try_get::<i64, _>("result_rank")
        .map_err(trace_sqlx_error)?;
    Ok(KnowledgeRetrievalTraceHitRecord {
        chunk_id: trace_u64_from_row(&row, "chunk_id")?,
        document_id: trace_u64_from_row(&row, "document_id")?,
        document_version_id: trace_optional_u64_from_row(&row, "document_version_id")?,
        space_id: trace_u64_from_row(&row, "space_id")?,
        collection_id: trace_optional_u64_from_row(&row, "collection_id")?,
        title: row.try_get("title").map_err(trace_sqlx_error)?,
        content: row.try_get("content_text").map_err(trace_sqlx_error)?,
        score: row.try_get("score").map_err(trace_sqlx_error)?,
        result_rank: u32::try_from(result_rank).map_err(|_| {
            KnowledgeRetrievalTraceStoreError::Internal(
                "result_rank is out of u32 range".to_string(),
            )
        })?,
        token_count: trace_optional_i64_from_row(&row, "token_count")?.map(|value| value as u32),
        retrieval_method: trace_method_from_match_reason(
            row.try_get("match_reason").map_err(trace_sqlx_error)?,
        )?,
        citation_json: row.try_get("citation").map_err(trace_sqlx_error)?,
    })
}

fn u64_from_row(row: &AnyRow, column: &str) -> Result<u64, KnowledgeRetrievalBackendError> {
    let value: i64 = row.try_get(column).map_err(backend_sqlx_error)?;
    u64::try_from(value).map_err(|_| {
        KnowledgeRetrievalBackendError::Internal(format!("{column} must not be negative"))
    })
}

fn optional_u64_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<Option<u64>, KnowledgeRetrievalBackendError> {
    optional_i64_from_row(row, column)?
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                KnowledgeRetrievalBackendError::Internal(format!("{column} must not be negative"))
            })
        })
        .transpose()
}

fn optional_i64_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<Option<i64>, KnowledgeRetrievalBackendError> {
    row.try_get(column).map_err(backend_sqlx_error)
}

fn backend_to_i64(field_name: &str, value: u64) -> Result<i64, KnowledgeRetrievalBackendError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeRetrievalBackendError::Internal(format!("{field_name} exceeds signed int64 range"))
    })
}

fn trace_to_i64(field_name: &str, value: u64) -> Result<i64, KnowledgeRetrievalTraceStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeRetrievalTraceStoreError::Internal(format!(
            "{field_name} exceeds signed int64 range"
        ))
    })
}

fn trace_u64_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<u64, KnowledgeRetrievalTraceStoreError> {
    let value: i64 = row.try_get(column).map_err(trace_sqlx_error)?;
    u64::try_from(value).map_err(|_| {
        KnowledgeRetrievalTraceStoreError::Internal(format!("{column} must not be negative"))
    })
}

fn trace_optional_u64_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<Option<u64>, KnowledgeRetrievalTraceStoreError> {
    trace_optional_i64_from_row(row, column)?
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                KnowledgeRetrievalTraceStoreError::Internal(format!(
                    "{column} must not be negative"
                ))
            })
        })
        .transpose()
}

fn trace_optional_i64_from_row(
    row: &AnyRow,
    column: &str,
) -> Result<Option<i64>, KnowledgeRetrievalTraceStoreError> {
    row.try_get(column).map_err(trace_sqlx_error)
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

fn backend_sqlx_error(error: sqlx::Error) -> KnowledgeRetrievalBackendError {
    KnowledgeRetrievalBackendError::Internal(error.to_string())
}

fn trace_sqlx_error(error: sqlx::Error) -> KnowledgeRetrievalTraceStoreError {
    KnowledgeRetrievalTraceStoreError::Internal(error.to_string())
}

fn trace_id_error(
    error: crate::id::KnowledgeIdGeneratorError,
) -> KnowledgeRetrievalTraceStoreError {
    KnowledgeRetrievalTraceStoreError::Internal(error.to_string())
}
