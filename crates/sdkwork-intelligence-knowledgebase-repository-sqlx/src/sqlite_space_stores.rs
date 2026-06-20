use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_bundle_file_store::{
    CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
    KnowledgeOkfBundleFileStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_knowledgebase_contract::okf_bundle_file::{KnowledgeOkfBundleFile, OkfBundleFileKind};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::space::{KnowledgeSpace, KnowledgeSpaceStatus};
use sqlx::{any::AnyRow, AnyPool, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeSpaceStore {
    pool: AnyPool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeSpaceStore {
    pub fn new(pool: AnyPool, tenant_id: u64, organization_id: u64) -> Self {
        Self::with_id_generator(
            pool,
            tenant_id,
            organization_id,
            default_knowledge_id_generator(),
        )
    }

    pub fn with_id_generator(
        pool: AnyPool,
        tenant_id: u64,
        organization_id: u64,
        id_generator: Arc<dyn KnowledgeIdGenerator>,
    ) -> Self {
        Self {
            pool,
            tenant_id,
            organization_id,
            id_generator,
        }
    }
}

#[async_trait]
impl KnowledgeSpaceStore for SqliteKnowledgeSpaceStore {
    async fn create_space(
        &self,
        record: CreateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let id = next_i64_id(&self.id_generator).map_err(space_id_error)?;
        let now = space_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_space (
                id,
                uuid,
                tenant_id,
                organization_id,
                name,
                description,
                drive_space_id,
                status,
                okf_bundle_initialized,
                knowledge_mode,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, uuid, name, description, drive_space_id, status, okf_bundle_initialized, knowledge_mode, knowledge_mode
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(organization_id)
        .bind(record.name)
        .bind(record.description)
        .bind(None::<String>)
        .bind(space_status_code(KnowledgeSpaceStatus::Active))
        .bind(bool_code(record.okf_bundle_initialized))
        .bind(space_knowledge_mode_code(record.knowledge_mode))
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(space_sqlx_error)?;

        space_from_row(&row)
    }

    async fn get_space(&self, space_id: u64) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let space_id_i64 = space_to_i64("space_id", space_id)?;

        let row = sqlx::query(
            r#"
            SELECT id, uuid, name, description, drive_space_id, status, okf_bundle_initialized, knowledge_mode
            FROM kb_space
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3 AND status = $4
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(space_id_i64)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| space_fetch_error(space_id, error))?;

        space_from_row(&row)
    }

    async fn mark_drive_space_bound(
        &self,
        space_id: u64,
        drive_space_id: String,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let space_id_i64 = space_to_i64("space_id", space_id)?;
        let drive_space_id = require_safe_drive_id(drive_space_id, "drive_space_id")?;
        let now = space_now()?;

        let row = sqlx::query(
            r#"
            UPDATE kb_space
            SET drive_space_id = $1, updated_at = $2, version = version + 1
            WHERE tenant_id = $3 AND organization_id = $4 AND id = $5 AND status = $6
            RETURNING id, uuid, name, description, drive_space_id, status, okf_bundle_initialized, knowledge_mode
            "#,
        )
        .bind(drive_space_id)
        .bind(now)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(space_id_i64)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| space_fetch_error(space_id, error))?;

        space_from_row(&row)
    }

    async fn mark_okf_bundle_initialized(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let space_id_i64 = space_to_i64("space_id", space_id)?;
        let now = space_now()?;

        let row = sqlx::query(
            r#"
            UPDATE kb_space
            SET okf_bundle_initialized = 1, updated_at = $1, version = version + 1
            WHERE tenant_id = $2 AND organization_id = $3 AND id = $4 AND status = $5
            RETURNING id, uuid, name, description, drive_space_id, status, okf_bundle_initialized, knowledge_mode
            "#,
        )
        .bind(now)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(space_id_i64)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| space_fetch_error(space_id, error))?;

        space_from_row(&row)
    }

    async fn mark_space_deleted(&self, space_id: u64) -> Result<(), KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let space_id_i64 = space_to_i64("space_id", space_id)?;
        let now = space_now()?;

        sqlx::query(
            r#"
            UPDATE kb_space
            SET status = $1, updated_at = $2, version = version + 1
            WHERE tenant_id = $3 AND organization_id = $4 AND id = $5 AND status = $6
            "#,
        )
        .bind(space_status_code(KnowledgeSpaceStatus::Deleted))
        .bind(now)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(space_id_i64)
        .bind(ACTIVE_STATUS)
        .execute(&self.pool)
        .await
        .map_err(space_sqlx_error)?;

        Ok(())
    }
}

impl SqliteKnowledgeSpaceStore {
    pub async fn find_first_okf_bundle_initialized_space(
        &self,
    ) -> Result<Option<KnowledgeSpace>, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let row = sqlx::query(
            r#"
            SELECT id, uuid, name, description, drive_space_id, status, okf_bundle_initialized, knowledge_mode
            FROM kb_space
            WHERE tenant_id = $1 AND organization_id = $2 AND status = $3 AND okf_bundle_initialized = 1
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(space_sqlx_error)?;

        row.map(|row| space_from_row(&row)).transpose()
    }
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeOkfBundleFileStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeOkfBundleFileStore {
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
impl KnowledgeOkfBundleFileStore for SqliteKnowledgeOkfBundleFileStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        self.insert_file_entry(record).await
    }

    async fn upsert_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        self.upsert_file_entry_record(record).await
    }
}

impl SqliteKnowledgeOkfBundleFileStore {
    async fn insert_file_entry(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        let tenant_id = okf_bundle_file_to_i64("tenant_id", self.tenant_id)?;
        let space_id = okf_bundle_file_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(okf_bundle_file_id_error)?;
        let now = okf_bundle_file_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_okf_bundle_file (
                id,
                uuid,
                tenant_id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING
                id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.logical_path)
        .bind(okf_bundle_file_type_code(record.file_kind))
        .bind(record.artifact_role)
        .bind(record.drive_bucket)
        .bind(record.drive_object_key)
        .bind(record.checksum_sha256_hex)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(okf_bundle_file_sqlx_error)?;

        okf_bundle_file_from_row(&row)
    }

    async fn upsert_file_entry_record(
        &self,
        record: CreateKnowledgeOkfBundleFileRecord,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        let tenant_id = okf_bundle_file_to_i64("tenant_id", self.tenant_id)?;
        let space_id = okf_bundle_file_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(okf_bundle_file_id_error)?;
        let now = okf_bundle_file_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_okf_bundle_file (
                id,
                uuid,
                tenant_id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT(tenant_id, space_id, logical_path)
            DO UPDATE SET
                file_kind = excluded.file_kind,
                artifact_role = excluded.artifact_role,
                drive_bucket = excluded.drive_bucket,
                drive_object_key = excluded.drive_object_key,
                checksum_sha256_hex = excluded.checksum_sha256_hex,
                status = excluded.status,
                updated_at = excluded.updated_at,
                version = kb_okf_bundle_file.version + 1
            RETURNING
                id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.logical_path)
        .bind(okf_bundle_file_type_code(record.file_kind))
        .bind(record.artifact_role)
        .bind(record.drive_bucket)
        .bind(record.drive_object_key)
        .bind(record.checksum_sha256_hex)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(okf_bundle_file_sqlx_error)?;

        okf_bundle_file_from_row(&row)
    }

    pub async fn list_file_entries(
        &self,
    ) -> Result<Vec<KnowledgeOkfBundleFile>, KnowledgeOkfBundleFileStoreError> {
        let tenant_id = okf_bundle_file_to_i64("tenant_id", self.tenant_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            FROM kb_okf_bundle_file
            WHERE tenant_id = $1 AND status = $2
            ORDER BY id ASC
            LIMIT 200
            "#,
        )
        .bind(tenant_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(okf_bundle_file_sqlx_error)?;

        rows.iter().map(okf_bundle_file_from_row).collect()
    }

    pub async fn get_file_entry_by_id(
        &self,
        entry_id: u64,
    ) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
        let tenant_id = okf_bundle_file_to_i64("tenant_id", self.tenant_id)?;
        let entry_id = okf_bundle_file_to_i64("entry_id", entry_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                logical_path,
                file_kind,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            FROM kb_okf_bundle_file
            WHERE tenant_id = $1 AND id = $2 AND status = $3
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(entry_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(okf_bundle_file_sqlx_error)?
        .ok_or_else(|| {
            KnowledgeOkfBundleFileStoreError::Internal(format!(
                "missing okf bundle file: {entry_id}"
            ))
        })?;

        okf_bundle_file_from_row(&row)
    }
}

fn space_from_row(row: &AnyRow) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
    let status_code: i64 = row.try_get("status").map_err(space_sqlx_error)?;
    let okf_bundle_initialized: i64 = row
        .try_get("okf_bundle_initialized")
        .map_err(space_sqlx_error)?;
    Ok(KnowledgeSpace {
        id: space_from_i64("id", row.try_get("id").map_err(space_sqlx_error)?)?,
        uuid: row.try_get("uuid").map_err(space_sqlx_error)?,
        name: row.try_get("name").map_err(space_sqlx_error)?,
        description: row.try_get("description").map_err(space_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(space_sqlx_error)?,
        status: space_status_from_code(status_code)?,
        okf_bundle_initialized: okf_bundle_initialized != 0,
        knowledge_mode: space_knowledge_mode_from_row(row)?,
    })
}

fn space_knowledge_mode_code(mode: KnowledgeAgentKnowledgeMode) -> &'static str {
    mode.as_str()
}

fn space_knowledge_mode_from_row(
    row: &AnyRow,
) -> Result<KnowledgeAgentKnowledgeMode, KnowledgeSpaceStoreError> {
    let value: Option<String> = row.try_get("knowledge_mode").map_err(space_sqlx_error)?;
    match value.as_deref().unwrap_or("okf_bundle") {
        "okf_bundle" => Ok(KnowledgeAgentKnowledgeMode::OkfBundle),
        "rag" => Ok(KnowledgeAgentKnowledgeMode::Rag),
        "external" => Ok(KnowledgeAgentKnowledgeMode::External),
        other => Err(KnowledgeSpaceStoreError::Internal(format!(
            "unsupported knowledge_mode value: {other}"
        ))),
    }
}

fn require_safe_drive_id(
    value: String,
    field_name: &str,
) -> Result<String, KnowledgeSpaceStoreError> {
    let value = value.trim().to_string();
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
    {
        return Err(KnowledgeSpaceStoreError::Internal(format!(
            "invalid {field_name}"
        )));
    }
    Ok(value)
}

fn okf_bundle_file_from_row(
    row: &AnyRow,
) -> Result<KnowledgeOkfBundleFile, KnowledgeOkfBundleFileStoreError> {
    let file_kind: String = row
        .try_get("file_kind")
        .map_err(okf_bundle_file_sqlx_error)?;
    Ok(KnowledgeOkfBundleFile {
        id: okf_bundle_file_from_i64("id", row.try_get("id").map_err(okf_bundle_file_sqlx_error)?)?,
        space_id: okf_bundle_file_from_i64(
            "space_id",
            row.try_get("space_id")
                .map_err(okf_bundle_file_sqlx_error)?,
        )?,
        logical_path: row
            .try_get("logical_path")
            .map_err(okf_bundle_file_sqlx_error)?,
        file_kind: okf_bundle_file_type_from_code(&file_kind)?,
        artifact_role: row
            .try_get("artifact_role")
            .map_err(okf_bundle_file_sqlx_error)?,
        drive_bucket: row
            .try_get("drive_bucket")
            .map_err(okf_bundle_file_sqlx_error)?,
        drive_object_key: row
            .try_get("drive_object_key")
            .map_err(okf_bundle_file_sqlx_error)?,
        checksum_sha256_hex: row
            .try_get("checksum_sha256_hex")
            .map_err(okf_bundle_file_sqlx_error)?,
        staged_import_root: None,
        import_id: None,
    })
}

fn space_status_code(value: KnowledgeSpaceStatus) -> i64 {
    match value {
        KnowledgeSpaceStatus::Active => 1,
        KnowledgeSpaceStatus::Archived => 2,
        KnowledgeSpaceStatus::Deleted => 3,
    }
}

fn space_status_from_code(code: i64) -> Result<KnowledgeSpaceStatus, KnowledgeSpaceStoreError> {
    match code {
        1 => Ok(KnowledgeSpaceStatus::Active),
        2 => Ok(KnowledgeSpaceStatus::Archived),
        3 => Ok(KnowledgeSpaceStatus::Deleted),
        _ => Err(KnowledgeSpaceStoreError::Internal(format!(
            "unknown knowledge space status code: {code}"
        ))),
    }
}

fn okf_bundle_file_type_code(value: OkfBundleFileKind) -> &'static str {
    match value {
        OkfBundleFileKind::BundleProfile => "bundle_profile",
        OkfBundleFileKind::BundleAgents => "bundle_agents",
        OkfBundleFileKind::BundleIndex => "bundle_index",
        OkfBundleFileKind::BundleLog => "bundle_log",
        OkfBundleFileKind::ConceptRevision => "concept_revision",
        OkfBundleFileKind::GraphExport => "graph_export",
        OkfBundleFileKind::ContextPack => "context_pack",
        OkfBundleFileKind::OutputExport => "output_export",
    }
}

fn okf_bundle_file_type_from_code(
    value: &str,
) -> Result<OkfBundleFileKind, KnowledgeOkfBundleFileStoreError> {
    match value {
        "bundle_profile" => Ok(OkfBundleFileKind::BundleProfile),
        "bundle_agents" => Ok(OkfBundleFileKind::BundleAgents),
        "bundle_index" => Ok(OkfBundleFileKind::BundleIndex),
        "bundle_log" => Ok(OkfBundleFileKind::BundleLog),
        "concept_revision" => Ok(OkfBundleFileKind::ConceptRevision),
        "graph_export" => Ok(OkfBundleFileKind::GraphExport),
        "context_pack" => Ok(OkfBundleFileKind::ContextPack),
        "output_export" => Ok(OkfBundleFileKind::OutputExport),
        _ => Err(KnowledgeOkfBundleFileStoreError::Internal(format!(
            "unknown okf bundle file kind: {value}"
        ))),
    }
}

fn bool_code(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn space_now() -> Result<String, KnowledgeSpaceStoreError> {
    now_rfc3339().map_err(KnowledgeSpaceStoreError::Internal)
}

fn okf_bundle_file_now() -> Result<String, KnowledgeOkfBundleFileStoreError> {
    now_rfc3339().map_err(KnowledgeOkfBundleFileStoreError::Internal)
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

fn space_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeSpaceStoreError> {
    to_i64(field, value).map_err(KnowledgeSpaceStoreError::Internal)
}

fn okf_bundle_file_to_i64(
    field: &str,
    value: u64,
) -> Result<i64, KnowledgeOkfBundleFileStoreError> {
    to_i64(field, value).map_err(KnowledgeOkfBundleFileStoreError::Internal)
}

fn space_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeSpaceStoreError> {
    from_i64(field, value).map_err(KnowledgeSpaceStoreError::Internal)
}

fn okf_bundle_file_from_i64(
    field: &str,
    value: i64,
) -> Result<u64, KnowledgeOkfBundleFileStoreError> {
    from_i64(field, value).map_err(KnowledgeOkfBundleFileStoreError::Internal)
}

fn to_i64(field: &str, value: u64) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("{field} is out of range"))
}

fn from_i64(field: &str, value: i64) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("{field} is negative"))
}

fn space_sqlx_error(error: sqlx::Error) -> KnowledgeSpaceStoreError {
    KnowledgeSpaceStoreError::Internal(error.to_string())
}

fn space_id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeSpaceStoreError {
    KnowledgeSpaceStoreError::Internal(error.to_string())
}

fn okf_bundle_file_sqlx_error(error: sqlx::Error) -> KnowledgeOkfBundleFileStoreError {
    KnowledgeOkfBundleFileStoreError::Internal(error.to_string())
}

fn okf_bundle_file_id_error(
    error: crate::KnowledgeIdGeneratorError,
) -> KnowledgeOkfBundleFileStoreError {
    KnowledgeOkfBundleFileStoreError::Internal(error.to_string())
}

fn space_fetch_error(space_id: u64, error: sqlx::Error) -> KnowledgeSpaceStoreError {
    if matches!(error, sqlx::Error::RowNotFound) {
        return KnowledgeSpaceStoreError::Internal(format!("missing knowledge space: {space_id}"));
    }
    space_sqlx_error(error)
}
