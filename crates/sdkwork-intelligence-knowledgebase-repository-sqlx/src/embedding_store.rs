use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_embedding_store::{
    ChunkEmbeddingUpsertRequest, KnowledgeEmbeddingStore, KnowledgeEmbeddingStoreError,
};
use sdkwork_knowledgebase_agent_provider::serialize_embedding_vector;
use sqlx::AnyPool;
use std::sync::Arc;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const DEFAULT_PROVIDER_ID: &str = "provider.model.sdkwork-claw-router";
const DEFAULT_EMBEDDING_MODEL: &str = "openai/text-embedding-3-small";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SqliteKnowledgeEmbeddingStoreError {
    #[error("knowledge embedding store internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeEmbeddingStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeEmbeddingStore {
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

    pub async fn upsert_chunk_embedding(
        &self,
        request: ChunkEmbeddingUpsertRequest,
    ) -> Result<(), SqliteKnowledgeEmbeddingStoreError> {
        ensure_tenant_scope(self.tenant_id, request.tenant_id)?;
        if request.vector.is_empty() {
            return Err(SqliteKnowledgeEmbeddingStoreError::Internal(
                "embedding vector must not be empty".to_string(),
            ));
        }

        let tenant_id = to_i64("tenant_id", request.tenant_id)?;
        let index_id = to_i64("index_id", request.index_id)?;
        let chunk_id = to_i64("chunk_id", request.chunk_id)?;
        let dimension = i64::try_from(request.vector.len()).map_err(|_| {
            SqliteKnowledgeEmbeddingStoreError::Internal("embedding dimension overflow".to_string())
        })?;
        let vector_json = serialize_embedding_vector(&request.vector)
            .map_err(SqliteKnowledgeEmbeddingStoreError::Internal)?;
        let provider_id = request
            .provider_id
            .unwrap_or_else(|| DEFAULT_PROVIDER_ID.to_string());
        let model = request
            .model
            .unwrap_or_else(|| DEFAULT_EMBEDDING_MODEL.to_string());
        let embedding_hash = format!("sha256:chunk:{chunk_id}:index:{index_id}");
        let vector_ref = format!("inline://vector_json/{chunk_id}");
        let now = now_rfc3339()?;

        let existing = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM kb_embedding
            WHERE tenant_id = $1 AND index_id = $2 AND chunk_id = $3
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(index_id)
        .bind(chunk_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if let Some(existing_id) = existing {
            sqlx::query(
                r#"
                UPDATE kb_embedding
                SET embedding_hash = $1, vector_ref = $2, vector_json = $3, dimension = $4,
                    provider_id = $5, model = $6, status = $7, updated_at = $8, version = version + 1
                WHERE tenant_id = $9 AND id = $10
                "#,
            )
            .bind(embedding_hash)
            .bind(vector_ref)
            .bind(vector_json)
            .bind(dimension)
            .bind(provider_id)
            .bind(model)
            .bind(ACTIVE_STATUS)
            .bind(now)
            .bind(tenant_id)
            .bind(existing_id)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
            return Ok(());
        }

        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        sqlx::query(
            r#"
            INSERT INTO kb_embedding (
                id, uuid, tenant_id, index_id, chunk_id, embedding_hash, vector_ref, vector_json,
                dimension, provider_id, model, metadata, status, created_at, updated_at, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NULL, $12, $13, $14, 0)
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(index_id)
        .bind(chunk_id)
        .bind(embedding_hash)
        .bind(vector_ref)
        .bind(vector_json)
        .bind(dimension)
        .bind(provider_id)
        .bind(model)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;

        Ok(())
    }

    pub async fn load_chunk_content(
        &self,
        chunk_id: u64,
    ) -> Result<Option<String>, SqliteKnowledgeEmbeddingStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let chunk_id = to_i64("chunk_id", chunk_id)?;
        let content = sqlx::query_scalar::<_, String>(
            r#"
            SELECT content_text
            FROM kb_chunk
            WHERE tenant_id = $1 AND id = $2 AND status = $3
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(chunk_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        Ok(content)
    }

    pub async fn list_active_chunk_ids_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<u64>, SqliteKnowledgeEmbeddingStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let rows = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM kb_chunk
            WHERE tenant_id = $1 AND space_id = $2 AND status = $3
            ORDER BY id ASC
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.into_iter()
            .map(|value| {
                u64::try_from(value).map_err(|_| {
                    SqliteKnowledgeEmbeddingStoreError::Internal("chunk id overflow".to_string())
                })
            })
            .collect()
    }
}

fn ensure_tenant_scope(
    expected: u64,
    actual: u64,
) -> Result<(), SqliteKnowledgeEmbeddingStoreError> {
    if expected != actual {
        return Err(SqliteKnowledgeEmbeddingStoreError::Internal(
            "tenant_id is out of store scope".to_string(),
        ));
    }
    Ok(())
}

fn to_i64(field: &str, value: u64) -> Result<i64, SqliteKnowledgeEmbeddingStoreError> {
    i64::try_from(value).map_err(|_| {
        SqliteKnowledgeEmbeddingStoreError::Internal(format!("{field} is out of i64 range"))
    })
}

fn now_rfc3339() -> Result<String, SqliteKnowledgeEmbeddingStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| SqliteKnowledgeEmbeddingStoreError::Internal(error.to_string()))
}

fn id_error(error: crate::id::KnowledgeIdGeneratorError) -> SqliteKnowledgeEmbeddingStoreError {
    SqliteKnowledgeEmbeddingStoreError::Internal(error.to_string())
}

fn sqlx_error(error: sqlx::Error) -> SqliteKnowledgeEmbeddingStoreError {
    SqliteKnowledgeEmbeddingStoreError::Internal(error.to_string())
}

#[async_trait]
impl KnowledgeEmbeddingStore for SqliteKnowledgeEmbeddingStore {
    async fn upsert_chunk_embedding(
        &self,
        request: ChunkEmbeddingUpsertRequest,
    ) -> Result<(), KnowledgeEmbeddingStoreError> {
        SqliteKnowledgeEmbeddingStore::upsert_chunk_embedding(self, request)
            .await
            .map_err(|error| KnowledgeEmbeddingStoreError::Internal(error.to_string()))
    }

    async fn load_chunk_content(
        &self,
        chunk_id: u64,
    ) -> Result<Option<String>, KnowledgeEmbeddingStoreError> {
        SqliteKnowledgeEmbeddingStore::load_chunk_content(self, chunk_id)
            .await
            .map_err(|error| KnowledgeEmbeddingStoreError::Internal(error.to_string()))
    }

    async fn list_active_chunk_ids_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<u64>, KnowledgeEmbeddingStoreError> {
        SqliteKnowledgeEmbeddingStore::list_active_chunk_ids_for_space(self, space_id)
            .await
            .map_err(|error| KnowledgeEmbeddingStoreError::Internal(error.to_string()))
    }
}
