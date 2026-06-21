use async_trait::async_trait;
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptPublishState, OkfConceptSummary,
    OkfLogEntry, OkfRevisionReviewState,
};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeOkfConceptStore: Send + Sync {
    async fn upsert_concept(
        &self,
        record: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError>;

    async fn create_revision(
        &self,
        record: CreateKnowledgeOkfConceptRevisionRecord,
    ) -> Result<KnowledgeOkfConceptRevision, KnowledgeOkfConceptStoreError>;

    async fn next_revision_no(
        &self,
        concept_row_id: u64,
    ) -> Result<u64, KnowledgeOkfConceptStoreError>;

    async fn mark_current_revision(
        &self,
        record: MarkKnowledgeOkfConceptCurrentRevisionRecord,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError>;

    async fn list_concept_summaries(
        &self,
        space_id: u64,
    ) -> Result<Vec<OkfConceptSummary>, KnowledgeOkfConceptStoreError>;

    async fn append_log_entry(
        &self,
        record: AppendKnowledgeOkfLogEntryRecord,
    ) -> Result<OkfLogEntry, KnowledgeOkfConceptStoreError>;

    async fn list_log_entries(
        &self,
        space_id: u64,
    ) -> Result<Vec<OkfLogEntry>, KnowledgeOkfConceptStoreError>;

    async fn batch_concept_projections_by_paths(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeOkfConceptProjection>, KnowledgeOkfConceptStoreError>;

    async fn mark_concept_deleted(
        &self,
        space_id: u64,
        concept_row_id: u64,
    ) -> Result<KnowledgeOkfConcept, KnowledgeOkfConceptStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertKnowledgeOkfConceptRecord {
    pub space_id: u64,
    pub concept_id: String,
    pub title: String,
    pub concept_type: String,
    pub logical_path: String,
    pub description: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub publish_state: OkfConceptPublishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeOkfConceptRevisionRecord {
    pub concept_row_id: u64,
    pub revision_no: u64,
    pub markdown_object_ref_id: u64,
    pub content_hash: String,
    pub review_state: OkfRevisionReviewState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkKnowledgeOkfConceptCurrentRevisionRecord {
    pub concept_row_id: u64,
    pub revision_id: u64,
    pub publish_state: OkfConceptPublishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendKnowledgeOkfLogEntryRecord {
    pub space_id: u64,
    pub event_type: String,
    pub event_time: String,
    pub title: String,
    pub actor: String,
    pub affected_concepts: Vec<String>,
    pub audit_event_id: Option<String>,
    pub warnings: Vec<String>,
    pub privacy_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOkfConceptProjection {
    pub logical_path: String,
    pub concept_row_id: u64,
    pub current_revision_id: Option<u64>,
    pub publish_state: OkfConceptPublishState,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeOkfConceptStoreError {
    #[error("okf concept store internal error: {0}")]
    Internal(String),
}
