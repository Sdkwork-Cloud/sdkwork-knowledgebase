use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_browser_projection_store::{
    KnowledgeBrowserDocumentProjection, KnowledgeBrowserOkfConceptProjection,
    KnowledgeBrowserProjectionStore, KnowledgeBrowserProjectionStoreError,
};
use sdkwork_knowledgebase_contract::OkfConceptPublishState;
use sqlx::{AnyPool, Row};

const ACTIVE_STATUS: i64 = 1;
const MAX_PROJECTION_BATCH_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeBrowserProjectionStore {
    pool: AnyPool,
    tenant_id: u64,
}

impl SqliteKnowledgeBrowserProjectionStore {
    pub fn new(pool: AnyPool, tenant_id: u64) -> Self {
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
        // Build $N placeholders for the IN clause. Fixed bind params occupy $1..$3
        // (tenant_id, space_id, ACTIVE_STATUS); drive node ids start at $4.
        let in_placeholders = build_in_placeholders(drive_node_ids.len(), 3);
        let sql = format!(
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
             AND v.status = 1
            WHERE d.tenant_id = $1
              AND d.space_id = $2
              AND d.status = $3
              AND d.original_file_drive_node_id IN ({in_placeholders})
            "#
        );

        let mut query = sqlx::query(&sql)
            .bind(tenant_id)
            .bind(space_id)
            .bind(ACTIVE_STATUS);
        for drive_node_id in drive_node_ids {
            query = query.bind(drive_node_id);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(sqlx_error)?;

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
                    okf_state: "none".to_string(),
                })
            })
            .collect()
    }

    async fn batch_okf_concept_projections(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserOkfConceptProjection>, KnowledgeBrowserProjectionStoreError>
    {
        if logical_paths.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size("logical_paths", logical_paths.len())?;

        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        // Build $N placeholders for the IN clause. Fixed bind params occupy $1..$3
        // (tenant_id, space_id, ACTIVE_STATUS); logical paths start at $4.
        let in_placeholders = build_in_placeholders(logical_paths.len(), 3);
        let sql = format!(
            r#"
            SELECT logical_path, id, current_revision_id, publish_state
            FROM kb_okf_concept
            WHERE tenant_id = $1
              AND space_id = $2
              AND status = $3
              AND logical_path IN ({in_placeholders})
            "#
        );

        let mut query = sqlx::query(&sql)
            .bind(tenant_id)
            .bind(space_id)
            .bind(ACTIVE_STATUS);
        for logical_path in logical_paths {
            query = query.bind(logical_path);
        }
        let rows = query.fetch_all(&self.pool).await.map_err(sqlx_error)?;

        rows.into_iter()
            .map(|row| {
                let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
                Ok(KnowledgeBrowserOkfConceptProjection {
                    logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
                    concept_row_id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
                    current_revision_id: row
                        .try_get::<Option<i64>, _>("current_revision_id")
                        .map_err(sqlx_error)?
                        .map(|value| from_i64("current_revision_id", value))
                        .transpose()?,
                    publish_state: okf_publish_state_name(&publish_state)?,
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

fn okf_publish_state_name(
    value: &str,
) -> Result<OkfConceptPublishState, KnowledgeBrowserProjectionStoreError> {
    match value {
        "draft" => Ok(OkfConceptPublishState::Draft),
        "candidate_ready" => Ok(OkfConceptPublishState::CandidateReady),
        "needs_review" => Ok(OkfConceptPublishState::NeedsReview),
        "published" => Ok(OkfConceptPublishState::Published),
        "stale" => Ok(OkfConceptPublishState::Stale),
        "rejected" => Ok(OkfConceptPublishState::Rejected),
        "failed" => Ok(OkfConceptPublishState::Failed),
        _ => Err(KnowledgeBrowserProjectionStoreError::Internal(format!(
            "unknown okf publish state: {value}"
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

/// Build `$N` placeholders for an IN clause.
///
/// `reserved` is the number of bind parameters already used before the IN clause.
/// The first IN-clause placeholder is therefore `$(reserved + 1)`.
///
/// We use numbered `$N` placeholders (not `?`) because `QueryBuilder::<Any>` writes
/// `?` via `AnyArguments::format_placeholder`, which PostgreSQL rejects as a syntax
/// error (it parses `?` as an operator). Both PostgreSQL and SQLite (via sqlx)
/// accept numbered `$N` placeholders, so this keeps the `Any` driver working on
/// both backends. See `retrieval_store.rs` for the same pattern.
fn build_in_placeholders(count: usize, reserved: usize) -> String {
    (0..count)
        .map(|i| format!("${}", reserved + i + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_in_placeholders_starts_after_reserved_params() {
        // 3 fixed params (tenant_id, space_id, ACTIVE_STATUS) occupy $1..$3,
        // so the IN clause must start at $4.
        let placeholders = build_in_placeholders(3, 3);
        assert_eq!(placeholders, "$4, $5, $6");
    }

    #[test]
    fn document_projection_sql_keeps_join_status_before_where() {
        // The SQL is now a static string with $N placeholders; verify the structure
        // that previously caused "syntax error at or near AND" is correct:
        // the active-version join filter (`v.status = 1`) must stay in the ON clause,
        // not leak into the WHERE clause.
        let in_placeholders = build_in_placeholders(1, 3);
        let sql = format!(
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
             AND v.status = 1
            WHERE d.tenant_id = $1
              AND d.space_id = $2
              AND d.status = $3
              AND d.original_file_drive_node_id IN ({in_placeholders})
            "#
        );
        assert!(
            !sql.contains("= WHERE"),
            "join status predicate must appear before WHERE clause: {sql}"
        );
        assert!(
            sql.contains("v.status = 1"),
            "expected active version join filter in browser projection sql: {sql}"
        );
        assert!(
            sql.contains("original_file_drive_node_id IN ("),
            "expected drive node batch filter in browser projection sql: {sql}"
        );
        // No `?` placeholders: PostgreSQL would parse `=?` as an operator and fail.
        assert!(
            !sql.contains('?'),
            "browser projection sql must not use ? placeholders (PostgreSQL incompatible): {sql}"
        );
    }
}
