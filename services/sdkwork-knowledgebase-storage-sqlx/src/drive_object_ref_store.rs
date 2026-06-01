use async_trait::async_trait;
use sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_product::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

const ACTIVE_STATUS: i64 = 1;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeDriveObjectRefStore {
    pool: SqlitePool,
    tenant_id: u64,
}

impl SqliteKnowledgeDriveObjectRefStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self { pool, tenant_id }
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
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                drive_provider_kind,
                drive_bucket,
                drive_object_key,
                drive_object_version,
                drive_etag,
                content_type,
                size_bytes,
                checksum_sha256_hex,
                object_role,
                access_mode
            FROM knowledge_drive_object_ref
            WHERE tenant_id = ?
              AND space_id = ?
              AND drive_bucket = ?
              AND drive_object_key = ?
              AND (drive_object_version IS ? OR drive_object_version = ?)
              AND object_role = ?
              AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.drive_bucket.clone())
        .bind(record.drive_object_key.clone())
        .bind(record.drive_object_version.clone())
        .bind(record.drive_object_version.clone())
        .bind(record.object_role.clone())
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if let Some(row) = row {
            return object_ref_from_row(&row);
        }

        self.insert_object_ref(record).await
    }
}

impl SqliteKnowledgeDriveObjectRefStore {
    async fn insert_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let size_bytes = to_i64("size_bytes", record.size_bytes)?;
        let now = now_rfc3339()?;

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_drive_object_ref (
                uuid,
                tenant_id,
                space_id,
                drive_provider_kind,
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)
            RETURNING
                id,
                space_id,
                drive_provider_kind,
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
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.drive_provider_kind)
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
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        object_ref_from_row(&row)
    }
}

fn object_ref_from_row(
    row: &SqliteRow,
) -> Result<KnowledgeDriveObjectRef, KnowledgeDriveObjectRefStoreError> {
    Ok(KnowledgeDriveObjectRef {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        space_id: from_i64("space_id", row.try_get("space_id").map_err(sqlx_error)?)?,
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
