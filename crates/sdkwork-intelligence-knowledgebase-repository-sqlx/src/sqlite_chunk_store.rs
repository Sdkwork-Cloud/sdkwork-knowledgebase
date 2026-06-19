use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_chunk_store::{
    CreateKnowledgeChunkRecord, KnowledgeChunkStore, KnowledgeChunkStoreError,
};
use sqlx::AnyPool;
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeChunkStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeChunkStore {
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

#[async_trait]
impl KnowledgeChunkStore for SqliteKnowledgeChunkStore {
    async fn replace_version_chunks(
        &self,
        document_version_id: u64,
        chunks: Vec<CreateKnowledgeChunkRecord>,
    ) -> Result<usize, KnowledgeChunkStoreError> {
        let tenant_id = chunk_to_i64("tenant_id", self.tenant_id)?;
        let version_id = chunk_to_i64("document_version_id", document_version_id)?;
        let now = chunk_now()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;

        sqlx::query(
            r#"
            DELETE FROM kb_chunk
            WHERE tenant_id = $1 AND document_version_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(version_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;

        for record in &chunks {
            if record.document_version_id != document_version_id {
                return Err(KnowledgeChunkStoreError::InvalidRecord(
                    "chunk document_version_id must match replace target".to_string(),
                ));
            }

            let id = next_i64_id(&self.id_generator).map_err(chunk_id_error)?;
            let space_id = chunk_to_i64("space_id", record.space_id)?;
            let collection_id = chunk_to_i64("collection_id", record.collection_id)?;
            let document_id = chunk_to_i64("document_id", record.document_id)?;

            sqlx::query(
                r#"
                INSERT INTO kb_chunk (
                    id, uuid, tenant_id, space_id, collection_id, document_id,
                    document_version_id, chunk_index, content_text, content_hash,
                    token_count, locator, status, created_at, updated_at, version
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(space_id)
            .bind(collection_id)
            .bind(document_id)
            .bind(version_id)
            .bind(i64::from(record.chunk_index))
            .bind(&record.content_text)
            .bind(&record.content_hash)
            .bind(record.token_count.map(i64::from))
            .bind(&record.locator)
            .bind(ACTIVE_STATUS)
            .bind(&now)
            .bind(&now)
            .bind(INITIAL_VERSION)
            .execute(&mut *tx)
            .await
            .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;

        Ok(chunks.len())
    }

    async fn list_chunk_ids_for_document_version(
        &self,
        document_version_id: u64,
    ) -> Result<Vec<u64>, KnowledgeChunkStoreError> {
        let tenant_id = chunk_to_i64("tenant_id", self.tenant_id)?;
        let version_id = chunk_to_i64("document_version_id", document_version_id)?;
        let rows = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM kb_chunk
            WHERE tenant_id = $1 AND document_version_id = $2 AND status = $3
            ORDER BY chunk_index ASC
            "#,
        )
        .bind(tenant_id)
        .bind(version_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;

        rows.into_iter()
            .map(|id| {
                u64::try_from(id).map_err(|_| {
                    KnowledgeChunkStoreError::Internal("chunk id exceeds u64 range".to_string())
                })
            })
            .collect()
    }
}

fn chunk_now() -> Result<String, KnowledgeChunkStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))
}

fn chunk_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeChunkStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeChunkStoreError::InvalidRecord(format!("{field} exceeds sqlite integer range"))
    })
}

fn chunk_id_error(error: crate::id::KnowledgeIdGeneratorError) -> KnowledgeChunkStoreError {
    KnowledgeChunkStoreError::Internal(error.to_string())
}
