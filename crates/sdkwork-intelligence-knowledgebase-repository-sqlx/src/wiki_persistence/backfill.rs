use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    ListWikiPublicationBackfillCandidatesRequest, WikiPersistenceError,
    WikiPublicationBackfillCandidate, WikiPublicationBackfillCandidatePage,
    WikiPublicationBackfillStore,
};
use sqlx::Row;

use super::{from_i64, row_error, sql_error, to_i64, validate_scope, SqlxWikiPersistenceStore};

const MAX_BACKFILL_PAGE_SIZE: u32 = 200;

#[async_trait]
impl WikiPublicationBackfillStore for SqlxWikiPersistenceStore {
    async fn list_backfill_candidates(
        &self,
        request: ListWikiPublicationBackfillCandidatesRequest,
    ) -> Result<WikiPublicationBackfillCandidatePage, WikiPersistenceError> {
        validate_scope(request.scope)?;
        if request.limit == 0 || request.limit > MAX_BACKFILL_PAGE_SIZE {
            return Err(WikiPersistenceError::InvalidRequest(format!(
                "limit must be between 1 and {MAX_BACKFILL_PAGE_SIZE}"
            )));
        }
        let after_space_id = request.after_space_id.unwrap_or(0);
        let fetch_limit = i64::from(request.limit) + 1;
        let rows = sqlx::query(
            r#"
            SELECT
                space.id AS space_id,
                space.uuid AS knowledgebase_uuid,
                space.name AS title,
                space.drive_space_id AS drive_space_uuid,
                CAST(CASE WHEN publication.id IS NULL THEN 1 ELSE 0 END AS BIGINT)
                    AS publication_missing,
                CAST(CASE
                    WHEN publication.id IS NULL
                      OR publication.source_root_node_uuid IS NULL
                      OR publication.source_scope_uuid IS NULL
                    THEN 1 ELSE 0 END AS BIGINT) AS source_scope_missing,
                CAST(CASE WHEN checkpoint.id IS NULL THEN 1 ELSE 0 END AS BIGINT)
                    AS checkpoint_missing
            FROM kb_space space
            LEFT JOIN kb_site_publication publication
              ON publication.tenant_id = space.tenant_id
             AND publication.organization_id = space.organization_id
             AND publication.space_id = space.id
             AND publication.status = 1
            LEFT JOIN kb_drive_source_checkpoint checkpoint
              ON checkpoint.tenant_id = publication.tenant_id
             AND checkpoint.organization_id = publication.organization_id
             AND checkpoint.site_publication_id = publication.id
             AND checkpoint.status = 1
            WHERE space.tenant_id = $1
              AND space.organization_id = $2
              AND space.status = 1
              AND space.drive_space_id IS NOT NULL
              AND space.id > $3
              AND (
                    publication.id IS NULL
                 OR publication.source_root_node_uuid IS NULL
                 OR publication.source_scope_uuid IS NULL
                 OR checkpoint.id IS NULL
              )
            ORDER BY space.id ASC
            LIMIT $4
            "#,
        )
        .bind(to_i64("tenant_id", request.scope.tenant_id)?)
        .bind(to_i64("organization_id", request.scope.organization_id)?)
        .bind(to_i64("after_space_id", after_space_id)?)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(sql_error)?;

        let mut candidates = rows
            .into_iter()
            .map(|row| {
                Ok(WikiPublicationBackfillCandidate {
                    space_id: from_i64("space_id", row.try_get("space_id").map_err(row_error)?)?,
                    knowledgebase_uuid: row.try_get("knowledgebase_uuid").map_err(row_error)?,
                    title: row.try_get("title").map_err(row_error)?,
                    drive_space_uuid: row.try_get("drive_space_uuid").map_err(row_error)?,
                    publication_missing: flag(&row, "publication_missing")?,
                    source_scope_missing: flag(&row, "source_scope_missing")?,
                    checkpoint_missing: flag(&row, "checkpoint_missing")?,
                })
            })
            .collect::<Result<Vec<_>, WikiPersistenceError>>()?;
        let has_more = candidates.len() > request.limit as usize;
        if has_more {
            candidates.truncate(request.limit as usize);
        }
        let next_after_space_id = has_more
            .then(|| candidates.last().map(|candidate| candidate.space_id))
            .flatten();
        Ok(WikiPublicationBackfillCandidatePage {
            candidates,
            next_after_space_id,
        })
    }
}

fn flag(row: &sqlx::any::AnyRow, column: &str) -> Result<bool, WikiPersistenceError> {
    let value: i64 = row.try_get(column).map_err(row_error)?;
    Ok(value != 0)
}
