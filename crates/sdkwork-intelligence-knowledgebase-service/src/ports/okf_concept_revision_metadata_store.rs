use async_trait::async_trait;
use sdkwork_knowledgebase_contract::drive::KnowledgeDriveObjectRef;
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, KnowledgeOkfConceptRevision, OkfConceptPublishState,
    OkfRevisionReviewState,
};
use thiserror::Error;

use crate::tenant_quota::TenantQuotaExceeded;

use super::{
    knowledge_drive_object_ref_store::CreateKnowledgeDriveObjectRefRecord,
    knowledge_okf_candidate_store::UpsertKnowledgeOkfCandidateRecord,
    knowledge_okf_concept_store::{
        MarkKnowledgeOkfConceptCurrentRevisionRecord, UpsertKnowledgeOkfConceptRecord,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedOkfConceptRevisionSlot {
    pub concept: KnowledgeOkfConcept,
    pub revision_no: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageOkfConceptRevisionMetadataRecord {
    pub revision_object_ref: CreateKnowledgeDriveObjectRefRecord,
    pub published_object_ref: Option<CreateKnowledgeDriveObjectRefRecord>,
    pub concept_row_id: u64,
    pub revision_no: u64,
    pub content_hash: String,
    pub review_state: OkfRevisionReviewState,
    pub publish_state: OkfConceptPublishState,
    pub candidate: Option<UpsertKnowledgeOkfCandidateRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedOkfConceptRevisionMetadata {
    pub revision: KnowledgeOkfConceptRevision,
    pub concept: KnowledgeOkfConcept,
    pub revision_object_ref: KnowledgeDriveObjectRef,
    pub published_object_ref: Option<KnowledgeDriveObjectRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOkfConceptCandidateStateRecord {
    pub concept_row_id: u64,
    pub state: OkfConceptPublishState,
    pub reviewer_id: Option<u64>,
    pub review_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOkfConceptRevisionMetadataRecord {
    pub published_object_ref: CreateKnowledgeDriveObjectRefRecord,
    pub mark_current: MarkKnowledgeOkfConceptCurrentRevisionRecord,
    pub candidate_state_update: Option<UpdateOkfConceptCandidateStateRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedOkfConceptRevisionMetadata {
    pub concept: KnowledgeOkfConcept,
    pub published_object_ref: KnowledgeDriveObjectRef,
}

#[async_trait]
pub trait OkfConceptRevisionMetadataStore: Send + Sync {
    async fn prepare_concept_revision_slot(
        &self,
        concept: UpsertKnowledgeOkfConceptRecord,
    ) -> Result<PreparedOkfConceptRevisionSlot, OkfConceptRevisionMetadataStoreError>;

    async fn stage_concept_revision_metadata(
        &self,
        record: StageOkfConceptRevisionMetadataRecord,
    ) -> Result<StagedOkfConceptRevisionMetadata, OkfConceptRevisionMetadataStoreError>;

    async fn publish_existing_revision_metadata(
        &self,
        record: PublishOkfConceptRevisionMetadataRecord,
    ) -> Result<PublishedOkfConceptRevisionMetadata, OkfConceptRevisionMetadataStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OkfConceptRevisionMetadataStoreError {
    #[error("invalid okf concept revision metadata request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    QuotaExceeded(#[from] TenantQuotaExceeded),
    #[error("okf concept revision metadata store internal error: {0}")]
    Internal(String),
}

impl OkfConceptRevisionMetadataStoreError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}
