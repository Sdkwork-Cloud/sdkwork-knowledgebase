use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::drive_import_metadata_store::DriveImportMetadataStoreError;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_candidate_store::KnowledgeOkfCandidateStoreError;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::{
    MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
};
use sdkwork_intelligence_knowledgebase_service::ports::okf_concept_revision_metadata_store::{
    OkfConceptRevisionMetadataStore, OkfConceptRevisionMetadataStoreError,
    PreparedOkfConceptRevisionSlot, PublishOkfConceptRevisionMetadataRecord,
    PublishedOkfConceptRevisionMetadata, StageOkfConceptRevisionMetadataRecord,
    StagedOkfConceptRevisionMetadata,
};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptPublishState,
    OkfRevisionReviewState,
};
use sqlx::{any::AnyRow, Any, AnyPool, Row, Transaction};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::id::{default_knowledge_id_generator, next_i64_id, KnowledgeIdGenerator};
use crate::sqlite_knowledge_document_metadata_transaction::create_or_get_object_ref_in_transaction;
use crate::sqlite_okf_candidate_transaction::{
    update_okf_candidate_state_by_concept_row_id_in_transaction,
    upsert_okf_candidate_in_transaction,
};
use crate::sqlite_okf_concept_transaction::{
    next_okf_revision_no_in_transaction, upsert_okf_concept_in_transaction,
};

const ACTIVE_STATUS: i64 = 1;
const INITIAL_VERSION: i64 = 0;

#[derive(Debug, Clone)]
pub struct SqliteOkfConceptRevisionMetadataStore {
    pool: AnyPool,
    tenant_id: u64,
    id_generator: Arc<dyn KnowledgeIdGenerator>,
}

impl SqliteOkfConceptRevisionMetadataStore {
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
impl OkfConceptRevisionMetadataStore for SqliteOkfConceptRevisionMetadataStore {
    async fn prepare_concept_revision_slot(
        &self,
        concept: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<PreparedOkfConceptRevisionSlot, OkfConceptRevisionMetadataStoreError> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                OkfConceptRevisionMetadataStoreError::internal(error.to_string())
            })?;

        let concept = upsert_okf_concept_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            concept,
        )
        .await
        .map_err(map_concept_store_error)?;

        let revision_no =
            next_okf_revision_no_in_transaction(&mut transaction, self.tenant_id, concept.id)
                .await
                .map_err(map_concept_store_error)?;

        transaction
            .commit()
            .await
            .map_err(|error| OkfConceptRevisionMetadataStoreError::internal(error.to_string()))?;

        Ok(PreparedOkfConceptRevisionSlot {
            concept,
            revision_no,
        })
    }

    async fn stage_concept_revision_metadata(
        &self,
        record: StageOkfConceptRevisionMetadataRecord,
    ) -> Result<StagedOkfConceptRevisionMetadata, OkfConceptRevisionMetadataStoreError> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                OkfConceptRevisionMetadataStoreError::internal(error.to_string())
            })?;

        let revision_object_ref = create_or_get_object_ref_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &record.revision_object_ref,
        )
        .await
        .map_err(map_object_ref_error)?;

        let published_object_ref = match &record.published_object_ref {
            Some(published_record) => Some(
                create_or_get_object_ref_in_transaction(
                    &mut transaction,
                    self.tenant_id,
                    &self.id_generator,
                    published_record,
                )
                .await
                .map_err(map_object_ref_error)?,
            ),
            None => None,
        };

        let revision = create_okf_revision_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            record.concept_row_id,
            record.revision_no,
            revision_object_ref.id,
            &record.content_hash,
            record.review_state,
        )
        .await?;

        let concept = mark_okf_current_revision_in_transaction(
            &mut transaction,
            self.tenant_id,
            MarkKnowledgeOkfConceptCurrentRevisionRecord {
                concept_row_id: record.concept_row_id,
                revision_id: revision.id,
                publish_state: record.publish_state,
            },
        )
        .await?;

        if let Some(candidate) = record.candidate {
            let mut candidate = candidate;
            if candidate.markdown_object_ref_id == 0 {
                candidate.markdown_object_ref_id = revision_object_ref.id;
            }
            upsert_okf_candidate_in_transaction(
                &mut transaction,
                self.tenant_id,
                &self.id_generator,
                candidate,
            )
            .await
            .map_err(map_candidate_store_error)?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| OkfConceptRevisionMetadataStoreError::internal(error.to_string()))?;

        Ok(StagedOkfConceptRevisionMetadata {
            revision,
            concept,
            revision_object_ref,
            published_object_ref,
        })
    }

    async fn publish_existing_revision_metadata(
        &self,
        record: PublishOkfConceptRevisionMetadataRecord,
    ) -> Result<PublishedOkfConceptRevisionMetadata, OkfConceptRevisionMetadataStoreError> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                OkfConceptRevisionMetadataStoreError::internal(error.to_string())
            })?;

        let published_object_ref = create_or_get_object_ref_in_transaction(
            &mut transaction,
            self.tenant_id,
            &self.id_generator,
            &record.published_object_ref,
        )
        .await
        .map_err(map_object_ref_error)?;

        let concept = mark_okf_current_revision_in_transaction(
            &mut transaction,
            self.tenant_id,
            record.mark_current,
        )
        .await?;

        if let Some(candidate_state_update) = record.candidate_state_update {
            update_okf_candidate_state_by_concept_row_id_in_transaction(
                &mut transaction,
                self.tenant_id,
                candidate_state_update.concept_row_id,
                candidate_state_update.state,
                candidate_state_update.reviewer_id,
                candidate_state_update.review_note,
            )
            .await
            .map_err(map_candidate_store_error)?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| OkfConceptRevisionMetadataStoreError::internal(error.to_string()))?;

        Ok(PublishedOkfConceptRevisionMetadata {
            concept,
            published_object_ref,
        })
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_okf_revision_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    id_generator: &Arc<dyn KnowledgeIdGenerator>,
    concept_row_id: u64,
    revision_no: u64,
    markdown_object_ref_id: u64,
    content_hash: &str,
    review_state: OkfRevisionReviewState,
) -> Result<KnowledgeOkfConceptRevision, OkfConceptRevisionMetadataStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let concept_row_id = to_i64("concept_row_id", concept_row_id)?;
    let revision_no = to_i64("revision_no", revision_no)?;
    let markdown_object_ref_id = to_i64("markdown_object_ref_id", markdown_object_ref_id)?;
    let id = next_i64_id(id_generator).map_err(id_error)?;
    let now = now_rfc3339()?;

    let row = sqlx::query(
        r#"
        INSERT INTO kb_okf_concept_revision (
            id, uuid, tenant_id, concept_row_id, revision_no, markdown_object_ref_id,
            content_hash, review_state, status, created_at, updated_at, version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id, concept_row_id, revision_no, markdown_object_ref_id, content_hash,
            review_state, created_at
        "#,
    )
    .bind(id)
    .bind(Uuid::new_v4().to_string())
    .bind(tenant_id)
    .bind(concept_row_id)
    .bind(revision_no)
    .bind(markdown_object_ref_id)
    .bind(content_hash)
    .bind(review_state.as_str())
    .bind(ACTIVE_STATUS)
    .bind(now.clone())
    .bind(now)
    .bind(INITIAL_VERSION)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    revision_from_row(&row)
}

async fn mark_okf_current_revision_in_transaction(
    transaction: &mut Transaction<'_, Any>,
    tenant_id: u64,
    record: MarkKnowledgeOkfConceptCurrentRevisionRecord,
) -> Result<KnowledgeOkfConcept, OkfConceptRevisionMetadataStoreError> {
    let tenant_id = to_i64("tenant_id", tenant_id)?;
    let concept_row_id = to_i64("concept_row_id", record.concept_row_id)?;
    let revision_id = to_i64("revision_id", record.revision_id)?;
    let now = now_rfc3339()?;

    let row = sqlx::query(
        r#"
        UPDATE kb_okf_concept
        SET current_revision_id = $1,
            publish_state = $2,
            updated_at = $3,
            version = version + 1
        WHERE tenant_id = $4 AND id = $5 AND status = $6
        RETURNING
            id, space_id, concept_id, title, concept_type, logical_path, description,
            source_count, tags, current_revision_id, publish_state, updated_at
        "#,
    )
    .bind(revision_id)
    .bind(record.publish_state.as_str())
    .bind(now)
    .bind(tenant_id)
    .bind(concept_row_id)
    .bind(ACTIVE_STATUS)
    .fetch_one(&mut **transaction)
    .await
    .map_err(sqlx_error)?;

    concept_from_row(&row)
}

fn concept_from_row(
    row: &AnyRow,
) -> Result<KnowledgeOkfConcept, OkfConceptRevisionMetadataStoreError> {
    let concept_type: String = row.try_get("concept_type").map_err(sqlx_error)?;
    let publish_state: String = row.try_get("publish_state").map_err(sqlx_error)?;
    let source_count: i64 = row.try_get("source_count").map_err(sqlx_error)?;
    let tags_json: String = row.try_get("tags").map_err(sqlx_error)?;
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
            OkfConceptRevisionMetadataStoreError::internal("source_count is out of range")
        })?,
        tags: serde_json::from_str(&tags_json)
            .map_err(|error| OkfConceptRevisionMetadataStoreError::internal(error.to_string()))?,
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
) -> Result<KnowledgeOkfConceptRevision, OkfConceptRevisionMetadataStoreError> {
    let review_state: String = row.try_get("review_state").map_err(sqlx_error)?;
    Ok(KnowledgeOkfConceptRevision {
        id: from_i64("id", row.try_get("id").map_err(sqlx_error)?)?,
        concept_row_id: from_i64(
            "concept_row_id",
            row.try_get("concept_row_id").map_err(sqlx_error)?,
        )?,
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

fn bundle_relative_path_from_logical_path(logical_path: &str) -> String {
    logical_path
        .strip_prefix("okf/")
        .unwrap_or(logical_path)
        .to_string()
}

fn publish_state_from_str(
    value: &str,
) -> Result<OkfConceptPublishState, OkfConceptRevisionMetadataStoreError> {
    match value {
        "draft" => Ok(OkfConceptPublishState::Draft),
        "candidate_ready" => Ok(OkfConceptPublishState::CandidateReady),
        "needs_review" => Ok(OkfConceptPublishState::NeedsReview),
        "published" => Ok(OkfConceptPublishState::Published),
        "stale" => Ok(OkfConceptPublishState::Stale),
        "rejected" => Ok(OkfConceptPublishState::Rejected),
        "failed" => Ok(OkfConceptPublishState::Failed),
        _ => Err(OkfConceptRevisionMetadataStoreError::internal(format!(
            "unknown okf concept publish state: {value}"
        ))),
    }
}

fn review_state_from_str(
    value: &str,
) -> Result<OkfRevisionReviewState, OkfConceptRevisionMetadataStoreError> {
    match value {
        "pending" => Ok(OkfRevisionReviewState::Pending),
        "approved" => Ok(OkfRevisionReviewState::Approved),
        "rejected" => Ok(OkfRevisionReviewState::Rejected),
        _ => Err(OkfConceptRevisionMetadataStoreError::internal(format!(
            "unknown okf revision review state: {value}"
        ))),
    }
}

fn map_object_ref_error(
    error: DriveImportMetadataStoreError,
) -> OkfConceptRevisionMetadataStoreError {
    match error {
        DriveImportMetadataStoreError::InvalidRequest(detail) => {
            OkfConceptRevisionMetadataStoreError::invalid_request(detail)
        }
        DriveImportMetadataStoreError::Conflict(detail) => {
            OkfConceptRevisionMetadataStoreError::internal(detail)
        }
        DriveImportMetadataStoreError::Internal(detail) => {
            OkfConceptRevisionMetadataStoreError::internal(detail)
        }
    }
}

fn map_candidate_store_error(
    error: KnowledgeOkfCandidateStoreError,
) -> OkfConceptRevisionMetadataStoreError {
    OkfConceptRevisionMetadataStoreError::internal(error.to_string())
}

fn map_concept_store_error(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> OkfConceptRevisionMetadataStoreError {
    OkfConceptRevisionMetadataStoreError::internal(error.to_string())
}

fn now_rfc3339() -> Result<String, OkfConceptRevisionMetadataStoreError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| OkfConceptRevisionMetadataStoreError::internal(error.to_string()))
}

fn to_i64(field: &str, value: u64) -> Result<i64, OkfConceptRevisionMetadataStoreError> {
    i64::try_from(value).map_err(|_| {
        OkfConceptRevisionMetadataStoreError::internal(format!("{field} is out of range"))
    })
}

fn from_i64(field: &str, value: i64) -> Result<u64, OkfConceptRevisionMetadataStoreError> {
    u64::try_from(value)
        .map_err(|_| OkfConceptRevisionMetadataStoreError::internal(format!("{field} is negative")))
}

fn id_error(error: crate::KnowledgeIdGeneratorError) -> OkfConceptRevisionMetadataStoreError {
    OkfConceptRevisionMetadataStoreError::internal(error.to_string())
}

fn sqlx_error(error: sqlx::Error) -> OkfConceptRevisionMetadataStoreError {
    OkfConceptRevisionMetadataStoreError::internal(error.to_string())
}
