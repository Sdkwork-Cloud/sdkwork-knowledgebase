use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateListItem, KnowledgeOkfCandidateStore, KnowledgeOkfCandidateStoreError,
    UpsertKnowledgeOkfCandidateRecord,
};
use sdkwork_knowledgebase_contract::{OkfCandidateType, OkfConceptPublishState};
use sqlx::AnyPool;
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;
const MAX_CANDIDATE_ROWS: i64 = 200;

#[derive(Debug, Clone)]
pub struct SqliteKnowledgeOkfCandidateStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteKnowledgeOkfCandidateStore {
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
impl KnowledgeOkfCandidateStore for SqliteKnowledgeOkfCandidateStore {
    async fn upsert_candidate(
        &self,
        record: UpsertKnowledgeOkfCandidateRecord,
    ) -> Result<(), KnowledgeOkfCandidateStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let space_id = to_i64("space_id", record.space_id)?;
        let concept_row_id = to_i64("concept_row_id", record.concept_row_id)?;
        let markdown_object_ref_id =
            to_i64("markdown_object_ref_id", record.markdown_object_ref_id)?;
        let now = now_rfc3339()?;

        let existing_id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM kb_okf_candidate
            WHERE tenant_id = $1 AND space_id = $2 AND concept_id = $3 AND status = $4
            "#,
        )
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.concept_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?;

        if let Some(candidate_id) = existing_id {
            sqlx::query(
                r#"
                UPDATE kb_okf_candidate
                SET candidate_type = $1,
                    state = $2,
                    markdown_object_ref_id = $3,
                    updated_at = $4,
                    version = version + 1
                WHERE tenant_id = $5 AND id = $6 AND status = $7
                "#,
            )
            .bind(record.candidate_type.as_str())
            .bind(record.state.as_str())
            .bind(markdown_object_ref_id)
            .bind(&now)
            .bind(tenant_id)
            .bind(candidate_id)
            .bind(ACTIVE_STATUS)
            .execute(&self.pool)
            .await
            .map_err(sqlx_error)?;
            return Ok(());
        }

        let id = next_i64_id(&self.id_generator).map_err(id_error)?;
        sqlx::query(
            r#"
            INSERT INTO kb_okf_candidate (
                id, uuid, tenant_id, space_id, concept_id, candidate_type, state,
                markdown_object_ref_id, reviewer_id, review_note, status,
                created_at, updated_at, version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, NULL, $9, $10, $11, $12)
            "#,
        )
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(&record.concept_id)
        .bind(record.candidate_type.as_str())
        .bind(record.state.as_str())
        .bind(markdown_object_ref_id)
        .bind(ACTIVE_STATUS)
        .bind(&now)
        .bind(&now)
        .bind(INITIAL_VERSION)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;
        Ok(())
    }

    async fn update_candidate_state_by_concept_row_id(
        &self,
        concept_row_id: u64,
        state: OkfConceptPublishState,
        reviewer_id: Option<u64>,
        review_note: Option<String>,
    ) -> Result<(), KnowledgeOkfCandidateStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let concept_row_id = to_i64("concept_row_id", concept_row_id)?;
        let concept_id: String = sqlx::query_scalar(
            r#"
            SELECT concept_id
            FROM kb_okf_concept
            WHERE tenant_id = $1 AND id = $2 AND status = $3
            "#,
        )
        .bind(tenant_id)
        .bind(concept_row_id)
        .bind(ACTIVE_STATUS)
        .fetch_optional(&self.pool)
        .await
        .map_err(sqlx_error)?
        .ok_or_else(|| {
            KnowledgeOkfCandidateStoreError::Internal(format!(
                "missing okf concept row: {concept_row_id}"
            ))
        })?;
        let space_id: i64 = sqlx::query_scalar(
            r#"
            SELECT space_id
            FROM kb_okf_concept
            WHERE tenant_id = $1 AND id = $2 AND status = $3
            "#,
        )
        .bind(tenant_id)
        .bind(concept_row_id)
        .bind(ACTIVE_STATUS)
        .fetch_one(&self.pool)
        .await
        .map_err(sqlx_error)?;

        let reviewer_id = reviewer_id.map(|value| to_i64("reviewer_id", value)).transpose()?;
        let now = now_rfc3339()?;
        sqlx::query(
            r#"
            UPDATE kb_okf_candidate
            SET state = $1,
                reviewer_id = $2,
                review_note = $3,
                updated_at = $4,
                version = version + 1
            WHERE tenant_id = $5 AND space_id = $6 AND concept_id = $7 AND status = $8
            "#,
        )
        .bind(state.as_str())
        .bind(reviewer_id)
        .bind(review_note)
        .bind(&now)
        .bind(tenant_id)
        .bind(space_id)
        .bind(concept_id)
        .bind(ACTIVE_STATUS)
        .execute(&self.pool)
        .await
        .map_err(sqlx_error)?;
        Ok(())
    }

    async fn list_open_candidates(
        &self,
        space_id: Option<u64>,
    ) -> Result<Vec<KnowledgeOkfCandidateListItem>, KnowledgeOkfCandidateStoreError> {
        let tenant_id = to_i64("tenant_id", self.tenant_id)?;
        let rows = if let Some(space_id) = space_id {
            let space_id = to_i64("space_id", space_id)?;
            sqlx::query_as::<_, (i64, String)>(
                r#"
                SELECT c.id, c.publish_state
                FROM kb_okf_concept c
                INNER JOIN kb_okf_candidate k
                  ON k.tenant_id = c.tenant_id
                 AND k.space_id = c.space_id
                 AND k.concept_id = c.concept_id
                 AND k.status = $2
                WHERE c.tenant_id = $1
                  AND c.space_id = $3
                  AND c.status = $2
                  AND k.state IN ('candidate_ready', 'needs_review')
                ORDER BY c.id ASC
                LIMIT $4
                "#,
            )
            .bind(tenant_id)
            .bind(ACTIVE_STATUS)
            .bind(space_id)
            .bind(MAX_CANDIDATE_ROWS)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        } else {
            sqlx::query_as::<_, (i64, String)>(
                r#"
                SELECT c.id, c.publish_state
                FROM kb_okf_concept c
                INNER JOIN kb_okf_candidate k
                  ON k.tenant_id = c.tenant_id
                 AND k.space_id = c.space_id
                 AND k.concept_id = c.concept_id
                 AND k.status = $2
                WHERE c.tenant_id = $1
                  AND c.status = $2
                  AND k.state IN ('candidate_ready', 'needs_review')
                ORDER BY c.id ASC
                LIMIT $3
                "#,
            )
            .bind(tenant_id)
            .bind(ACTIVE_STATUS)
            .bind(MAX_CANDIDATE_ROWS)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        };

        rows.into_iter()
            .map(|(concept_row_id, publish_state)| {
                Ok(KnowledgeOkfCandidateListItem {
                    concept_row_id: from_i64("concept_row_id", concept_row_id)?,
                    publish_state: publish_state_from_str(&publish_state)?,
                })
            })
            .collect()
    }
}

fn now_rfc3339() -> Result<String, KnowledgeOkfCandidateStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOkfCandidateStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOkfCandidateStoreError> {
    i64::try_from(value).map_err(|_| {
        KnowledgeOkfCandidateStoreError::Internal(format!("{field} is out of range"))
    })
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeOkfCandidateStoreError> {
    u64::try_from(value).map_err(|_| {
        KnowledgeOkfCandidateStoreError::Internal(format!("{field} is out of range"))
    })
}

fn publish_state_from_str(value: &str) -> Result<OkfConceptPublishState, KnowledgeOkfCandidateStoreError> {
    match value {
        "draft" => Ok(OkfConceptPublishState::Draft),
        "candidate_ready" => Ok(OkfConceptPublishState::CandidateReady),
        "needs_review" => Ok(OkfConceptPublishState::NeedsReview),
        "published" => Ok(OkfConceptPublishState::Published),
        "stale" => Ok(OkfConceptPublishState::Stale),
        "rejected" => Ok(OkfConceptPublishState::Rejected),
        "failed" => Ok(OkfConceptPublishState::Failed),
        other => Err(KnowledgeOkfCandidateStoreError::Internal(format!(
            "unknown okf publish state: {other}"
        ))),
    }
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeOkfCandidateStoreError {
    KnowledgeOkfCandidateStoreError::Internal(error.to_string())
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeOkfCandidateStoreError {
    KnowledgeOkfCandidateStoreError::Internal(error.to_string())
}
