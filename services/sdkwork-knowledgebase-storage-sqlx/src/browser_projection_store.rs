use async_trait::async_trait;
use sdkwork_knowledgebase_contract::wiki::WikiPagePublishState;
use sdkwork_knowledgebase_product::ports::knowledge_browser_projection_store::{
    KnowledgeBrowserDocumentProjection, KnowledgeBrowserProjectionStore,
    KnowledgeBrowserProjectionStoreError, KnowledgeBrowserWikiPageProjection,
};
use sqlx::{QueryBuilder, Row, SqlitePool};

const ACTIVE_STATUS: i64 = 1;
const MAX_PROJECTION_BATCH_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeBrowserProjectionStore {
    pool: SqlitePool,
    tenant_id: u64,
}

impl SqliteKnowledgeBrowserProjectionStore {
    pub fn new(pool: SqlitePool, tenant_id: u64) -> Self {
        Self { pool, tenant_id }
    }
}

#[async_trait]
impl KnowledgeBrowserProjectionStore for SqliteKnowledgeBrowserProjectionStore {
    async fn batch_document_projections(
        &self,
        space_id: u64,
        drive_node_ids: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserDocumentProjection>, KnowledgeBrowserProjectionStoreError> {
        if drive_node_ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size("drive_node_ids", drive_node_ids.len())?;

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let mut builder = QueryBuilder::new(
            r#"
            SELECT
                d.original_file_drive_node_id,
                d.id AS document_id,
                d.current_version_id,
                d.index_state,
                v.parse_state
            FROM kb_document d
            LEFT JOIN kb_document_version v
              ON v.tenant_id = d.tenant_id
             AND v.id = d.current_version_id
             AND v.status = "#,
        );
        builder.push_bind(ACTIVE_STATUS);
        builder.push(
            r#"
            WHERE d.tenant_id = "#,
        );
        builder.push_bind(tenant_id);
        builder.push(" AND d.space_id = ");
        builder.push_bind(space_id);
        builder.push(" AND d.status = ");
        builder.push_bind(ACTIVE_STATUS);
        builder.push(" AND d.original_file_drive_node_id IN (");
        let mut separated = builder.separated(", ");
        for drive_node_id in drive_node_ids {
            separated.push_bind(drive_node_id);
        }
        separated.push_unseparated(")");

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let drive_node_id: Option<String> = row
                    .try_get("original_file_drive_node_id")
                    .map_err(sqlx_error)?;
                let drive_node_id = drive_node_id.ok_or_else(|| {
                    KnowledgeBrowserProjectionStoreError::Internal(
                        "document projection is missing drive node id".to_string(),
                    )
                })?;
                let document_id = from_i64(
                    "document_id",
                    row.try_get("document_id").map_err(sqlx_error)?,
                )?;
                let current_version_id = row
                    .try_get::<Option<i64>, _>("current_version_id")
                    .map_err(sqlx_error)?
                    .map(|value| from_i64("current_version_id", value))
                    .transpose()?;
                let index_state = row.try_get("index_state").map_err(sqlx_error)?;
                let parse_state = row
                    .try_get::<Option<i64>, _>("parse_state")
                    .map_err(sqlx_error)?
                    .unwrap_or(0);

                let parse_state_name = version_state_name(parse_state).to_string();
                let index_state_name = index_state_name(index_state).to_string();
                let ingest_state = ingest_state_name(parse_state, index_state).to_string();

                Ok(KnowledgeBrowserDocumentProjection {
                    drive_node_id,
                    document_id,
                    current_version_id,
                    ingest_state,
                    parse_state: parse_state_name,
                    index_state: index_state_name,
                    wiki_state: "none".to_string(),
                })
            })
            .collect()
    }

    async fn batch_wiki_page_projections(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserWikiPageProjection>, KnowledgeBrowserProjectionStoreError> {
        if logical_paths.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size("logical_paths", logical_paths.len())?;

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let mut builder = QueryBuilder::new(
            r#"
            SELECT logical_path, id, current_revision_id, publish_state
            FROM kb_wiki_page
            WHERE tenant_id =
            "#,
        );
        builder.push_bind(tenant_id);
        builder.push(" AND space_id = ");
        builder.push_bind(space_id);
        builder.push(" AND status = ");
        builder.push_bind(ACTIVE_STATUS);
        builder.push(" AND logical_path IN (");
        let mut separated = builder.separated(", ");
        for logical_path in logical_paths {
            separated.push_bind(logical_path);
        }
        separated.push_unseparated(")");

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
                Ok(KnowledgeBrowserWikiPageProjection {
                    logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
                    page_id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
                    current_revision_id: row
                        .try_get::<Option<i64>, _>("current_revision_id")
                        .map_err(sqlx_error)?
                        .map(|value| from_i64("current_revision_id", value))
                        .transpose()?,
                    publish_state: wiki_publish_state_name(&publish_state)?,
                })
            })
            .collect()
    }
}

fn validate_batch_size(
    field: &str,
    len: usize,
) -> Result<(), KnowledgeBrowserProjectionStoreError> {
    if len > MAX_PROJECTION_BATCH_SIZE {
        return Err(KnowledgeBrowserProjectionStoreError::InvalidRequest(
            format!("{field} batch size must be <= {MAX_PROJECTION_BATCH_SIZE}"),
        ));
    }
    Ok(())
}

fn version_state_name(code: i64) -> &'static str {
    match code {
        0 => "pending",
        1 => "running",
        2 => "succeeded",
        3 => "failed",
        _ => "unknown",
    }
}

fn index_state_name(code: i64) -> &'static str {
    match code {
        0 => "pending",
        1 => "running",
        2 => "indexed",
        3 => "failed",
        _ => "unknown",
    }
}

fn ingest_state_name(parse_state: i64, index_state: i64) -> &'static str {
    if parse_state == 3 || index_state == 3 {
        return "failed";
    }
    if parse_state == 1 || index_state == 1 {
        return "running";
    }
    if parse_state == 2 && index_state == 2 {
        return "completed";
    }
    "pending"
}

fn wiki_publish_state_name(
    value: &str,
) -> Result<WikiPagePublishState, KnowledgeBrowserProjectionStoreError> {
    match value {
        "draft" => Ok(WikiPagePublishState::Draft),
        "candidate_ready" => Ok(WikiPagePublishState::CandidateReady),
        "needs_review" => Ok(WikiPagePublishState::NeedsReview),
        "published" => Ok(WikiPagePublishState::Published),
        "stale" => Ok(WikiPagePublishState::Stale),
        "rejected" => Ok(WikiPagePublishState::Rejected),
        "failed" => Ok(WikiPagePublishState::Failed),
        _ => Err(KnowledgeBrowserProjectionStoreError::Internal(format!(
            "unknown wiki publish state: {value}"
        ))),
    }
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeBrowserProjectionStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeBrowserProjectionStoreError::Internal(format!("{field} is out of range"))
    })
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeBrowserProjectionStoreError> {
    u64::try_from(value)
        .map_err(|_| KnowledgeBrowserProjectionStoreError::Internal(format!("{field} is negative")))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeBrowserProjectionStoreError {
    KnowledgeBrowserProjectionStoreError::Internal(error.to_string())
}
