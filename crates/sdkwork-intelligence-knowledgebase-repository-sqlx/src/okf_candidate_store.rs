use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateListItem, KnowledgeOkfCandidateStore, KnowledgeOkfCandidateStoreError,
    UpsertKnowledgeOkfCandidateRecord,
};
use sdkwork_knowledgebase_contract::OkfConceptPublishState;
use sqlx::AnyPool;
use std::sync::Arc;

use crate::id::{default_knowledge_id_generator, KnowledgeIdGenerator};
use crate::sqlite_okf_candidate_transaction::{
    update_okf_candidate_state_by_concept_row_id_in_transaction,
    upsert_okf_candidate_in_transaction, OKF_CANDIDATE_ACTIVE_STATUS,
};

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
        let mut transaction = self.pool.begin().await.map_err(sqlx_error)?;
        upsert_okf_candidate_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            record,
        )
        .await?;
        transaction.commit().await.map_err(sqlx_error)?;
        Ok(())
    }

    async fn update_candidate_state_by_concept_row_id(
        &self,
        concept_row_id: u64,
        state: OkfConceptPublishState,
        reviewer_id: Option<u64>,
        review_note: Option<String>,
    ) -> Result<(), KnowledgeOkfCandidateStoreError> {
        let mut transaction = self.pool.begin().await.map_err(sqlx_error)?;
        update_okf_candidate_state_by_concept_row_id_in_transaction(
            &mut transaction,
            self.tenant_id,
            concept_row_id,
            state,
            reviewer_id,
            review_note,
        )
        .await?;
        transaction.commit().await.map_err(sqlx_error)?;
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
                SELECT c.id, k.state
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
            .bind(OKF_CANDIDATE_ACTIVE_STATUS)
            .bind(space_id)
            .bind(MAX_CANDIDATE_ROWS)
            .fetch_all(&self.pool)
            .await
            .map_err(sqlx_error)?
        } else {
            sqlx::query_as::<_, (i64, String)>(
                r#"
                SELECT c.id, k.state
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
            .bind(OKF_CANDIDATE_ACTIVE_STATUS)
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

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOkfCandidateStoreError> {
    i64::try_from(value)
        .map_err(|_| KnowledgeOkfCandidateStoreError::Internal(format!("{field} is out of range")))
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeOkfCandidateStoreError> {
    u64::try_from(value)
        .map_err(|_| KnowledgeOkfCandidateStoreError::Internal(format!("{field} is out of range")))
}

fn publish_state_from_str(
    value: &str,
) -> Result<OkfConceptPublishState, KnowledgeOkfCandidateStoreError> {
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
