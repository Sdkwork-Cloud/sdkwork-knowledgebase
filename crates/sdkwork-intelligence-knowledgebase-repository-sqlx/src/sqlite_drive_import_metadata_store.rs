use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::{
    DriveImportMetadataStore, DriveImportMetadataStoreError, PrepareDriveImportMetadataRecord,
    PreparedDriveImportMetadata,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_ingestion_job_store::{
    CreateIngestionJobRecord, DriveImportJobLinkage,
};
use sdkwork_knowledgebase_contract::document::{KnowledgeDocument, KnowledgeDocumentVersion};
use sdkwork_knowledgebase_contract::ingest::{IngestionJob, IngestionJobState};
use sdkwork_knowledgebase_contract::source::KnowledgeSource;
use sqlx::{any::AnyRow, Any, AnyPool, Row, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::id::{next_i64_id, KnowledgeIdGenerator};

use crate::sqlite_knowledge_document_metadata_transaction::{
    bind_document_current_version_in_transaction, create_or_get_document_in_transaction,
    create_or_get_document_version_in_transaction, create_or_get_object_ref_in_transaction,
    create_or_get_source_in_transaction, document_from_row, document_version_from_row, from_i64,
    id_gen_error, now_rfc3339, source_from_row, sqlx_error, to_i64,
    METADATA_ACTIVE_STATUS as ACTIVE_STATUS, METADATA_INITIAL_VERSION as INITIAL_VERSION,
};

#[derive(Debug, Clone)]
pub struct SqliteDriveImportMetadataStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteDriveImportMetadataStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
        Self::with_id_generator(pool, tenant_id, crate::default_knowledge_id_generator())
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

    async fn validate_idempotency(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<(), DriveImportMetadataStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let row = sqlx::query(
            r#"
            SELECT id, space_id, job_type, idempotency_key, state, error_detail, metadata
            FROM kb_ingestion_job
            WHERE tenant_id = $1 AND space_id = $2 AND idempotency_key = $3 AND status = $4
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.idempotency_key)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        let Some(row) = row else {
            return Ok(());
        };

        validate_existing_job_idempotency(&row, &record)?;
        Ok(())
    }

    async fn prepare(
        &self,
        record: PrepareDriveImportMetadataRecord,
    ) -> Result<PreparedDriveImportMetadata, DriveImportMetadataStoreError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;

        let (job, job_metadata) = create_or_get_job_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &record.job,
        )
        .await?;

        if let Some(linkage) = parse_drive_import_linkage(job_metadata.as_deref())? {
            let prepared =
                load_prepared_from_linkage(&mut transaction, self.tenant_id, job, linkage).await?;
            transaction
                .commit()
                .await
                .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;
            return Ok(prepared);
        }

        let object_ref = create_or_get_object_ref_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &record.object_ref,
        )
        .await?;

        let source = create_or_get_source_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &record.source,
        )
        .await?;

        let mut document_record = record.document.clone();
        document_record.source_id = Some(source.id);
        let document = create_or_get_document_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &document_record,
        )
        .await?;

        let mut version_record = record.version.clone();
        version_record.document_id = document.id;
        version_record.original_object_ref_id = object_ref.id;
        let version = create_or_get_document_version_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &version_record,
        )
        .await?;

        let mut document = document;
        document.current_version_id = Some(version.id);
        bind_document_current_version_in_transaction(
            &mut transaction,
            self.tenant_id,
            document.id,
            version.id,
            version.version_no,
        )
        .await?;

        attach_drive_import_linkage_in_transaction(
            &mut transaction,
            self.tenant_id,
            job.id,
            job_metadata.as_deref(),
            DriveImportJobLinkage {
                source_id: source.id,
                document_id: document.id,
                document_version_id: version.id,
                original_object_ref: object_ref.clone(),
            },
        )
        .await?;

        transaction
            .commit()
            .await
            .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;

        Ok(PreparedDriveImportMetadata {
            job,
            source,
            document,
            version,
            original_object_ref: object_ref,
        })
    }
}

#[async_trait]
impl DriveImportMetadataStore for SqliteDriveImportMetadataStore {
    async fn validate_drive_import_idempotency(
        &self,
        record: CreateIngestionJobRecord,
    ) -> Result<(), DriveImportMetadataStoreError> {
        self.validate_idempotency(record).await
    }

    async fn create_or_prepare_drive_import_metadata(
        &self,
        record: PrepareDriveImportMetadataRecord,
    ) -> Result<PreparedDriveImportMetadata, DriveImportMetadataStoreError> {
        self.prepare(record).await
    }
}

async fn load_prepared_from_linkage(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    job: IngestionJob,
    linkage: DriveImportJobLinkage,
) -> Result<PreparedDriveImportMetadata, DriveImportMetadataStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let source = load_source_by_id(transaction, tenant_id, linkage.source_id).await?;
    let document = load_document_by_id(transaction, tenant_id, linkage.document_id).await?;
    let version =
        load_document_version_by_id(transaction, tenant_id, linkage.document_version_id).await?;
    Ok(PreparedDriveImportMetadata {
        job,
        source,
        document,
        version,
        original_object_ref: linkage.original_object_ref,
    })
}

async fn create_or_get_job_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    record: &CreateIngestionJobRecord,
) -> Result<(IngestionJob, Option<String>), DriveImportMetadataStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let space_id = to_i64("space_id", record.space_id)?;
    let id = next_i64_id(id_generator).map_err(id_gen_error)?;
    let now = now_rfc3339()?;
    let metadata = job_metadata_to_json(record.idempotency_fingerprint_sha256_hex.as_deref())?;
    let row = sqlx::query(
        r#"
        INSERT INTO kb_ingestion_job (
            id, uuid, tenant_id, space_id, job_type, state, priority, progress,
            idempotency_key, metadata, status, created_at, updated_at, version
        )
        VALUES ($1, $2, $3, $4, $5, $6, 0, 0, $7, $8, $9, $10, $11, $12)
        ON CONFLICT(tenant_id, space_id, idempotency_key) DO NOTHING
        RETURNING id, space_id, job_type, idempotency_key, state, error_detail, metadata
        "#,
    )
    .bind(id)
    .bind(Uuid::new_v4().to_string())
    .bind(tenant_id)
    .bind(space_id)
    .bind(&record.source_type)
    .bind(ingestion_state_code(IngestionJobState::Queued))
    .bind(&record.idempotency_key)
    .bind(metadata.clone())
    .bind(ACTIVE_STATUS)
    .bind(now.clone())
    .bind(now)
    .bind(INITIAL_VERSION)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    if let Some(row) = row {
        let job = job_from_row(&row)?;
        let metadata: Option<String> = row.try_get("metadata").map_err(sqlx_error)?;
        return Ok((job, metadata));
    }

    let row = sqlx::query(
        r#"
        SELECT id, space_id, job_type, idempotency_key, state, error_detail, metadata
        FROM kb_ingestion_job
        WHERE tenant_id = $1 AND space_id = $2 AND idempotency_key = $3 AND status = $4
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(space_id)
    .bind(&record.idempotency_key)
    .bind(ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    validate_existing_job_idempotency(&row, record)?;
    let metadata: Option<String> = row.try_get("metadata").map_err(sqlx_error)?;
    Ok((job_from_row(&row)?, metadata))
}

async fn attach_drive_import_linkage_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    job_id: u64,
    existing_metadata: Option<&str>,
    linkage: DriveImportJobLinkage,
) -> Result<(), DriveImportMetadataStoreError> {
    let metadata = merge_drive_import_linkage_metadata(existing_metadata, &linkage)?;
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let job_id = to_i64("job_id", job_id)?;
    let now = now_rfc3339()?;
    let updated = sqlx::query(
        r#"
        UPDATE kb_ingestion_job
        SET metadata = $1, updated_at = CAST($2 AS TIMESTAMP), version = version + 1
        WHERE tenant_id = $3 AND id = $4 AND status = $5
        "#,
    )
    .bind(metadata)
    .bind(now)
    .bind(tenant_id)
    .bind(job_id)
    .bind(ACTIVE_STATUS)
    .execute(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    if updated.rows_affected() == 0 {
        return Err(DriveImportMetadataStoreError::Internal(format!(
            "ingestion job not found: {job_id}"
        )));
    }
    Ok(())
}

async fn load_source_by_id(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    source_id: u64,
) -> Result<KnowledgeSource, DriveImportMetadataStoreError> {
    let source_id = to_i64("source_id", source_id)?;
    let row = sqlx::query(
        r#"
        SELECT id, space_id, source_type, provider, drive_bucket, drive_prefix, metadata
        FROM kb_source
        WHERE tenant_id = $1 AND id = $2 AND status = $3
        "#,
    )
    .bind(tenant_id)
    .bind(source_id)
    .bind(ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    source_from_row(&row)
}

async fn load_document_by_id(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    document_id: u64,
) -> Result<KnowledgeDocument, DriveImportMetadataStoreError> {
    let document_id = to_i64("document_id", document_id)?;
    let row = sqlx::query(
        r#"
        SELECT id, space_id, collection_id, source_id, original_file_drive_node_id, title, mime_type, language,
               current_version_id, visibility, content_state, index_state
        FROM kb_document
        WHERE tenant_id = $1 AND id = $2 AND status = $3
        "#,
    )
    .bind(tenant_id)
    .bind(document_id)
    .bind(ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    document_from_row(&row)
}

async fn load_document_version_by_id(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: i64,
    version_id: u64,
) -> Result<KnowledgeDocumentVersion, DriveImportMetadataStoreError> {
    let version_id = to_i64("version_id", version_id)?;
    let row = sqlx::query(
        r#"
        SELECT id, document_id, version_no, original_object_ref_id, checksum_sha256_hex,
               size_bytes, mime_type, parse_state, index_state
        FROM kb_document_version
        WHERE tenant_id = $1 AND id = $2 AND status = $3
        "#,
    )
    .bind(tenant_id)
    .bind(version_id)
    .bind(ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    document_version_from_row(&row)
}

fn validate_existing_job_idempotency(
    row: &AnyRow,
    record: &CreateIngestionJobRecord,
) -> Result<(), DriveImportMetadataStoreError> {
    let existing_job_type: String = row.try_get("job_type").map_err(sqlx_error)?;
    if existing_job_type != record.source_type {
        return Err(DriveImportMetadataStoreError::Conflict(
            "idempotency_key is already used for a different job_type".to_string(),
        ));
    }
    if let Some(expected_fingerprint) = &record.idempotency_fingerprint_sha256_hex {
        let existing_fingerprint = job_metadata_fingerprint(row)?;
        if existing_fingerprint.as_deref() != Some(expected_fingerprint.as_str()) {
            return Err(DriveImportMetadataStoreError::Conflict(
                "idempotency_key is already used for a different request".to_string(),
            ));
        }
    }
    Ok(())
}

fn job_metadata_to_json(
    idempotency_fingerprint_sha256_hex: Option<&str>,
) -> Result<Option<String>, DriveImportMetadataStoreError> {
    let Some(fingerprint) = idempotency_fingerprint_sha256_hex else {
        return Ok(None);
    };
    if fingerprint.len() != 64 || !fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DriveImportMetadataStoreError::InvalidRequest(
            "idempotency_fingerprint_sha256_hex must be a 64-char hex digest".to_string(),
        ));
    }
    Ok(Some(
        serde_json::json!({
            "idempotency_fingerprint_sha256_hex": fingerprint,
        })
        .to_string(),
    ))
}

fn job_metadata_fingerprint(row: &AnyRow) -> Result<Option<String>, DriveImportMetadataStoreError> {
    let metadata_text: Option<String> = row.try_get("metadata").map_err(sqlx_error)?;
    let Some(metadata_text) = metadata_text else {
        return Ok(None);
    };
    let metadata: serde_json::Value = serde_json::from_str(&metadata_text)
        .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;
    Ok(metadata
        .get("idempotency_fingerprint_sha256_hex")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string))
}

fn merge_drive_import_linkage_metadata(
    existing: Option<&str>,
    linkage: &DriveImportJobLinkage,
) -> Result<String, DriveImportMetadataStoreError> {
    let mut metadata = existing
        .map(serde_json::from_str::<serde_json::Value>)
        .transpose()
        .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?
        .unwrap_or_else(|| serde_json::json!({}));
    let object = serde_json::to_value(&linkage.original_object_ref)
        .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;
    metadata["drive_import"] = serde_json::json!({
        "source_id": linkage.source_id,
        "document_id": linkage.document_id,
        "document_version_id": linkage.document_version_id,
        "original_object_ref": object,
    });
    serde_json::to_string(&metadata)
        .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))
}

fn parse_drive_import_linkage(
    metadata_json: Option<&str>,
) -> Result<Option<DriveImportJobLinkage>, DriveImportMetadataStoreError> {
    let Some(metadata_json) = metadata_json else {
        return Ok(None);
    };
    let metadata: serde_json::Value = serde_json::from_str(metadata_json)
        .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))?;
    let Some(linkage) = metadata.get("drive_import") else {
        return Ok(None);
    };
    let read_u64 = |field: &str| -> Result<u64, DriveImportMetadataStoreError> {
        linkage
            .get(field)
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                DriveImportMetadataStoreError::Internal(format!(
                    "drive_import metadata missing {field}"
                ))
            })
    };
    let original_object_ref = linkage
        .get("original_object_ref")
        .ok_or_else(|| {
            DriveImportMetadataStoreError::Internal(
                "drive_import metadata missing original_object_ref".to_string(),
            )
        })
        .and_then(|value| {
            serde_json::from_value(value.clone())
                .map_err(|error| DriveImportMetadataStoreError::Internal(error.to_string()))
        })?;
    Ok(Some(DriveImportJobLinkage {
        source_id: read_u64("source_id")?,
        document_id: read_u64("document_id")?,
        document_version_id: read_u64("document_version_id")?,
        original_object_ref,
    }))
}

fn job_from_row(row: &AnyRow) -> Result<IngestionJob, DriveImportMetadataStoreError> {
    let state_code: i64 = row.try_get("state").map_err(sqlx_error)?;
    Ok(IngestionJob {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        space_id: from_i64("space_id", row.try_get("space_id").map_err(sqlx_error)?)?,
        source_type: row.try_get("job_type").map_err(sqlx_error)?,
        idempotency_key: row.try_get("idempotency_key").map_err(sqlx_error)?,
        state: ingestion_state_from_code(state_code)?,
        error_message: row.try_get("error_detail").map_err(sqlx_error)?,
    })
}

fn ingestion_state_code(state: IngestionJobState) -> i64 {
    match state {
        IngestionJobState::Queued => 0,
        IngestionJobState::Running => 1,
        IngestionJobState::Succeeded => 2,
        IngestionJobState::Failed => 3,
        IngestionJobState::Cancelled => 4,
    }
}

fn ingestion_state_from_code(
    code: i64,
) -> Result<IngestionJobState, DriveImportMetadataStoreError> {
    match code {
        0 => Ok(IngestionJobState::Queued),
        1 => Ok(IngestionJobState::Running),
        2 => Ok(IngestionJobState::Succeeded),
        3 => Ok(IngestionJobState::Failed),
        4 => Ok(IngestionJobState::Cancelled),
        _ => Err(DriveImportMetadataStoreError::Internal(format!(
            "invalid ingestion job state code: {code}"
        ))),
    }
}
