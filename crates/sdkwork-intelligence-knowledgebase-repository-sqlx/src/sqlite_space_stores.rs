use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::space::{KnowledgeSpace, KnowledgeSpaceStatus};
use sdkwork_knowledgebase_contract::wiki_file::{KnowledgeWikiFileEntry, WikiFileEntryType};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeSpaceStore {
    pool: SqlitePool,
    tenant_id: u64,
    organization_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeSpaceStore {
    pub fn new(pool: SqlitePool, tenant_id: u64, organization_id: u64) -> Self {
        Self::with_id_generator(
            pool,
            tenant_id,
            organization_id,
            default_knowledge_id_generator(),
        )
    }

    pub fn with_id_generator(
        pool: SqlitePool,
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
                llm_wiki_initialized,
                knowledge_mode,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, uuid, name, description, drive_space_id, status, llm_wiki_initialized, knowledge_mode, knowledge_mode
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
        .bind(bool_code(record.llm_wiki_initialized))
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
            SELECT id, uuid, name, description, drive_space_id, status, llm_wiki_initialized, knowledge_mode
            FROM kb_space
            WHERE tenant_id = ? AND organization_id = ? AND id = ? AND status = ?
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
            SET drive_space_id = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND organization_id = ? AND id = ? AND status = ?
            RETURNING id, uuid, name, description, drive_space_id, status, llm_wiki_initialized, knowledge_mode
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

    async fn mark_llm_wiki_initialized(
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
            SET llm_wiki_initialized = 1, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND organization_id = ? AND id = ? AND status = ?
            RETURNING id, uuid, name, description, drive_space_id, status, llm_wiki_initialized, knowledge_mode
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
            SET status = ?, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND organization_id = ? AND id = ? AND status = ?
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
    pub async fn find_first_wiki_initialized_space(
        &self,
    ) -> Result<Option<KnowledgeSpace>, KnowledgeSpaceStoreError> {
        let tenant_id = space_to_i64("tenant_id", self.tenant_id)?;
        let organization_id = space_to_i64("organization_id", self.organization_id)?;
        let row = sqlx::query(
            r#"
            SELECT id, uuid, name, description, drive_space_id, status, llm_wiki_initialized, knowledge_mode
            FROM kb_space
            WHERE tenant_id = ? AND organization_id = ? AND status = ? AND llm_wiki_initialized = 1
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
pub struct SqliteKnowledgeWikiFileEntryStore {
    pool: SqlitePool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeWikiFileEntryStore {
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
impl KnowledgeWikiFileEntryStore for SqliteKnowledgeWikiFileEntryStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        self.insert_file_entry(record).await
    }

    async fn upsert_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        self.upsert_file_entry_record(record).await
    }
}

impl SqliteKnowledgeWikiFileEntryStore {
    async fn insert_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        let tenant_id = wiki_entry_to_i64("tenant_id", self.tenant_id)?;
        let space_id = wiki_entry_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(wiki_entry_id_error)?;
        let now = wiki_entry_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_wiki_file_entry (
                id,
                uuid,
                tenant_id,
                space_id,
                logical_path,
                entry_type,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id,
                space_id,
                logical_path,
                entry_type,
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
        .bind(wiki_entry_type_code(record.entry_type))
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
        .map_err(wiki_entry_sqlx_error)?;

        wiki_file_entry_from_row(&row)
    }

    async fn upsert_file_entry_record(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        let tenant_id = wiki_entry_to_i64("tenant_id", self.tenant_id)?;
        let space_id = wiki_entry_to_i64("space_id", record.space_id)?;
        let id = next_i64_id(&self.id_generator).map_err(wiki_entry_id_error)?;
        let now = wiki_entry_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_wiki_file_entry (
                id,
                uuid,
                tenant_id,
                space_id,
                logical_path,
                entry_type,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tenant_id, space_id, logical_path)
            DO UPDATE SET
                entry_type = excluded.entry_type,
                artifact_role = excluded.artifact_role,
                drive_bucket = excluded.drive_bucket,
                drive_object_key = excluded.drive_object_key,
                checksum_sha256_hex = excluded.checksum_sha256_hex,
                status = excluded.status,
                updated_at = excluded.updated_at,
                version = kb_wiki_file_entry.version + 1
            RETURNING
                id,
                space_id,
                logical_path,
                entry_type,
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
        .bind(wiki_entry_type_code(record.entry_type))
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
        .map_err(wiki_entry_sqlx_error)?;

        wiki_file_entry_from_row(&row)
    }

    pub async fn list_file_entries(
        &self,
    ) -> Result<Vec<KnowledgeWikiFileEntry>, KnowledgeWikiFileEntryStoreError> {
        let tenant_id = wiki_entry_to_i64("tenant_id", self.tenant_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                logical_path,
                entry_type,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            FROM kb_wiki_file_entry
            WHERE tenant_id = ? AND status = ?
            ORDER BY id ASC
            LIMIT 200
            "#,
        )
        .bind(tenant_id)
        .bind(ACTIVE_STATUS)
        .fetch_all(&self.pool)
        .await
        .map_err(wiki_entry_sqlx_error)?;

        rows.iter().map(wiki_file_entry_from_row).collect()
    }

    pub async fn get_file_entry_by_id(
        &self,
        entry_id: u64,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        let tenant_id = wiki_entry_to_i64("tenant_id", self.tenant_id)?;
        let entry_id = wiki_entry_to_i64("entry_id", entry_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                logical_path,
                entry_type,
                artifact_role,
                drive_bucket,
                drive_object_key,
                checksum_sha256_hex
            FROM kb_wiki_file_entry
            WHERE tenant_id = ? AND id = ? AND status = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(entry_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(wiki_entry_sqlx_error)?
        .ok_or_else(|| {
            KnowledgeWikiFileEntryStoreError::Internal(format!(
                "missing wiki file entry: {entry_id}"
            ))
        })?;

        wiki_file_entry_from_row(&row)
    }
}

fn space_from_row(row: &SqliteRow) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
    let status_code: i64 = row.try_get("status").map_err(space_sqlx_error)?;
    let llm_wiki_initialized: i64 = row
        .try_get("llm_wiki_initialized")
        .map_err(space_sqlx_error)?;
    Ok(KnowledgeSpace {
        id: space_from_i64("id", row.try_get("id").map_err(space_sqlx_error)?)?,
        uuid: row.try_get("uuid").map_err(space_sqlx_error)?,
        name: row.try_get("name").map_err(space_sqlx_error)?,
        description: row.try_get("description").map_err(space_sqlx_error)?,
        drive_space_id: row.try_get("drive_space_id").map_err(space_sqlx_error)?,
        status: space_status_from_code(status_code)?,
        llm_wiki_initialized: llm_wiki_initialized != 0,
        knowledge_mode: space_knowledge_mode_from_row(row)?,
    })
}

fn space_knowledge_mode_code(mode: KnowledgeAgentKnowledgeMode) -> &'static str {
    match mode {
        KnowledgeAgentKnowledgeMode::LlmWiki => "llm_wiki",
        KnowledgeAgentKnowledgeMode::Rag => "rag",
    }
}

fn space_knowledge_mode_from_row(
    row: &SqliteRow,
) -> Result<KnowledgeAgentKnowledgeMode, KnowledgeSpaceStoreError> {
    let value: Option<String> = row.try_get("knowledge_mode").map_err(space_sqlx_error)?;
    match value.as_deref().unwrap_or("llm_wiki") {
        "llm_wiki" => Ok(KnowledgeAgentKnowledgeMode::LlmWiki),
        "rag" => Ok(KnowledgeAgentKnowledgeMode::Rag),
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

fn wiki_file_entry_from_row(
    row: &SqliteRow,
) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
    let entry_type: String = row.try_get("entry_type").map_err(wiki_entry_sqlx_error)?;
    Ok(KnowledgeWikiFileEntry {
        id: wiki_entry_from_i64("id", row.try_get("id").map_err(wiki_entry_sqlx_error)?)?,
        space_id: wiki_entry_from_i64(
            "space_id",
            row.try_get("space_id").map_err(wiki_entry_sqlx_error)?,
        )?,
        logical_path: row.try_get("logical_path").map_err(wiki_entry_sqlx_error)?,
        entry_type: wiki_entry_type_from_code(&entry_type)?,
        artifact_role: row
            .try_get("artifact_role")
            .map_err(wiki_entry_sqlx_error)?,
        drive_bucket: row.try_get("drive_bucket").map_err(wiki_entry_sqlx_error)?,
        drive_object_key: row
            .try_get("drive_object_key")
            .map_err(wiki_entry_sqlx_error)?,
        checksum_sha256_hex: row
            .try_get("checksum_sha256_hex")
            .map_err(wiki_entry_sqlx_error)?,
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

fn wiki_entry_type_code(value: WikiFileEntryType) -> &'static str {
    match value {
        WikiFileEntryType::WikiSchema => "wiki_schema",
        WikiFileEntryType::WikiIndex => "wiki_index",
        WikiFileEntryType::WikiLog => "wiki_log",
        WikiFileEntryType::WikiRevision => "wiki_revision",
        WikiFileEntryType::GraphExport => "graph_export",
        WikiFileEntryType::ContextPack => "context_pack",
        WikiFileEntryType::OutputExport => "output_export",
    }
}

fn wiki_entry_type_from_code(
    value: &str,
) -> Result<WikiFileEntryType, KnowledgeWikiFileEntryStoreError> {
    match value {
        "wiki_schema" => Ok(WikiFileEntryType::WikiSchema),
        "wiki_index" => Ok(WikiFileEntryType::WikiIndex),
        "wiki_log" => Ok(WikiFileEntryType::WikiLog),
        "wiki_revision" => Ok(WikiFileEntryType::WikiRevision),
        "graph_export" => Ok(WikiFileEntryType::GraphExport),
        "context_pack" => Ok(WikiFileEntryType::ContextPack),
        "output_export" => Ok(WikiFileEntryType::OutputExport),
        _ => Err(KnowledgeWikiFileEntryStoreError::Internal(format!(
            "unknown wiki file entry type: {value}"
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

fn wiki_entry_now() -> Result<String, KnowledgeWikiFileEntryStoreError> {
    now_rfc3339().map_err(KnowledgeWikiFileEntryStoreError::Internal)
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

fn space_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeSpaceStoreError> {
    to_i64(field, value).map_err(KnowledgeSpaceStoreError::Internal)
}

fn wiki_entry_to_i64(field: &str, value: u64) -> Result<i64, KnowledgeWikiFileEntryStoreError> {
    to_i64(field, value).map_err(KnowledgeWikiFileEntryStoreError::Internal)
}

fn space_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeSpaceStoreError> {
    from_i64(field, value).map_err(KnowledgeSpaceStoreError::Internal)
}

fn wiki_entry_from_i64(field: &str, value: i64) -> Result<u64, KnowledgeWikiFileEntryStoreError> {
    from_i64(field, value).map_err(KnowledgeWikiFileEntryStoreError::Internal)
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

fn wiki_entry_sqlx_error(error: sqlx::Error) -> KnowledgeWikiFileEntryStoreError {
    KnowledgeWikiFileEntryStoreError::Internal(error.to_string())
}

fn wiki_entry_id_error(
    error: crate::KnowledgeIdGeneratorError,
) -> KnowledgeWikiFileEntryStoreError {
    KnowledgeWikiFileEntryStoreError::Internal(error.to_string())
}

fn space_fetch_error(space_id: u64, error: sqlx::Error) -> KnowledgeSpaceStoreError {
    if matches!(error, sqlx::Error::RowNotFound) {
        return KnowledgeSpaceStoreError::Internal(format!("missing knowledge space: {space_id}"));
    }
    space_sqlx_error(error)
}
