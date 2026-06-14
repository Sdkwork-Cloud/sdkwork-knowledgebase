use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_store::{
    CreateKnowledgeDocumentRecord, KnowledgeDocumentIdentityScope, KnowledgeDocumentStore,
    KnowledgeDocumentStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_document_version_store::{
    CreateKnowledgeDocumentVersionRecord, KnowledgeDocumentVersionStore,
    KnowledgeDocumentVersionStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, CreateOrGetIngestionJobResult, IngestionJobStore,
    IngestionJobStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::document::{
    KnowledgeDocument, KnowledgeDocumentState, KnowledgeDocumentVersion,
    KnowledgeDocumentVersionState, KnowledgeDocumentVisibility,
};
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeSourceStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeSourceStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: SqlitePool,
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
impl KnowledgeSourceStore for SqliteKnowledgeSourceStore {
    async fn create_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        self.insert_source(record).await
    }

    async fn create_or_get_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        if let Some(source) = self.insert_source_if_absent(record.clone()).await? {
            return Ok(source);
        }

        self.get_source_by_identity(&record).await
    }
}

impl SqliteKnowledgeSourceStore {
    async fn get_source_by_identity(
        &self,
        record: &CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        let tenant_id = source_to_i64("tenant_id", self.tenant_id)?;
        let space_id = source_to_i64("space_id", record.space_id)?;
        let source_type_value = record.source_type.as_str().to_string();
        let row = sqlx::query(
            r#"
            SELECT id, space_id, source_type, provider, drive_bucket, drive_prefix
            FROM kb_source
            WHERE tenant_id = ?
              AND space_id = ?
              AND source_type = ?
              AND COALESCE(provider, '') = COALESCE(?, '')
              AND COALESCE(drive_bucket, '') = COALESCE(?, '')
              AND COALESCE(drive_prefix, '') = COALESCE(?, '')
              AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(source_type_value)
        .bind(&record.provider)
        .bind(&record.drive_bucket)
        .bind(&record.drive_prefix)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(source_sqlx_error)?;

        source_from_row(&row)
    }

    async fn insert_source_if_absent(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<Option<KnowledgeSource>, KnowledgeSourceStoreError> {
        self.insert_source_with_conflict_clause(record, "ON CONFLICT DO NOTHING")
            .await
    }

    async fn insert_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        self.insert_source_with_conflict_clause(record, "")
            .await?
            .ok_or_else(|| {
                KnowledgeSourceStoreError::Internal("failed to insert source".to_string())
            })
    }

    async fn insert_source_with_conflict_clause(
        &self,
        record: CreateKnowledgeSourceRecord,
        conflict_clause: &str,
    ) -> Result<Option<KnowledgeSource>, KnowledgeSourceStoreError> {
        let tenant_id = source_to_i64("tenant_id", self.tenant_id)?;
        let space_id = source_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(source_id_error)?;
        let now = source_now()?;
        let source_type_value = record.source_type.as_str().to_string();
        let query = format!(
            r#"
            INSERT INTO kb_source (
                id,
                uuid,
                tenant_id,
                space_id,
                source_type,
                provider,
                drive_bucket,
                drive_prefix,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            {conflict_clause}
            RETURNING id, space_id, source_type, provider, drive_bucket, drive_prefix
            "#
        );

        let row = sqlx::query(&query)
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(space_id)
            .bind(source_type_value)
            .bind(record.provider.clone())
            .bind(record.drive_bucket.clone())
            .bind(record.drive_prefix.clone())
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .bind(now)
            .bind(INITIAL_VERSION)
            .fetch_optional(&self.pool)
            .await
            .map_err(source_sqlx_error)?;

        row.map(|row| source_from_row(&row)).transpose()
    }
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeDocumentStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeDocumentStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: SqlitePool,
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
impl KnowledgeDocumentStore for SqliteKnowledgeDocumentStore {
    async fn create_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        self.insert_document(record).await
    }

    async fn create_or_get_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        if let Some(document) = self.insert_document_if_absent(record.clone()).await? {
            return Ok(document);
        }

        let document = self.get_document_by_identity(&record).await?;
        self.enrich_document_drive_node_binding(
            document,
            record.original_file_drive_node_id.as_deref(),
        )
        .await
    }
}

impl SqliteKnowledgeDocumentStore {
    async fn get_document_by_identity(
        &self,
        record: &CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        validate_document_identity(record)?;

        let tenant_id = document_to_i64("tenant_id", self.tenant_id)?;
        let space_id = document_to_i64("space_id", record.space_id)?;
        let collection_id = document_to_i64("collection_id", record.collection_id)?;
        let source_id = record
            .source_id
            .map(|value| document_to_i64("source_id", value))
            .transpose()?;
        let row = sqlx::query(
            r#"
            SELECT id, space_id, collection_id, source_id, original_file_drive_node_id, title, mime_type, language,
                   current_version_id, visibility, content_state, index_state
            FROM kb_document
            WHERE tenant_id = ?
              AND space_id = ?
              AND collection_id = ?
              AND identity_scope = ?
              AND (
                  (? = 'source_only' AND source_id = ?)
                  OR (
                      ? = 'source_and_original_drive_node'
                      AND (
                          (? IS NULL AND source_id IS NULL)
                          OR (? IS NOT NULL AND source_id = ?)
                      )
                      AND COALESCE(original_file_drive_node_id, '') = COALESCE(?, '')
                  )
              )
              AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(collection_id)
        .bind(record.identity_scope.as_str())
        .bind(record.identity_scope.as_str())
        .bind(source_id)
        .bind(record.identity_scope.as_str())
        .bind(source_id)
        .bind(source_id)
        .bind(source_id)
        .bind(&record.original_file_drive_node_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(document_sqlx_error)?;

        document_from_row(&row)
    }

    async fn enrich_document_drive_node_binding(
        &self,
        document: KnowledgeDocument,
        original_file_drive_node_id: Option<&str>,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        let Some(original_file_drive_node_id) = original_file_drive_node_id else {
            return Ok(document);
        };
        if document.original_file_drive_node_id.is_some() {
            return Ok(document);
        }

        let tenant_id = document_to_i64("tenant_id", self.tenant_id)?;
        let document_id = document_to_i64("document_id", document.id)?;
        let now = document_now()?;
        let row = sqlx::query(
            r#"
            UPDATE kb_document
            SET original_file_drive_node_id = COALESCE(original_file_drive_node_id, ?),
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
            RETURNING
                id,
                space_id,
                collection_id,
                source_id,
                identity_scope,
                original_file_drive_node_id,
                title,
                mime_type,
                language,
                current_version_id,
                visibility,
                content_state,
                index_state
            "#,
        )
        .bind(original_file_drive_node_id)
        .bind(now)
        .bind(tenant_id)
        .bind(document_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(document_sqlx_error)?;

        document_from_row(&row)
    }

    async fn insert_document_if_absent(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<Option<KnowledgeDocument>, KnowledgeDocumentStoreError> {
        self.insert_document_with_conflict_clause(record, "ON CONFLICT DO NOTHING")
            .await
    }

    async fn insert_document(
        &self,
        record: CreateKnowledgeDocumentRecord,
    ) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
        self.insert_document_with_conflict_clause(record, "")
            .await?
            .ok_or_else(|| {
                KnowledgeDocumentStoreError::Internal("failed to insert document".to_string())
            })
    }

    async fn insert_document_with_conflict_clause(
        &self,
        record: CreateKnowledgeDocumentRecord,
        conflict_clause: &str,
    ) -> Result<Option<KnowledgeDocument>, KnowledgeDocumentStoreError> {
        validate_document_identity(&record)?;

        let tenant_id = document_to_i64("tenant_id", self.tenant_id)?;
        let space_id = document_to_i64("space_id", record.space_id)?;
        let collection_id = document_to_i64("collection_id", record.collection_id)?;
        let source_id = record
            .source_id
            .map(|value| document_to_i64("source_id", value))
            .transpose()?;
        let generated_id = next_i64_id(&self.id_generator).map_err(document_id_error)?;
        let now = document_now()?;
        let query = format!(
            r#"
            INSERT INTO kb_document (
                id,
                uuid,
                tenant_id,
                space_id,
                collection_id,
                source_id,
                identity_scope,
                original_file_drive_node_id,
                title,
                mime_type,
                language,
                visibility,
                content_state,
                index_state,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            {conflict_clause}
            RETURNING
                id,
                space_id,
                collection_id,
                source_id,
                original_file_drive_node_id,
                title,
                mime_type,
                language,
                current_version_id,
                visibility,
                content_state,
                index_state
            "#
        );

        let row = sqlx::query(&query)
            .bind(generated_id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(space_id)
            .bind(collection_id)
            .bind(source_id)
            .bind(record.identity_scope.as_str())
            .bind(record.original_file_drive_node_id.clone())
            .bind(record.title.clone())
            .bind(record.mime_type.clone())
            .bind(record.language.clone())
            .bind(document_visibility_code(KnowledgeDocumentVisibility::Space))
            .bind(document_state_code(KnowledgeDocumentState::Ready))
            .bind(version_state_code(KnowledgeDocumentVersionState::Pending))
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .bind(now)
            .bind(INITIAL_VERSION)
            .fetch_optional(&self.pool)
            .await
            .map_err(document_sqlx_error)?;

        row.map(|row| document_from_row(&row)).transpose()
    }
}

fn validate_document_identity(
    record: &CreateKnowledgeDocumentRecord,
) -> Result<(), KnowledgeDocumentStoreError> {
    if record.identity_scope == KnowledgeDocumentIdentityScope::SourceOnly
        && record.source_id.is_none()
    {
        return Err(KnowledgeDocumentStoreError::InvalidRecord(
            "source_only document identity requires source_id".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeDocumentVersionStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeDocumentVersionStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: SqlitePool,
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
impl KnowledgeDocumentVersionStore for SqliteKnowledgeDocumentVersionStore {
    async fn create_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        self.insert_document_version(record).await
    }

    async fn create_or_get_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        if let Some(version) = self
            .insert_document_version_if_absent(record.clone())
            .await?
        {
            return Ok(version);
        }

        let version = self.get_document_version_by_identity(&record).await?;
        self.bind_document_current_version_if_latest(
            version.document_id,
            version.id,
            version.version_no,
            &version_now()?,
        )
        .await?;
        Ok(version)
    }
}

impl SqliteKnowledgeDocumentVersionStore {
    async fn get_document_version_by_identity(
        &self,
        record: &CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        let tenant_id = version_to_i64("tenant_id", self.tenant_id)?;
        let document_id = version_to_i64("document_id", record.document_id)?;
        let version_no = version_to_i64("version_no", record.version_no)?;
        let row = sqlx::query(
            r#"
            SELECT id, document_id, version_no, original_object_ref_id, checksum_sha256_hex,
                   size_bytes, mime_type, parse_state, index_state
            FROM kb_document_version
            WHERE tenant_id = ? AND document_id = ? AND version_no = ? AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(document_id)
        .bind(version_no)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(version_sqlx_error)?;

        document_version_from_row(&row)
    }

    async fn insert_document_version_if_absent(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<Option<KnowledgeDocumentVersion>, KnowledgeDocumentVersionStoreError> {
        self.insert_document_version_with_conflict_clause(record, "ON CONFLICT DO NOTHING")
            .await
    }

    async fn insert_document_version(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
    ) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
        self.insert_document_version_with_conflict_clause(record, "")
            .await?
            .ok_or_else(|| {
                KnowledgeDocumentVersionStoreError::Internal(
                    "failed to insert document version".to_string(),
                )
            })
    }

    async fn insert_document_version_with_conflict_clause(
        &self,
        record: CreateKnowledgeDocumentVersionRecord,
        conflict_clause: &str,
    ) -> Result<Option<KnowledgeDocumentVersion>, KnowledgeDocumentVersionStoreError> {
        let tenant_id = version_to_i64("tenant_id", self.tenant_id)?;
        let document_id = version_to_i64("document_id", record.document_id)?;
        let version_no = version_to_i64("version_no", record.version_no)?;
        let original_object_ref_id =
            version_to_i64("original_object_ref_id", record.original_object_ref_id)?;
        let size_bytes = version_to_i64("size_bytes", record.size_bytes)?;
        let generated_id = next_i64_id(&self.id_generator).map_err(version_id_error)?;
        let now = version_now()?;
        let query = format!(
            r#"
            INSERT INTO kb_document_version (
                id,
                uuid,
                tenant_id,
                document_id,
                version_no,
                original_object_ref_id,
                checksum_sha256_hex,
                size_bytes,
                mime_type,
                parse_state,
                index_state,
                submitted_at,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            {conflict_clause}
            RETURNING
                id,
                document_id,
                version_no,
                original_object_ref_id,
                checksum_sha256_hex,
                size_bytes,
                mime_type,
                parse_state,
                index_state
            "#
        );

        let row = sqlx::query(&query)
            .bind(generated_id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(document_id)
            .bind(version_no)
            .bind(original_object_ref_id)
            .bind(record.checksum_sha256_hex.clone())
            .bind(size_bytes)
            .bind(record.mime_type.clone())
            .bind(version_state_code(KnowledgeDocumentVersionState::Pending))
            .bind(version_state_code(KnowledgeDocumentVersionState::Pending))
            .bind(now.clone())
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .bind(now.clone())
            .bind(INITIAL_VERSION)
            .fetch_optional(&self.pool)
            .await
            .map_err(version_sqlx_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let version = document_version_from_row(&row)?;

        self.bind_document_current_version_if_latest(
            version.document_id,
            version.id,
            version.version_no,
            &now,
        )
        .await?;

        Ok(Some(version))
    }

    async fn bind_document_current_version_if_latest(
        &self,
        document_id: u64,
        version_id: u64,
        version_no: u64,
        now: &str,
    ) -> Result<(), KnowledgeDocumentVersionStoreError> {
        let tenant_id = version_to_i64("tenant_id", self.tenant_id)?;
        let document_id = version_to_i64("document_id", document_id)?;
        let version_id = version_to_i64("version_id", version_id)?;
        let version_no = version_to_i64("version_no", version_no)?;

        sqlx::query(
            r#"
            UPDATE kb_document
            SET current_version_id = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
              AND (
                  current_version_id IS NULL
                  OR NOT EXISTS (
                      SELECT 1
                      FROM kb_document_version current_version
                      WHERE current_version.tenant_id = kb_document.tenant_id
                        AND current_version.document_id = kb_document.id
                        AND current_version.id = kb_document.current_version_id
                        AND current_version.status = ?
                        AND current_version.version_no >= ?
                  )
              )
            "#,
        )
        .bind(version_id)
        .bind(now)
        .bind(tenant_id)
        .bind(document_id)
        .bind(ACTIVE_STATUS)
        .bind(ACTIVE_STATUS)
        .bind(version_no)
        .execute(&self.pool)
        .await
        .map_err(version_sqlx_error)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SqliteIngestionJobStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteIngestionJobStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, default_knowledge_id_generator())
    }

    pub fn with_id_generator(
        pool: SqlitePool,
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
impl IngestionJobStore for SqliteIngestionJobStore {
    async fn create_or_get_job(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<CreateOrGetIngestionJobResult, IngestionJobStoreError> {
        if let Some(job) = self.insert_job_if_absent(record.clone()).await? {
            return Ok(CreateOrGetIngestionJobResult { job, created: true });
        }

        let row = self.get_job_by_idempotency(&record).await?;
        validate_existing_job_idempotency(&row, &record)?;
        Ok(CreateOrGetIngestionJobResult {
            job: job_from_row(&row)?,
            created: false,
        })
    }

    async fn get_job(&self, job_id: u64) -> Result<IngestionJob, IngestionJobStoreError> {
        let tenant_id = job_to_i64("tenant_id", self.tenant_id)?;
        let job_id = job_to_i64("job_id", job_id)?;
        let row = sqlx::query(
            r#"
            SELECT id, space_id, job_type, idempotency_key, state, error_detail, metadata
            FROM kb_ingestion_job
            WHERE tenant_id = ? AND id = ? AND status = ?
            "#,
        )
        .bind(tenant_id)
        .bind(job_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| job_fetch_error(job_id, error))?;

        job_from_row(&row)
    }

    async fn update_job_state(
        &self,
        job_id: u64,
        state: IngestionJobState,
        error_message: Option<String>,
    ) -> Result<IngestionJob, IngestionJobStoreError> {
        let tenant_id = job_to_i64("tenant_id", self.tenant_id)?;
        let job_id = job_to_i64("job_id", job_id)?;
        let now = job_now()?;
        let row = sqlx::query(
            r#"
            UPDATE kb_ingestion_job
            SET state = ?, error_detail = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
            RETURNING id, space_id, job_type, idempotency_key, state, error_detail, metadata
            "#,
        )
        .bind(ingestion_state_code(state))
        .bind(error_message)
        .bind(now)
        .bind(tenant_id)
        .bind(job_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| job_fetch_error(job_id, error))?;

        job_from_row(&row)
    }
}

impl SqliteIngestionJobStore {
    async fn get_job_by_idempotency(
        &self,
        record: &CreateIngestionJobRecord,
    ) -> Result<SqliteRow, IngestionJobStoreError> {
        let tenant_id = job_to_i64("tenant_id", self.tenant_id)?;
        let space_id = job_to_i64("space_id", record.space_id)?;
        sqlx::query(
            r#"
            SELECT id, space_id, job_type, idempotency_key, state, error_detail, metadata
            FROM kb_ingestion_job
            WHERE tenant_id = ? AND space_id = ? AND idempotency_key = ? AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.idempotency_key)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(job_sqlx_error)
    }

    async fn insert_job_if_absent(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<Option<IngestionJob>, IngestionJobStoreError> {
        let tenant_id = job_to_i64("tenant_id", self.tenant_id)?;
        let space_id = job_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(job_id_error)?;
        let now = job_now()?;
        let metadata = job_metadata_to_json(record.idempotency_fingerprint_sha256_hex.as_deref())?;
        let row = sqlx::query(
            r#"
            INSERT INTO kb_ingestion_job (
                id,
                uuid,
                tenant_id,
                space_id,
                job_type,
                state,
                priority,
                progress,
                idempotency_key,
                metadata,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, 0, 0, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tenant_id, space_id, idempotency_key) DO NOTHING
            RETURNING id, space_id, job_type, idempotency_key, state, error_detail, metadata
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.source_type)
        .bind(ingestion_state_code(IngestionJobState::Queued))
        .bind(record.idempotency_key)
        .bind(metadata)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_optional(&self.pool)
        .await
        .map_err(job_sqlx_error)?;

        row.map(|row| job_from_row(&row)).transpose()
    }
}

fn job_from_row(row: &SqliteRow) -> Result<IngestionJob, IngestionJobStoreError> {
    let state_code: i64 = row.try_get("state").map_err(job_sqlx_error)?;
    Ok(IngestionJob {
        id: job_from_i64("id", row.try_get("id").map_err(job_sqlx_error)?)?,
        space_id: job_from_i64("space_id", row.try_get("space_id").map_err(job_sqlx_error)?)?,
        source_type: row.try_get("job_type").map_err(job_sqlx_error)?,
        idempotency_key: row.try_get("idempotency_key").map_err(job_sqlx_error)?,
        state: ingestion_state_from_code(state_code)?,
        error_message: row.try_get("error_detail").map_err(job_sqlx_error)?,
    })
}

fn validate_existing_job_idempotency(
    row: &SqliteRow,
    record: &CreateIngestionJobRecord,
) -> Result<(), IngestionJobStoreError> {
    let existing_job_type: String = row.try_get("job_type").map_err(job_sqlx_error)?;
    if existing_job_type != record.source_type {
        return Err(IngestionJobStoreError::Conflict(
            "idempotency_key is already used for a different job_type".to_string(),
        ));
    }

    if let Some(expected_fingerprint) = &record.idempotency_fingerprint_sha256_hex {
        let existing_fingerprint = job_metadata_fingerprint(row)?;
        if existing_fingerprint.as_deref() != Some(expected_fingerprint.as_str()) {
            return Err(IngestionJobStoreError::Conflict(
                "idempotency_key is already used for a different request".to_string(),
            ));
        }
    }

    Ok(())
}

fn job_metadata_to_json(
    idempotency_fingerprint_sha256_hex: Option<&str>,
) -> Result<Option<String>, IngestionJobStoreError> {
    let Some(fingerprint) = idempotency_fingerprint_sha256_hex else {
        return Ok(None);
    };
    if fingerprint.len() != 64 || !fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(IngestionJobStoreError::Internal(
            "idempotency_fingerprint_sha256_hex must be 64 hex characters".to_string(),
        ));
    }

    Ok(Some(
        serde_json::json!({
            "idempotency_fingerprint_sha256_hex": fingerprint,
        })
        .to_string(),
    ))
}

fn job_metadata_fingerprint(row: &SqliteRow) -> Result<Option<String>, IngestionJobStoreError> {
    let metadata: Option<String> = row.try_get("metadata").map_err(job_sqlx_error)?;
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let metadata: serde_json::Value = serde_json::from_str(&metadata)
        .map_err(|error| IngestionJobStoreError::Internal(error.to_string()))?;
    Ok(metadata
        .get("idempotency_fingerprint_sha256_hex")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string))
}

fn source_from_row(row: &SqliteRow) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
    Ok(KnowledgeSource {
        id: source_from_i64("id", row.try_get("id").map_err(source_sqlx_error)?)?,
        space_id: source_from_i64(
            "space_id",
            row.try_get("space_id").map_err(source_sqlx_error)?,
        )?,
        source_type: source_type_from_value(
            &row.try_get::<String, _>("source_type")
                .map_err(source_sqlx_error)?,
        )?,
        provider: row.try_get("provider").map_err(source_sqlx_error)?,
        drive_bucket: row.try_get("drive_bucket").map_err(source_sqlx_error)?,
        drive_prefix: row.try_get("drive_prefix").map_err(source_sqlx_error)?,
    })
}

fn document_from_row(row: &SqliteRow) -> Result<KnowledgeDocument, KnowledgeDocumentStoreError> {
    let visibility_code: i64 = row.try_get("visibility").map_err(document_sqlx_error)?;
    let content_state_code: i64 = row.try_get("content_state").map_err(document_sqlx_error)?;
    let index_state_code: i64 = row.try_get("index_state").map_err(document_sqlx_error)?;
    Ok(KnowledgeDocument {
        id: document_from_i64("id", row.try_get("id").map_err(document_sqlx_error)?)?,
        space_id: document_from_i64(
            "space_id",
            row.try_get("space_id").map_err(document_sqlx_error)?,
        )?,
        collection_id: document_from_i64(
            "collection_id",
            row.try_get("collection_id").map_err(document_sqlx_error)?,
        )?,
        source_id: row
            .try_get::<Option<i64>, _>("source_id")
            .map_err(document_sqlx_error)?
            .map(|value| document_from_i64("source_id", value))
            .transpose()?,
        original_file_drive_node_id: row
            .try_get("original_file_drive_node_id")
            .map_err(document_sqlx_error)?,
        title: row.try_get("title").map_err(document_sqlx_error)?,
        mime_type: row.try_get("mime_type").map_err(document_sqlx_error)?,
        language: row.try_get("language").map_err(document_sqlx_error)?,
        current_version_id: row
            .try_get::<Option<i64>, _>("current_version_id")
            .map_err(document_sqlx_error)?
            .map(|value| document_from_i64("current_version_id", value))
            .transpose()?,
        visibility: document_visibility_from_code(visibility_code)?,
        content_state: document_state_from_code(content_state_code)?,
        index_state: version_state_from_code_for_document(index_state_code)?,
    })
}

fn document_version_from_row(
    row: &SqliteRow,
) -> Result<KnowledgeDocumentVersion, KnowledgeDocumentVersionStoreError> {
    let parse_state_code: i64 = row.try_get("parse_state").map_err(version_sqlx_error)?;
    let index_state_code: i64 = row.try_get("index_state").map_err(version_sqlx_error)?;
    Ok(KnowledgeDocumentVersion {
        id: version_from_i64("id", row.try_get("id").map_err(version_sqlx_error)?)?,
        document_id: version_from_i64(
            "document_id",
            row.try_get("document_id").map_err(version_sqlx_error)?,
        )?,
        version_no: version_from_i64(
            "version_no",
            row.try_get("version_no").map_err(version_sqlx_error)?,
        )?,
        original_object_ref_id: version_from_i64(
            "original_object_ref_id",
            row.try_get("original_object_ref_id")
                .map_err(version_sqlx_error)?,
        )?,
        checksum_sha256_hex: row
            .try_get("checksum_sha256_hex")
            .map_err(version_sqlx_error)?,
        size_bytes: version_from_i64(
            "size_bytes",
            row.try_get("size_bytes").map_err(version_sqlx_error)?,
        )?,
        mime_type: row.try_get("mime_type").map_err(version_sqlx_error)?,
        parse_state: version_state_from_code(parse_state_code)?,
        index_state: version_state_from_code(index_state_code)?,
    })
}

fn document_visibility_code(value: KnowledgeDocumentVisibility) -> i64 {
    match value {
        KnowledgeDocumentVisibility::Private => 0,
        KnowledgeDocumentVisibility::Space => 1,
        KnowledgeDocumentVisibility::Organization => 2,
        KnowledgeDocumentVisibility::Public => 3,
    }
}

fn document_state_code(value: KnowledgeDocumentState) -> i64 {
    match value {
        KnowledgeDocumentState::Draft => 0,
        KnowledgeDocumentState::Ready => 1,
        KnowledgeDocumentState::Archived => 2,
        KnowledgeDocumentState::Deleted => 3,
    }
}

fn version_state_code(value: KnowledgeDocumentVersionState) -> i64 {
    match value {
        KnowledgeDocumentVersionState::Pending => 0,
        KnowledgeDocumentVersionState::Running => 1,
        KnowledgeDocumentVersionState::Succeeded => 2,
        KnowledgeDocumentVersionState::Failed => 3,
    }
}

fn source_type_from_value(value: &str) -> Result<KnowledgeSourceType, KnowledgeSourceStoreError> {
    match value {
        "upload" => Ok(KnowledgeSourceType::Upload),
        "drive_object" => Ok(KnowledgeSourceType::DriveObject),
        "drive_folder" => Ok(KnowledgeSourceType::DriveFolder),
        "url" => Ok(KnowledgeSourceType::Url),
        "connector" => Ok(KnowledgeSourceType::Connector),
        "api" => Ok(KnowledgeSourceType::Api),
        _ => Err(KnowledgeSourceStoreError::Internal(format!(
            "unknown knowledge source type: {value}"
        ))),
    }
}

fn document_visibility_from_code(
    code: i64,
) -> Result<KnowledgeDocumentVisibility, KnowledgeDocumentStoreError> {
    match code {
        0 => Ok(KnowledgeDocumentVisibility::Private),
        1 => Ok(KnowledgeDocumentVisibility::Space),
        2 => Ok(KnowledgeDocumentVisibility::Organization),
        3 => Ok(KnowledgeDocumentVisibility::Public),
        _ => Err(KnowledgeDocumentStoreError::Internal(format!(
            "unknown document visibility code: {code}"
        ))),
    }
}

fn document_state_from_code(
    code: i64,
) -> Result<KnowledgeDocumentState, KnowledgeDocumentStoreError> {
    match code {
        0 => Ok(KnowledgeDocumentState::Draft),
        1 => Ok(KnowledgeDocumentState::Ready),
        2 => Ok(KnowledgeDocumentState::Archived),
        3 => Ok(KnowledgeDocumentState::Deleted),
        _ => Err(KnowledgeDocumentStoreError::Internal(format!(
            "unknown document state code: {code}"
        ))),
    }
}

fn version_state_from_code_for_document(
    code: i64,
) -> Result<KnowledgeDocumentVersionState, KnowledgeDocumentStoreError> {
    match code {
        0 => Ok(KnowledgeDocumentVersionState::Pending),
        1 => Ok(KnowledgeDocumentVersionState::Running),
        2 => Ok(KnowledgeDocumentVersionState::Succeeded),
        3 => Ok(KnowledgeDocumentVersionState::Failed),
        _ => Err(KnowledgeDocumentStoreError::Internal(format!(
            "unknown document version state code: {code}"
        ))),
    }
}

fn version_state_from_code(
    code: i64,
) -> Result<KnowledgeDocumentVersionState, KnowledgeDocumentVersionStoreError> {
    match code {
        0 => Ok(KnowledgeDocumentVersionState::Pending),
        1 => Ok(KnowledgeDocumentVersionState::Running),
        2 => Ok(KnowledgeDocumentVersionState::Succeeded),
        3 => Ok(KnowledgeDocumentVersionState::Failed),
        _ => Err(KnowledgeDocumentVersionStoreError::Internal(format!(
            "unknown document version state code: {code}"
        ))),
    }
}

fn ingestion_state_code(value: IngestionJobState) -> i64 {
    match value {
        IngestionJobState::Queued => 0,
        IngestionJobState::Running => 1,
        IngestionJobState::Succeeded => 2,
        IngestionJobState::Failed => 3,
        IngestionJobState::Cancelled => 4,
    }
}

fn ingestion_state_from_code(code: i64) -> Result<IngestionJobState, IngestionJobStoreError> {
    match code {
        0 => Ok(IngestionJobState::Queued),
        1 => Ok(IngestionJobState::Running),
        2 => Ok(IngestionJobState::Succeeded),
        3 => Ok(IngestionJobState::Failed),
        4 => Ok(IngestionJobState::Cancelled),
        _ => Err(IngestionJobStoreError::Internal(format!(
            "unknown ingestion job state code: {code}"
        ))),
    }
}

fn source_now() -> Result<String, KnowledgeSourceStoreError> {
    now_rfc3339().map_err(KnowledgeSourceStoreError::Internal)
}

fn document_now() -> Result<String, KnowledgeDocumentStoreError> {
    now_rfc3339().map_err(KnowledgeDocumentStoreError::Internal)
}

fn version_now() -> Result<String, KnowledgeDocumentVersionStoreError> {
    now_rfc3339().map_err(KnowledgeDocumentVersionStoreError::Internal)
}

fn job_now() -> Result<String, IngestionJobStoreError> {
    now_rfc3339().map_err(IngestionJobStoreError::Internal)
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

fn source_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeSourceStoreError> {
    to_i64(field, value).map_err(KnowledgeSourceStoreError::Internal)
}

fn document_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeDocumentStoreError> {
    to_i64(field, value).map_err(KnowledgeDocumentStoreError::Internal)
}

fn version_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeDocumentVersionStoreError> {
    to_i64(field, value).map_err(KnowledgeDocumentVersionStoreError::Internal)
}

fn job_to_i64(field: &str, value: u64) -> Result<i64, IngestionJobStoreError> {
    to_i64(field, value).map_err(IngestionJobStoreError::Internal)
}

fn source_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeSourceStoreError> {
    from_i64(field, value).map_err(KnowledgeSourceStoreError::Internal)
}

fn document_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeDocumentStoreError> {
    from_i64(field, value).map_err(KnowledgeDocumentStoreError::Internal)
}

fn version_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeDocumentVersionStoreError> {
    from_i64(field, value).map_err(KnowledgeDocumentVersionStoreError::Internal)
}

fn job_from_i64(field: &str, value: i64) -> Result<u64, IngestionJobStoreError> {
    from_i64(field, value).map_err(IngestionJobStoreError::Internal)
}

fn to_i64(field: &str, value: u64) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("{field} is out of range"))
}

fn from_i64(field: &str, value: i64) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("{field} is negative"))
}

fn source_sqlx_error(error: sqlx::Error) -> KnowledgeSourceStoreError {
    KnowledgeSourceStoreError::Internal(error.to_string())
}

fn source_id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeSourceStoreError {
    KnowledgeSourceStoreError::Internal(error.to_string())
}

fn document_sqlx_error(error: sqlx::Error) -> KnowledgeDocumentStoreError {
    KnowledgeDocumentStoreError::Internal(error.to_string())
}

fn document_id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeDocumentStoreError {
    KnowledgeDocumentStoreError::Internal(error.to_string())
}

fn version_sqlx_error(error: sqlx::Error) -> KnowledgeDocumentVersionStoreError {
    KnowledgeDocumentVersionStoreError::Internal(error.to_string())
}

fn version_id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeDocumentVersionStoreError {
    KnowledgeDocumentVersionStoreError::Internal(error.to_string())
}

fn job_sqlx_error(error: sqlx::Error) -> IngestionJobStoreError {
    IngestionJobStoreError::Internal(error.to_string())
}

fn job_id_error(error: crate::KnowledgeIdGeneratorError) -> IngestionJobStoreError {
    IngestionJobStoreError::Internal(error.to_string())
}

fn job_fetch_error(job_id: i64, error: sqlx::Error) -> IngestionJobStoreError {
    if matches!(error, sqlx::Error::RowNotFound) {
        return IngestionJobStoreError::NotFound(job_id as u64);
    }
    job_sqlx_error(error)
}
