use async_trait::async_trait;
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_product::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeDriveObjectRefStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeDriveObjectRefStore {
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
impl KnowledgeDriveObjectRefStore for SqliteKnowledgeDriveObjectRefStore {
    async fn create_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        self.insert_object_ref(record).await
    }

    async fn create_or_get_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        if let Some(object_ref) = self.insert_object_ref_if_absent(record.clone()).await? {
            return Ok(object_ref);
        }

        let object_ref = self.get_object_ref_by_locator(&record).await?;
        self.enrich_object_ref_drive_binding(object_ref, &record)
            .await
    }
}

impl SqliteKnowledgeDriveObjectRefStore {
    async fn get_object_ref_by_locator(
        &self,
        record: &CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                drive_provider_kind,
                drive_space_id,
                drive_node_id,
                logical_path,
                drive_bucket,
                drive_object_key,
                drive_object_version,
                drive_etag,
                content_type,
                size_bytes,
                checksum_sha256_hex,
                object_role,
                access_mode
            FROM kb_drive_object_ref
            WHERE tenant_id = ?
              AND space_id = ?
              AND drive_bucket = ?
              AND drive_object_key = ?
              AND COALESCE(drive_object_version, '') = COALESCE(?, '')
              AND object_role = ?
              AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.drive_bucket)
        .bind(&record.drive_object_key)
        .bind(&record.drive_object_version)
        .bind(&record.object_role)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        object_ref_from_row(&row)
    }

    async fn enrich_object_ref_drive_binding(
        &self,
        object_ref: KnowledgeDriveObjectRef,
        record: &CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        let should_update_drive_space =
            object_ref.drive_space_id.is_none() && record.drive_space_id.is_some();
        let should_update_drive_node =
            object_ref.drive_node_id.is_none() && record.drive_node_id.is_some();
        let should_update_logical_path =
            object_ref.logical_path.is_none() && record.logical_path.is_some();
        if !should_update_drive_space && !should_update_drive_node && !should_update_logical_path {
            return Ok(object_ref);
        }

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let object_ref_id = to_i64("object_ref_id", object_ref.id)?;
        let now = now_rfc3339()?;
        let row = sqlx::query(
            r#"
            UPDATE kb_drive_object_ref
            SET drive_space_id = COALESCE(drive_space_id, ?),
                drive_node_id = COALESCE(drive_node_id, ?),
                logical_path = COALESCE(logical_path, ?),
                updated_at = ?,
                version = version + 1
            WHERE tenant_id = ? AND id = ? AND status = ?
            RETURNING
                id,
                space_id,
                drive_provider_kind,
                drive_space_id,
                drive_node_id,
                logical_path,
                drive_bucket,
                drive_object_key,
                drive_object_version,
                drive_etag,
                content_type,
                size_bytes,
                checksum_sha256_hex,
                object_role,
                access_mode
            "#,
        )
        .bind(&record.drive_space_id)
        .bind(&record.drive_node_id)
        .bind(&record.logical_path)
        .bind(now)
        .bind(tenant_id)
        .bind(object_ref_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        object_ref_from_row(&row)
    }

    async fn insert_object_ref_if_absent(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<Option<KnowledgeDriveObjectRef>, KnowledgeDriveObjectRefStoreError> {
        self.insert_object_ref_with_conflict_clause(record, "ON CONFLICT DO NOTHING")
            .await
    }

    async fn insert_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        self.insert_object_ref_with_conflict_clause(record, "")
            .await?
            .ok_or_else(|| {
                KnowledgeDriveObjectRefStoreError::Internal(
                    "failed to insert drive object ref".to_string(),
                )
            })
    }

    async fn insert_object_ref_with_conflict_clause(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
        conflict_clause: &str,
    ) -> Result<Option<KnowledgeDriveObjectRef>, KnowledgeDriveObjectRefStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let size_bytes = to_i64("size_bytes", record.size_bytes)?;
        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        let now = now_rfc3339()?;
        let query = format!(
            r#"
            INSERT INTO kb_drive_object_ref (
                id,
                uuid,
                tenant_id,
                space_id,
                drive_provider_kind,
                drive_space_id,
                drive_node_id,
                logical_path,
                drive_bucket,
                drive_object_key,
                drive_object_version,
                drive_etag,
                content_type,
                size_bytes,
                checksum_sha256_hex,
                object_role,
                access_mode,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
            {conflict_clause}
            RETURNING
                id,
                space_id,
                drive_provider_kind,
                drive_space_id,
                drive_node_id,
                logical_path,
                drive_bucket,
                drive_object_key,
                drive_object_version,
                drive_etag,
                content_type,
                size_bytes,
                checksum_sha256_hex,
                object_role,
                access_mode
            "#
        );

        let row = sqlx::query(&query)
            .bind(id)
            .bind(Uuid::new_v4().to_string())
            .bind(tenant_id)
            .bind(space_id)
            .bind(record.drive_provider_kind)
            .bind(record.drive_space_id)
            .bind(record.drive_node_id)
            .bind(record.logical_path)
            .bind(record.drive_bucket)
            .bind(record.drive_object_key)
            .bind(record.drive_object_version)
            .bind(record.drive_etag)
            .bind(record.content_type)
            .bind(size_bytes)
            .bind(record.checksum_sha256_hex)
            .bind(record.object_role)
            .bind(record.access_mode)
            .bind(ACTIVE_STATUS)
            .bind(now.clone())
            .bind(now)
            .fetch_optional(&self.pool)
            .await
            .map_err(sqlx_error)?;

        row.map(|row| object_ref_from_row(&row)).transpose()
    }
}

fn object_ref_from_row(
    row: &SqliteRow,
) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
    Ok(KnowledgeDriveObjectRef {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        space_id: from_i64("space_id", row.try_get("space_id").map_err(sqlx_error)?)?,
        drive_space_id: row.try_get("drive_space_id").map_err(sqlx_error)?,
        drive_node_id: row.try_get("drive_node_id").map_err(sqlx_error)?,
        logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
        drive_provider_kind: row.try_get("drive_provider_kind").map_err(sqlx_error)?,
        drive_bucket: row.try_get("drive_bucket").map_err(sqlx_error)?,
        drive_object_key: row.try_get("drive_object_key").map_err(sqlx_error)?,
        drive_object_version: row.try_get("drive_object_version").map_err(sqlx_error)?,
        drive_etag: row.try_get("drive_etag").map_err(sqlx_error)?,
        content_type: row.try_get("content_type").map_err(sqlx_error)?,
        size_bytes: from_i64("size_bytes", row.try_get("size_bytes").map_err(sqlx_error)?)?,
        checksum_sha256_hex: row.try_get("checksum_sha256_hex").map_err(sqlx_error)?,
        object_role: row.try_get("object_role").map_err(sqlx_error)?,
        access_mode: row.try_get("access_mode").map_err(sqlx_error)?,
    })
}

fn now_rfc3339() -> Result<String, KnowledgeDriveObjectRefStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeDriveObjectRefStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeDriveObjectRefStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeDriveObjectRefStoreError::Internal(format!("{field} is out of range"))
    })
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeDriveObjectRefStoreError> {
    u64::try_from(value)
        .map_err(|_| KnowledgeDriveObjectRefStoreError::Internal(format!("{field} is negative")))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeDriveObjectRefStoreError {
    KnowledgeDriveObjectRefStoreError::Internal(error.to_string())
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeDriveObjectRefStoreError {
    KnowledgeDriveObjectRefStoreError::Internal(error.to_string())
}
