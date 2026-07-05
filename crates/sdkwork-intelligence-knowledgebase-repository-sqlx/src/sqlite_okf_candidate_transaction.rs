//! Shared SQLite transaction helpers for OKF candidate rows.

use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::{
    KnowledgeOkfCandidateStoreError, UpsertKnowledgeOkfCandidateRecord,
};
use sdkwork_knowledgebase_contract::OkfConceptPublishState;
use sqlx::{Any, Transaction};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{next_i64_id, KnowledgeIdGenerator};

pub(crate) const OKF_CANDIDATE_ACTIVE_STATUS: i64 = 1;
pub(crate) const OKF_CANDIDATE_INITIAL_VERSION: i64 = 0;

pub(crate) async fn upsert_okf_candidate_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    record: UpsertKnowledgeOkfCandidateRecord,
) -> Result<(), KnowledgeOkfCandidateStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let space_id = to_i64("space_id", record.space_id)?;
    let _concept_row_id = to_i64("concept_row_id", record.concept_row_id)?;
    let markdown_object_ref_id = to_i64("markdown_object_ref_id", record.markdown_object_ref_id)?;
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
    .bind(OKF_CANDIDATE_ACTIVE_STATUS)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    if let Some(candidate_id) = existing_id {
        sqlx::query(
            r#"
            UPDATE kb_okf_candidate
            SET candidate_type = $1,
                state = $2,
                markdown_object_ref_id = $3,
                updated_at = CAST($4 AS TIMESTAMP),
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
        .bind(OKF_CANDIDATE_ACTIVE_STATUS)
        .execute(&mut **transaction)
        .await
        .map_err(sqlx_error)?;
        return Ok(());
    }

    let id = next_i64_id(id_generator).map_err(id_error)?;
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
    .bind(OKF_CANDIDATE_ACTIVE_STATUS)
    .bind(&now)
    .bind(&now)
    .bind(OKF_CANDIDATE_INITIAL_VERSION)
    .execute(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    Ok(())
}

pub(crate) async fn update_okf_candidate_state_by_concept_row_id_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    concept_row_id: u64,
    state: OkfConceptPublishState,
    reviewer_id: Option<u64>,
    review_note: Option<String>,
) -> Result<(), KnowledgeOkfCandidateStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
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
    .bind(OKF_CANDIDATE_ACTIVE_STATUS)
    .fetch_optional(&mut **transaction)
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
    .bind(OKF_CANDIDATE_ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    let reviewer_id = reviewer_id
        .map(|value| to_i64("reviewer_id", value))
        .transpose()?;
    let now = now_rfc3339()?;
    sqlx::query(
        r#"
        UPDATE kb_okf_candidate
        SET state = $1,
            reviewer_id = $2,
            review_note = $3,
            updated_at = CAST($4 AS TIMESTAMP),
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
    .bind(OKF_CANDIDATE_ACTIVE_STATUS)
    .execute(&mut **transaction)
    .await
    .map_err(sqlx_error)?;
    Ok(())
}

fn now_rfc3339() -> Result<String, KnowledgeOkfCandidateStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOkfCandidateStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOkfCandidateStoreError> {
    i64::try_from(value)
        .map_err(|_| KnowledgeOkfCandidateStoreError::Internal(format!("{field} is out of range")))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeOkfCandidateStoreError {
    KnowledgeOkfCandidateStoreError::Internal(error.to_string())
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeOkfCandidateStoreError {
    KnowledgeOkfCandidateStoreError::Internal(error.to_string())
}
