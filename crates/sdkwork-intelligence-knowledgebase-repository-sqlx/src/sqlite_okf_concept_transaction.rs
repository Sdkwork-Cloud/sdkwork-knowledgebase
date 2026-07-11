//! Shared SQLite transaction helpers for OKF concept rows.

use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStoreError, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_knowledgebase_contract::okf::KnowledgeOkfConcept;
use sqlx::{any::AnyRow, Any, Row, Transaction};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::db::sql_timestamp::SqlTimestampDialect;
use crate::id::{next_i64_id, KnowledgeIdGenerator};

pub(crate) const OKF_CONCEPT_ACTIVE_STATUS: i64 = 1;
pub(crate) const OKF_CONCEPT_INITIAL_VERSION: i64 = 0;

pub(crate) async fn upsert_okf_concept_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    timestamp_dialect: SqlTimestampDialect,
    record: UpsertKnowledgeOkfConceptRecord,
) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let space_id = to_i64("space_id", record.space_id)?;
    let source_count = i64::from(record.source_count);
    let now = now_rfc3339()?;
    let concept_type = record.concept_type.as_str();
    let publish_state = record.publish_state.as_str();
    let tags = tags_to_json(&record.tags)?;
    let id = next_i64_id(id_generator).map_err(id_error)?;
    let tags_expr = timestamp_dialect.sql_json_expr("$11");
    let created_at_expr = timestamp_dialect.sql_timestamp_expr("$14");
    let updated_at_expr = timestamp_dialect.sql_timestamp_expr("$15");

    let query = format!(
        r#"
        INSERT INTO kb_okf_concept (
            id,
            uuid,
            tenant_id,
            space_id,
            concept_id,
            title,
            concept_type,
            logical_path,
            description,
            source_count,
            tags,
            current_revision_id,
            publish_state,
            status,
            created_at,
            updated_at,
            version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, {tags_expr}, NULL, $12, $13, {created_at_expr}, {updated_at_expr}, $16)
        ON CONFLICT(tenant_id, space_id, concept_id)
        DO UPDATE SET
            title = excluded.title,
            concept_type = excluded.concept_type,
            logical_path = excluded.logical_path,
            description = excluded.description,
            source_count = excluded.source_count,
            tags = excluded.tags,
            publish_state = excluded.publish_state,
            updated_at = excluded.updated_at,
            version = kb_okf_concept.version + 1
        RETURNING
            id,
            space_id,
            concept_id,
            title,
            concept_type,
            logical_path,
            description,
            source_count,
            CAST(tags AS TEXT) AS tags,
            current_revision_id,
            publish_state,
            updated_at
        "#,
    );
    let row = sqlx::query(&query)
        .bind(id)
        .bind(Uuid::new_v4().to_string())
        .bind(tenant_id)
        .bind(space_id)
        .bind(record.concept_id)
        .bind(record.title)
        .bind(concept_type)
        .bind(record.logical_path)
        .bind(record.description)
        .bind(source_count)
        .bind(tags)
        .bind(publish_state)
        .bind(OKF_CONCEPT_ACTIVE_STATUS)
        .bind(now.clone())
        .bind(now)
        .bind(OKF_CONCEPT_INITIAL_VERSION)
        .fetch_one(&mut **transaction)
        .await
        .map_err(sqlx_error)?;

    concept_from_row(&row)
}

pub(crate) async fn next_okf_revision_no_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    timestamp_dialect: SqlTimestampDialect,
    concept_row_id: u64,
) -> Result<u64, KnowledgeOkfConceptStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let concept_row_id = to_i64("concept_row_id", concept_row_id)?;
    let updated_at_expr = timestamp_dialect.sql_timestamp_expr("$1");
    let query = format!(
        r#"
        UPDATE kb_okf_concept
        SET revision_counter = revision_counter + 1,
            updated_at = {updated_at_expr},
            version = version + 1
        WHERE tenant_id = $2 AND id = $3 AND status = $4
        RETURNING revision_counter
        "#,
    );
    let next: i64 = sqlx::query_scalar(&query)
        .bind(now_rfc3339()?)
        .bind(tenant_id)
        .bind(concept_row_id)
        .bind(OKF_CONCEPT_ACTIVE_STATUS)
        .fetch_one(&mut **transaction)
        .await
        .map_err(sqlx_error)?;
    from_i64("revision_no", next)
}

fn concept_from_row(row: &AnyRow) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError> {
    let concept_type: String = row.try_get("concept_type").map_err(sqlx_error)?;
    let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
    let source_count: i64 = row.try_get("source_count").map_err(sqlx_error)?;
    Ok(KnowledgeOkfConcept {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        space_id: from_i64("space_id", row.try_get("space_id").map_err(sqlx_error)?)?,
        concept_id: row.try_get("concept_id").map_err(sqlx_error)?,
        title: row.try_get("title").map_err(sqlx_error)?,
        concept_type,
        logical_path: row.try_get("logical_path").map_err(sqlx_error)?,
        bundle_relative_path: bundle_relative_path_from_logical_path(
            &row.try_get::<String, _>("logical_path")
                .map_err(sqlx_error)?,
        ),
        description: row.try_get("description").map_err(sqlx_error)?,
        source_count: u32::try_from(source_count).map_err(|_| {
            KnowledgeOkfConceptStoreError::Internal("source_count is out of range".to_string())
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

fn bundle_relative_path_from_logical_path(logical_path: &str) -> String {
    logical_path
        .strip_prefix("okf/")
        .unwrap_or(logical_path)
        .to_string()
}

fn publish_state_from_str(
    value: &str,
) -> Result<sdkwork_knowledgebase_contract::OkfConceptPublishState, KnowledgeOkfConceptStoreError> {
    use sdkwork_knowledgebase_contract::OkfConceptPublishState;
    match value {
        "draft" => Ok(OkfConceptPublishState::Draft),
        "candidate_ready" => Ok(OkfConceptPublishState::CandidateReady),
        "needs_review" => Ok(OkfConceptPublishState::NeedsReview),
        "published" => Ok(OkfConceptPublishState::Published),
        "stale" => Ok(OkfConceptPublishState::Stale),
        "rejected" => Ok(OkfConceptPublishState::Rejected),
        "failed" => Ok(OkfConceptPublishState::Failed),
        other => Err(KnowledgeOkfConceptStoreError::Internal(format!(
            "unknown okf concept publish state: {other}"
        ))),
    }
}

fn tags_to_json(tags: &[String]) -> Result<String, KnowledgeOkfConceptStoreError> {
    serde_json::to_string(tags)
        .map_err(|error| KnowledgeOkfConceptStoreError::Internal(error.to_string()))
}

fn tags_from_json(value: String) -> Result<Vec<String>, KnowledgeOkfConceptStoreError> {
    serde_json::from_str(&value)
        .map_err(|error| KnowledgeOkfConceptStoreError::Internal(error.to_string()))
}

fn now_rfc3339() -> Result<String, KnowledgeOkfConceptStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| KnowledgeOkfConceptStoreError::Internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, KnowledgeOkfConceptStoreError> {
    i64::try_from(value)
        .map_err(|_| KnowledgeOkfConceptStoreError::Internal(format!("{field} is out of range")))
}

fn from_i64(field: &str, value: i64) -> Result<u64, KnowledgeOkfConceptStoreError> {
    u64::try_from(value)
        .map_err(|_| KnowledgeOkfConceptStoreError::Internal(format!("{field} is out of range")))
}

fn sqlx_error(error: sqlx::Error) -> KnowledgeOkfConceptStoreError {
    KnowledgeOkfConceptStoreError::Internal(error.to_string())
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> KnowledgeOkfConceptStoreError {
    KnowledgeOkfConceptStoreError::Internal(error.to_string())
}
