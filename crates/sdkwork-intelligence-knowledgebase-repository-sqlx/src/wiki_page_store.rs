use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::{
    AppendKnowledgeWikiLogEntryRecord, CreateKnowledgeWikiPageRevisionRecord,
    KnowledgeWikiPageProjection, KnowledgeWikiPageStore, KnowledgeWikiPageStoreError,
    MarkKnowledgeWikiCurrentRevisionRecord, UpsertKnowledgeWikiPageRecord,
};
use sdkwork_knowledgebase_contract::wiki::{
    KnowledgeWikiPage, KnowledgeWikiPageRevision, WikiLogEntry, WikiLogEventType,
    WikiPagePublishState, WikiPageSummary, WikiPageType, WikiRevisionReviewState,
};
use sqlx::{any::AnyRow, AnyPool, QueryBuilder, Row};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const MAX_WIKI_LIST_ROWS: i64 = 200;
const INITIAL_VERSION: i64 = 0;
const MAX_PROJECTION_BATCH_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeWikiPageStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeWikiPageStore {
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

    pub async fn next_revision_no(&self, page_id: u64) -> Result<u64, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", page_id)?;
        let next: i64 = sqlx::query_scalar(
            r#"
            UPDATE kb_wiki_page
            SET revision_counter = revision_counter + 1,
                updated_at = $1,
                version = version + 1
            WHERE tenant_id = $2 AND id = $3 AND status = $4
            RETURNING revision_counter
            "#,
        )
        .bind(now_rfc3339()?)
        .bind(tenant_id)
        .bind(page_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;
        from_i64("revision_no", next)
    }

    pub async fn list_all_page_summaries(
        &self,
    ) -> Result<Vec<WikiPageSummary>, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                title,
                slug,
                page_type,
                logical_path,
                summary,
                source_count,
                updated_at,
                tags
            FROM kb_wiki_page
            WHERE tenant_id = $1 AND status = $2
            ORDER BY space_id ASC, page_type ASC, title ASC, id ASC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_WIKI_LIST_ROWS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.into_iter().map(summary_from_row).collect()
    }

    pub async fn get_page_by_id(
        &self,
        page_id: u64,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", page_id)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                space_id,
                slug,
                title,
                page_type,
                logical_path,
                summary,
                source_count,
                tags,
                current_revision_id,
                publish_state,
                updated_at
            FROM kb_wiki_page
            WHERE tenant_id = $1 AND id = $2 AND status = $3
            "#,
        )
        .bind(tenant_id)
        .bind(page_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or_else(|| {
            KnowledgeWikiPageStoreError::Internal(format!("missing wiki page: {page_id}"))
        })?;

        page_from_row(&row)
    }

    pub async fn list_page_revisions(
        &self,
        page_id: u64,
    ) -> Result<Vec<KnowledgeWikiPageRevision>, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", page_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                page_id,
                revision_no,
                markdown_object_ref_id,
                content_hash,
                review_state,
                created_at
            FROM kb_wiki_page_revision
            WHERE tenant_id = $1 AND page_id = $2 AND status = $3
            ORDER BY revision_no ASC
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(page_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_WIKI_LIST_ROWS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.iter().map(revision_from_row).collect()
    }

    pub async fn list_candidate_pages(
        &self,
    ) -> Result<Vec<(u64, WikiPagePublishState)>, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let rows = sqlx::query(
            r#"
            SELECT id, publish_state
            FROM kb_wiki_page
            WHERE tenant_id = $1 AND status = $2
              AND publish_state IN ('candidate_ready', 'needs_review')
            ORDER BY id ASC
            LIMIT $3
            "#,
        )
        .bind(tenant_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_WIKI_LIST_ROWS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.iter()
            .map(|row| {
                let id = from_i64("id", row.try_get("id").map_err(sqlx_error)?)?;
                let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
                Ok((id, publish_state_from_str(&publish_state)?))
            })
            .collect()
    }

    pub async fn update_page_publish_state(
        &self,
        page_id: u64,
        publish_state: WikiPagePublishState,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", page_id)?;
        let now = now_rfc3339()?;
        let row = sqlx::query(
            r#"
            UPDATE kb_wiki_page
            SET publish_state = $1, updated_at = $2, version = version + 1
            WHERE tenant_id = $3 AND id = $4 AND status = $5
            RETURNING
                id,
                space_id,
                slug,
                title,
                page_type,
                logical_path,
                summary,
                source_count,
                tags,
                current_revision_id,
                publish_state,
                updated_at
            "#,
        )
        .bind(publish_state.as_str())
        .bind(now)
        .bind(tenant_id)
        .bind(page_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or_else(|| {
            KnowledgeWikiPageStoreError::Internal(format!("missing wiki page: {page_id}"))
        })?;

        page_from_row(&row)
    }
}

#[async_trait]
impl KnowledgeWikiPageStore for SqliteKnowledgeWikiPageStore {
    async fn upsert_page(
        &self,
        record: UpsertKnowledgeWikiPageRecord,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let source_count = i64::from(record.source_count);
        let now = now_rfc3339()?;
        let page_type = record.page_type.as_str();
        let publish_state = record.publish_state.as_str();
        let tags = tags_to_json(&record.tags)?;
        let id = next_i64_id(&self.id_generator).map_err(id_error)?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_wiki_page (
                id,
                uuid,
                tenant_id,
                space_id,
                slug,
                title,
                page_type,
                logical_path,
                summary,
                source_count,
                tags,
                current_revision_id,
                publish_state,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NULL, $12, $13, $14, $15, $16)
            ON CONFLICT(tenant_id, space_id, slug)
            DO UPDATE SET
                title = excluded.title,
                page_type = excluded.page_type,
                logical_path = excluded.logical_path,
                summary = excluded.summary,
                source_count = excluded.source_count,
                tags = excluded.tags,
                publish_state = excluded.publish_state,
                updated_at = excluded.updated_at,
                version = kb_wiki_page.version + 1
            RETURNING
                id,
                space_id,
                slug,
                title,
                page_type,
                logical_path,
                summary,
                source_count,
                tags,
                current_revision_id,
                publish_state,
                updated_at
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.slug)
        .bind(record.title)
        .bind(page_type)
        .bind(record.logical_path)
        .bind(record.summary)
        .bind(source_count)
        .bind(tags)
        .bind(publish_state)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        page_from_row(&row)
    }

    async fn create_revision(
        &self,
        record: CreateKnowledgeWikiPageRevisionRecord,
    ) -> Result<KnowledgeWikiPageRevision, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", record.page_id)?;
        let revision_no = to_i64("revision_no", record.revision_no)?;
        let markdown_object_ref_id =
            to_i64("markdown_object_ref_id", record.markdown_object_ref_id)?;
        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        let now = now_rfc3339()?;

        let row = sqlx::query(
            r#"
            INSERT INTO kb_wiki_page_revision (
                id,
                uuid,
                tenant_id,
                page_id,
                revision_no,
                markdown_object_ref_id,
                content_hash,
                review_state,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id,
                page_id,
                revision_no,
                markdown_object_ref_id,
                content_hash,
                review_state,
                created_at
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(page_id)
        .bind(revision_no)
        .bind(markdown_object_ref_id)
        .bind(record.content_hash)
        .bind(record.review_state.as_str())
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        revision_from_row(&row)
    }

    async fn next_revision_no(&self, page_id: u64) -> Result<u64, KnowledgeWikiPageStoreError> {
        SqliteKnowledgeWikiPageStore::next_revision_no(self, page_id).await
    }

    async fn mark_current_revision(
        &self,
        record: MarkKnowledgeWikiCurrentRevisionRecord,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let page_id = to_i64("page_id", record.page_id)?;
        let revision_id = to_i64("revision_id", record.revision_id)?;
        let now = now_rfc3339()?;

        let row = sqlx::query(
            r#"
            UPDATE kb_wiki_page
            SET current_revision_id = $1,
                publish_state = $2,
                updated_at = $3,
                version = version + 1
            WHERE tenant_id = $4 AND id = $5 AND status = $6
            RETURNING
                id,
                space_id,
                slug,
                title,
                page_type,
                logical_path,
                summary,
                source_count,
                tags,
                current_revision_id,
                publish_state,
                updated_at
            "#,
        )
        .bind(revision_id)
        .bind(record.publish_state.as_str())
        .bind(now)
        .bind(tenant_id)
        .bind(page_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        page_from_row(&row)
    }

    async fn list_page_summaries(
        &self,
        space_id: u64,
    ) -> Result<Vec<WikiPageSummary>, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let rows = sqlx::query(
            r#"
            SELECT
                title,
                slug,
                page_type,
                logical_path,
                summary,
                source_count,
                updated_at,
                tags
            FROM kb_wiki_page
            WHERE tenant_id = $1 AND space_id = $2 AND status = $3
            ORDER BY page_type ASC, title ASC, id ASC
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_WIKI_LIST_ROWS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.into_iter().map(summary_from_row).collect()
    }

    async fn append_log_entry(
        &self,
        record: AppendKnowledgeWikiLogEntryRecord,
    ) -> Result<WikiLogEntry, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let now = now_rfc3339()?;
        let sequence_no: i64 = sqlx::query_scalar(
            r#"
            UPDATE kb_space
            SET wiki_log_sequence_counter = wiki_log_sequence_counter + 1,
                updated_at = $1,
                version = version + 1
            WHERE tenant_id = $2 AND id = $3 AND status = $4
            RETURNING wiki_log_sequence_counter
            "#,
        )
        .bind(now.clone())
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        let metadata = log_metadata_to_json(
            &record.actor,
            &record.affected_pages,
            record.audit_event_id.as_deref(),
            &record.warnings,
        )?;
        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        let row = sqlx::query(
            r#"
            INSERT INTO kb_wiki_log_entry (
                id,
                uuid,
                tenant_id,
                space_id,
                sequence_no,
                event_type,
                event_time,
                title,
                privacy_level,
                metadata,
                status,
                created_at,
                updated_at,
                version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING
                event_type,
                event_time,
                title,
                metadata
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(sequence_no)
        .bind(record.event_type)
        .bind(record.event_time)
        .bind(record.title)
        .bind(record.privacy_level)
        .bind(metadata)
        .bind(ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(INITIAL_VERSION)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        log_entry_from_row(&row)
    }

    async fn list_log_entries(
        &self,
        space_id: u64,
    ) -> Result<Vec<WikiLogEntry>, KnowledgeWikiPageStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", space_id)?;
        let rows = sqlx::query(
            r#"
            SELECT event_type, event_time, title, metadata
            FROM kb_wiki_log_entry
            WHERE tenant_id = $1 AND space_id = $2 AND status = $3
            ORDER BY sequence_no ASC
            LIMIT $4
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(ACTIVE_STATUS)
        .bind(MAX_WIKI_LIST_ROWS)
        .fetch_all(&self.pool)
        .await
        .map_err(sqlx_error)?;

        rows.into_iter()
            .map(|row| log_entry_from_row(&row))
            .collect()
    }

    async fn batch_page_projections_by_paths(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeWikiPageProjection>, KnowledgeWikiPageStoreError> {
        if logical_paths.is_empty() {
            return Ok(vec![]);
        }
        validate_projection_batch_size(logical_paths.len())?;

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
        for path in logical_paths {
            separated.push_bind(path);
        }
        separated.push_unseparated(")");

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?;

        rows.into_iter().map(projection_from_row).collect()
    }
}

fn validate_projection_batch_size(len: usize) -> Result<(), KnowledgeWikiPageStoreError> {
    if len > MAX_PROJECTION_BATCH_SIZE {
        return Err(KnowledgeWikiPageStoreError::Internal(format!(
            "logical_paths batch size must be <= {MAX_PROJECTION_BATCH_SIZE}"
        )));
    }
    Ok(())
}

fn page_from_row(row: &AnyRow) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError> {
    let page_type: String = row.try_get("page_type").map_err(sqlx_error)?;
    let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
    let source_count: i64 = row.try_get("source_count").map_err(sqlx_error)?;
    Ok(KnowledgeWikiPage {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        space_id: from_i64("space_id", row.try_get("space_id").map_err(sqlx_error)?)?,
        slug: row.try_get("slug").map_err(sqlx_error)?,
        title: row.try_get("title").map_err(sqlx_error)?,
        page_type: page_type_from_str(&page_type)?,
        logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
        summary: row.try_get("summary").map_err(sqlx_error)?,
        source_count: u32::try_from(source_count).map_err(|_| {
            KnowledgeWikiPageStoreError::Internal("source_count is out of range".to_string())
        })?,
        tags: tags_from_json(row.try_get("tags").map_err(sqlx_error)?)?,
        current_revision_id: row
            .try_get::<Option<i64>, _>("current_revision_id")
            .map_err(sqlx_error)?
            .map(|value| from_i64("current_revision_id", value))
            .transpose()?,
        publish_state: publish_state_from_str(&publish_state)?,
        updated_at: row.try_get("updated_at").map_err(sqlx_error)?,
    })
}

fn revision_from_row(
    row: &AnyRow,
) -> Result<KnowledgeWikiPageRevision, KnowledgeWikiPageStoreError> {
    let review_state: String = row.try_get("review_state").map_err(sqlx_error)?;
    Ok(KnowledgeWikiPageRevision {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        page_id: from_i64("page_id", row.try_get("page_id").map_err(sqlx_error)?)?,
        revision_no: from_i64(
            "revision_no",
            row.try_get("revision_no").map_err(sqlx_error)?,
        )?,
        markdown_object_ref_id: from_i64(
            "markdown_object_ref_id",
            row.try_get("markdown_object_ref_id").map_err(sqlx_error)?,
        )?,
        content_hash: row.try_get("content_hash").map_err(sqlx_error)?,
        review_state: review_state_from_str(&review_state)?,
        created_at: row.try_get("created_at").map_err(sqlx_error)?,
    })
}

fn summary_from_row(row: AnyRow) -> Result<WikiPageSummary, KnowledgeWikiPageStoreError> {
    let page_type: String = row.try_get("page_type").map_err(sqlx_error)?;
    let source_count: i64 = row.try_get("source_count").map_err(sqlx_error)?;
    Ok(WikiPageSummary {
        title: row.try_get("title").map_err(sqlx_error)?,
        slug: row.try_get("slug").map_err(sqlx_error)?,
        page_type: page_type_from_str(&page_type)?,
        logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
        summary: row.try_get("summary").map_err(sqlx_error)?,
        source_count: u32::try_from(source_count).map_err(|_| {
            KnowledgeWikiPageStoreError::Internal("source_count is out of range".to_string())
        })?,
        updated_at: row.try_get("updated_at").map_err(sqlx_error)?,
        tags: tags_from_json(row.try_get("tags").map_err(sqlx_error)?)?,
    })
}

fn projection_from_row(
    row: AnyRow,
) -> Result<KnowledgeWikiPageProjection, KnowledgeWikiPageStoreError> {
    let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
    Ok(KnowledgeWikiPageProjection {
        logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
        page_id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        current_revision_id: row
            .try_get::<Option<i64>, _>("current_revision_id")
            .map_err(sqlx_error)?
            .map(|value| from_i64("current_revision_id", value))
            .transpose()?,
        publish_state: publish_state_from_str(&publish_state)?,
    })
}

fn log_entry_from_row(row: &AnyRow) -> Result<WikiLogEntry, KnowledgeWikiPageStoreError> {
    let event_type: String = row.try_get("event_type").map_err(sqlx_error)?;
    let metadata: Option<String> = row.try_get("metadata").map_err(sqlx_error)?;
    let metadata = log_metadata_from_json(metadata)?;
    Ok(WikiLogEntry {
        occurred_at: row.try_get("event_time").map_err(sqlx_error)?,
        event_type: log_event_type_from_str(&event_type)?,
        title: row.try_get("title").map_err(sqlx_error)?,
        actor: metadata.actor,
        affected_pages: metadata.affected_pages,
        audit_event_id: metadata.audit_event_id,
        warnings: metadata.warnings,
    })
}

fn page_type_from_str(value: &str) -> Result<WikiPageType, KnowledgeWikiPageStoreError> {
    match value {
        "source" => Ok(WikiPageType::Source),
        "entity" => Ok(WikiPageType::Entity),
        "topic" => Ok(WikiPageType::Topic),
        "concept" => Ok(WikiPageType::Concept),
        "how_to" => Ok(WikiPageType::HowTo),
        "reference" => Ok(WikiPageType::Reference),
        "faq" => Ok(WikiPageType::Faq),
        "glossary" => Ok(WikiPageType::Glossary),
        "answer" => Ok(WikiPageType::Answer),
        "comparison" => Ok(WikiPageType::Comparison),
        "presentation" => Ok(WikiPageType::Presentation),
        "chart" => Ok(WikiPageType::Chart),
        "index" => Ok(WikiPageType::Index),
        "policy" => Ok(WikiPageType::Policy),
        "runbook" => Ok(WikiPageType::Runbook),
        _ => Err(KnowledgeWikiPageStoreError::Internal(format!(
            "unknown wiki page type: {value}"
        ))),
    }
}

fn publish_state_from_str(
    value: &str,
) -> Result<WikiPagePublishState, KnowledgeWikiPageStoreError> {
    match value {
        "draft" => Ok(WikiPagePublishState::Draft),
        "candidate_ready" => Ok(WikiPagePublishState::CandidateReady),
        "needs_review" => Ok(WikiPagePublishState::NeedsReview),
        "published" => Ok(WikiPagePublishState::Published),
        "stale" => Ok(WikiPagePublishState::Stale),
        "rejected" => Ok(WikiPagePublishState::Rejected),
        "failed" => Ok(WikiPagePublishState::Failed),
        _ => Err(KnowledgeWikiPageStoreError::Internal(format!(
            "unknown wiki publish state: {value}"
        ))),
    }
}

fn review_state_from_str(
    value: &str,
) -> Result<WikiRevisionReviewState, KnowledgeWikiPageStoreError> {
    match value {
        "pending" => Ok(WikiRevisionReviewState::Pending),
        "approved" => Ok(WikiRevisionReviewState::Approved),
        "rejected" => Ok(WikiRevisionReviewState::Rejected),
        _ => Err(KnowledgeWikiPageStoreError::Internal(format!(
            "unknown wiki revision review state: {value}"
        ))),
    }
}

fn log_event_type_from_str(value: &str) -> Result<WikiLogEventType, KnowledgeWikiPageStoreError> {
    match value {
        "ingest" => Ok(WikiLogEventType::Ingest),
        "query" => Ok(WikiLogEventType::Query),
        "filed_answer" => Ok(WikiLogEventType::FiledAnswer),
        "compile" => Ok(WikiLogEventType::Compile),
        "review" => Ok(WikiLogEventType::Review),
        "publish" => Ok(WikiLogEventType::Publish),
        "lint" => Ok(WikiLogEventType::Lint),
        "eval" => Ok(WikiLogEventType::Eval),
        "package" => Ok(WikiLogEventType::Package),
        "mirror" => Ok(WikiLogEventType::Mirror),
        "delta_update" => Ok(WikiLogEventType::DeltaUpdate),
        _ => Err(KnowledgeWikiPageStoreError::Internal(format!(
            "unknown wiki log event type: {value}"
        ))),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogMetadata {
    actor: String,
    affected_pages: Vec<String>,
    audit_event_id: Option<String>,
    warnings: Vec<String>,
}

fn tags_to_json(tags: &[String]) -> Result<String, KnowledgeWikiPageStoreError> {
    serde_json::to_string(tags)
        .map_err(|error| KnowledgeWikiPageStoreError::Internal(error.to_string()))
}

fn tags_from_json(value: Option<String>) -> Result<Vec<String>, KnowledgeWikiPageStoreError> {
    match value {
        Some(value) if !value.trim().is_empty() => serde_json::from_str(&value)
            .map_err(|error| KnowledgeWikiPageStoreError::Internal(error.to_string())),
        _ => Ok(vec![]),
    }
}

fn log_metadata_to_json(
    actor: &str,
    affected_pages: &[String],
    audit_event_id: Option<&str>,
    warnings: &[String],
) -> Result<String, KnowledgeWikiPageStoreError> {
    serde_json::to_string(&LogMetadata {
        actor: actor.to_string(),
        affected_pages: affected_pages.to_vec(),
        audit_event_id: audit_event_id.map(str::to_string),
        warnings: warnings.to_vec(),
    })
    .map_err(|error| KnowledgeWikiPageStoreError::Internal(error.to_string()))
}

fn log_metadata_from_json(
    value: Option<String>,
) -> Result<LogMetadata, KnowledgeWikiPageStoreError> {
    match value {
        Some(value) if !value.trim().is_empty() => serde_json::from_str(&value)
            .map_err(|error| KnowledgeWikiPageStoreError::Internal(error.to_string())),
        _ => Ok(LogMetadata {
            actor: "system".to_string(),
            affected_pages: vec![],
            audit_event_id: None,
            warnings: vec![],
        }),
    }
}

fn now_rfc3339() -> Result<String, KnowledgeWikiPageStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeWikiPageStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeWikiPageStoreError> {
    i64::try_from(value)
        .map_err(|_| KnowledgeWikiPageStoreError::Internal(format!("{field} is out of range")))
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeWikiPageStoreError> {
    u64::try_from(value)
        .map_err(|_| KnowledgeWikiPageStoreError::Internal(format!("{field} is negative")))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeWikiPageStoreError {
    KnowledgeWikiPageStoreError::Internal(error.to_string())
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeWikiPageStoreError {
    KnowledgeWikiPageStoreError::Internal(error.to_string())
}
