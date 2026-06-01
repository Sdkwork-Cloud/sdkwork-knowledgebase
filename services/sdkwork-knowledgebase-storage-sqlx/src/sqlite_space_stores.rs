use async_trait::async_trait;
use sdkwork_knowledgebase_contract::space::{KnowledgeSpace, KnowledgeSpaceStatus};
use sdkwork_knowledgebase_contract::wiki_file::{KnowledgeWikiFileEntry, WikiFileEntryType};
use sdkwork_knowledgebase_product::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use sdkwork_knowledgebase_product::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeSpaceStore {
    pool: SqlitePool,
    tenant_id: u64,
    organization_id: u64,
}

impl SqliteKnowledgeSpaceStore {
    pub fn new(pool: SqlitePool, tenant_id: u64, organization_id: u64) -> Self {
        Self {
            pool,
            tenant_id,
            organization_id,
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
        let now = space_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_space (
                uuid,
                tenant_id,
                organization_id,
                name,
                description,
                status,
                llm_wiki_initialized,
                created_at,
                updated_at,
                version
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, uuid, name, description, status, llm_wiki_initialized
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(organization_id)
        .bind(record.name)
        .bind(record.description)
        .bind(space_status_code(KnowledgeSpaceStatus::Active))
        .bind(bool_code(record.llm_wiki_initialized))
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(space_sqlx_error)?;

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
            UPDATE knowledge_space
            SET llm_wiki_initialized = 1, updated_at = ?, version = version + 1
            WHERE tenant_id = ? AND organization_id = ? AND id = ? AND status = ?
            RETURNING id, uuid, name, description, status, llm_wiki_initialized
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
}

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeWikiFileEntryStore {
    pool: SqlitePool,
    tenant_id: u64,
}

impl SqliteKnowledgeWikiFileEntryStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self { pool, tenant_id }
    }
}

#[async_trait]
impl KnowledgeWikiFileEntryStore for SqliteKnowledgeWikiFileEntryStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError> {
        let tenant_id = wiki_entry_to_i64("tenant_id", self.tenant_id)?;
        let space_id = wiki_entry_to_i64("space_id", record.space_id)?;
        let now = wiki_entry_now()?;

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_wiki_file_entry (
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
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        status: space_status_from_code(status_code)?,
        llm_wiki_initialized: llm_wiki_initialized != 0,
    })
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

fn wiki_entry_sqlx_error(error: sqlx::Error) -> KnowledgeWikiFileEntryStoreError {
    KnowledgeWikiFileEntryStoreError::Internal(error.to_string())
}

fn space_fetch_error(space_id: u64, error: sqlx::Error) -> KnowledgeSpaceStoreError {
    if matches!(error, sqlx::Error::RowNotFound) {
        return KnowledgeSpaceStoreError::Internal(format!("missing knowledge space: {space_id}"));
    }
    space_sqlx_error(error)
}
