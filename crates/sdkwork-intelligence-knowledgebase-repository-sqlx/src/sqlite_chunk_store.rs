use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_chunk_store::{
    CreateKnowledgeChunkRecord, KnowledgeChunkStore, KnowledgeChunkStoreError,
};
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::chunk_transaction::replace_version_chunks_with_pool;
use crate::id::{default_knowledge_id_generator, KnowledgeIdGenerator};
use crate::keyword_search::KeywordSearchBackend;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeChunkStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
    keyword_backend: KeywordSearchBackend,
}

impl SqliteKnowledgeChunkStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_keyword_backend(
            pool,
            tenant_id,
            KeywordSearchBackend::SqliteFts5,
            default_knowledge_id_generator(),
        )
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self::with_keyword_backend(
            pool,
            tenant_id,
            KeywordSearchBackend::SqliteFts5,
            id_generator,
        )
    }

    pub fn with_keyword_backend(
        pool: AnyPool,
        tenant_id: u64,
        keyword_backend: KeywordSearchBackend,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            id_generator,
            keyword_backend,
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
        replace_version_chunks_with_pool(
            &self.pool,
            self.tenant_id,
            &self.id_generator,
            self.keyword_backend,
            document_version_id,
            chunks,
        )
        .await
    }

    async fn list_chunk_ids_for_document_version(
        &self,
        document_version_id: u64,
    ) -> Result<Vec<u64>, KnowledgeChunkStoreError> {
        let tenant_id = chunk_to_i64("tenant_id", self.tenant_id)?;
        let version_id = chunk_to_i64("document_version_id", document_version_id)?;
        const ACTIVE_STATUS: i64 = 1;
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

    async fn list_chunk_texts_for_document_version(
        &self,
        document_version_id: u64,
    ) -> Result<Vec<String>, KnowledgeChunkStoreError> {
        let tenant_id = chunk_to_i64("tenant_id", self.tenant_id)?;
        let version_id = chunk_to_i64("document_version_id", document_version_id)?;
        const ACTIVE_STATUS: i64 = 1;
        let rows = sqlx::query_scalar::<_, String>(
            r#"
            SELECT content_text
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

        Ok(rows)
    }

    async fn list_chunk_id_content_for_document_version(
        &self,
        document_version_id: u64,
    ) -> Result<Vec<(u64, String)>, KnowledgeChunkStoreError> {
        let tenant_id = chunk_to_i64("tenant_id", self.tenant_id)?;
        let version_id = chunk_to_i64("document_version_id", document_version_id)?;
        const ACTIVE_STATUS: i64 = 1;
        let rows = sqlx::query(
            r#"
            SELECT id, content_text
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
            .map(|row| {
                let chunk_id = row
                    .try_get::<i64, _>("id")
                    .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;
                let content = row
                    .try_get::<String, _>("content_text")
                    .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;
                let chunk_id = u64::try_from(chunk_id).map_err(|_| {
                    KnowledgeChunkStoreError::Internal("chunk id exceeds u64 range".to_string())
                })?;
                Ok((chunk_id, content))
            })
            .collect()
    }
}

fn chunk_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeChunkStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeChunkStoreError::InvalidRecord(format!("{field} exceeds sqlite integer range"))
    })
}
