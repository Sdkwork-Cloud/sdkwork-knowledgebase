use sdkwork_intelligence_knowledgebase_service::ports::knowledge_chunk_store::{
    CreateKnowledgeChunkRecord, KnowledgeChunkStoreError,
};
use sqlx::{Any, AnyPool, QueryBuilder, Transaction};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::sql_timestamp::{push_sql_timestamp_bind, SqlTimestampDialect};
use crate::id::{next_i64_id, KnowledgeIdGenerator};
use crate::keyword_search::KeywordSearchBackend;

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;
const CHUNK_INSERT_BATCH_SIZE: usize = 50;

struct PreparedChunkRow {
    id: i64,
    uuid: String,
    space_id: i64,
    collection_id: i64,
    document_id: i64,
    chunk_index: i64,
    content_text: String,
    content_hash: String,
    token_count: Option<i64>,
    locator: Option<String>,
}

pub async fn replace_version_chunks_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    keyword_backend: KeywordSearchBackend,
    timestamp_dialect: SqlTimestampDialect,
    document_version_id: u64,
    chunks: &[CreateKnowledgeChunkRecord],
) -> Result<usize, KnowledgeChunkStoreError> {
    let tenant_id_i64 = chunk_to_i64("tenant_id", tenant_id)?;
    let version_id = chunk_to_i64("document_version_id", document_version_id)?;
    let now = chunk_now()?;
    let use_sqlite_fts = keyword_backend == KeywordSearchBackend::SqliteFts5;

    if use_sqlite_fts {
        sqlx::query(
            r#"
            DELETE FROM kb_chunk_fts
            WHERE chunk_id IN (
                SELECT id
                FROM kb_chunk
                WHERE tenant_id = $1 AND document_version_id = $2
            )
            "#,
        )
        .bind(tenant_id_i64)
        .bind(version_id)
        .execute(&mut **transaction)
        .await
        .map_err(chunk_internal_error)?;
    }

    sqlx::query(
        r#"
        DELETE FROM kb_chunk
        WHERE tenant_id = $1 AND document_version_id = $2
        "#,
    )
    .bind(tenant_id_i64)
    .bind(version_id)
    .execute(&mut **transaction)
    .await
    .map_err(chunk_internal_error)?;

    if chunks.is_empty() {
        return Ok(0);
    }

    let mut prepared = Vec::with_capacity(chunks.len());
    for record in chunks {
        if record.document_version_id != document_version_id {
            return Err(KnowledgeChunkStoreError::InvalidRecord(
                "chunk document_version_id must match replace target".to_string(),
            ));
        }

        prepared.push(PreparedChunkRow {
            id: next_i64_id(id_generator).map_err(chunk_id_error)?,
            uuid: Uuid::new_v4().to_string(),
            space_id: chunk_to_i64("space_id", record.space_id)?,
            collection_id: chunk_to_i64("collection_id", record.collection_id)?,
            document_id: chunk_to_i64("document_id", record.document_id)?,
            chunk_index: i64::from(record.chunk_index),
            content_text: record.content_text.clone(),
            content_hash: record.content_hash.clone(),
            token_count: record.token_count.map(i64::from),
            locator: record.locator.clone(),
        });
    }

    for batch in prepared.chunks(CHUNK_INSERT_BATCH_SIZE) {
        if use_sqlite_fts {
            bulk_insert_kb_chunks_sqlite(transaction, tenant_id_i64, version_id, &now, batch)
                .await?;
            bulk_insert_kb_chunk_fts(transaction, tenant_id_i64, batch).await?;
        } else {
            bulk_insert_kb_chunks_postgres(
                transaction,
                tenant_id_i64,
                version_id,
                timestamp_dialect,
                &now,
                batch,
            )
            .await?;
        }
    }

    Ok(chunks.len())
}

async fn bulk_insert_kb_chunks_sqlite(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    version_id: i64,
    now: &str,
    batch: &[PreparedChunkRow],
) -> Result<(), KnowledgeChunkStoreError> {
    let mut builder = QueryBuilder::new(
        r#"
        INSERT INTO kb_chunk (
            id, uuid, tenant_id, space_id, collection_id, document_id,
            document_version_id, chunk_index, content_text, content_hash,
            token_count, locator, status, created_at, updated_at, version
        )
        "#,
    );
    builder.push_values(batch, |mut row, chunk| {
        row.push_bind(chunk.id)
            .push_bind(chunk.uuid.as_str())
            .push_bind(tenant_id)
            .push_bind(chunk.space_id)
            .push_bind(chunk.collection_id)
            .push_bind(chunk.document_id)
            .push_bind(version_id)
            .push_bind(chunk.chunk_index)
            .push_bind(chunk.content_text.as_str())
            .push_bind(chunk.content_hash.as_str())
            .push_bind(chunk.token_count)
            .push_bind(chunk.locator.as_deref())
            .push_bind(ACTIVE_STATUS)
            .push_bind(now)
            .push_bind(now)
            .push_bind(INITIAL_VERSION);
    });

    builder
        .build()
        .execute(&mut **transaction)
        .await
        .map_err(chunk_internal_error)?;
    Ok(())
}

async fn bulk_insert_kb_chunks_postgres(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    version_id: i64,
    timestamp_dialect: SqlTimestampDialect,
    now: &str,
    batch: &[PreparedChunkRow],
) -> Result<(), KnowledgeChunkStoreError> {
    let mut builder = QueryBuilder::new(
        r#"
        INSERT INTO kb_chunk (
            id, uuid, tenant_id, space_id, collection_id, document_id,
            document_version_id, chunk_index, content_text, content_hash,
            token_count, locator, status, created_at, updated_at, version,
            search_vector
        )
        "#,
    );
    builder.push_values(batch, |mut row, chunk| {
        row.push_bind(chunk.id)
            .push_bind(chunk.uuid.as_str())
            .push_bind(tenant_id)
            .push_bind(chunk.space_id)
            .push_bind(chunk.collection_id)
            .push_bind(chunk.document_id)
            .push_bind(version_id)
            .push_bind(chunk.chunk_index)
            .push_bind(chunk.content_text.as_str())
            .push_bind(chunk.content_hash.as_str())
            .push_bind(chunk.token_count)
            .push_bind(chunk.locator.as_deref())
            .push_bind(ACTIVE_STATUS);
        push_sql_timestamp_bind(&mut row, timestamp_dialect, now);
        push_sql_timestamp_bind(&mut row, timestamp_dialect, now);
        row.push_bind(INITIAL_VERSION);
        row.push("to_tsvector('simple', ");
        row.push_bind_unseparated(chunk.content_text.as_str());
        row.push_unseparated(")");
    });

    builder
        .build()
        .execute(&mut **transaction)
        .await
        .map_err(chunk_internal_error)?;
    Ok(())
}

async fn bulk_insert_kb_chunk_fts(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    batch: &[PreparedChunkRow],
) -> Result<(), KnowledgeChunkStoreError> {
    let mut builder = QueryBuilder::new(
        r#"
        INSERT INTO kb_chunk_fts (
            content_text, chunk_id, tenant_id, space_id, document_id
        )
        "#,
    );
    builder.push_values(batch, |mut row, chunk| {
        row.push_bind(chunk.content_text.as_str())
            .push_bind(chunk.id)
            .push_bind(tenant_id)
            .push_bind(chunk.space_id)
            .push_bind(chunk.document_id);
    });

    builder
        .build()
        .execute(&mut **transaction)
        .await
        .map_err(chunk_internal_error)?;
    Ok(())
}

pub async fn replace_version_chunks_with_pool(
    pool: &AnyPool,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    keyword_backend: KeywordSearchBackend,
    document_version_id: u64,
    chunks: Vec<CreateKnowledgeChunkRecord>,
) -> Result<usize, KnowledgeChunkStoreError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;
    let count = replace_version_chunks_in_transaction(
        &mut tx,
        tenant_id,
        id_generator,
        keyword_backend,
        SqlTimestampDialect::default(),
        document_version_id,
        &chunks,
    )
    .await?;
    tx.commit()
        .await
        .map_err(|error| KnowledgeChunkStoreError::Internal(error.to_string()))?;
    Ok(count)
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

fn chunk_internal_error(error: sqlx::Error) -> KnowledgeChunkStoreError {
    KnowledgeChunkStoreError::Internal(error.to_string())
}
