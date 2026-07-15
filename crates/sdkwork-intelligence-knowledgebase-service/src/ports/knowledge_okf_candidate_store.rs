use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{OkfCandidateType, OkfConceptPublishState};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeOkfCandidateStore: Send + Sync {
    async fn upsert_candidate(
        &self,
        record: UpsertKnowledgeOkfCandidateRecord,
    ) -> Result<(), KnowledgeOkfCandidateStoreError>;

    async fn update_candidate_state_by_concept_row_id(
        &self,
        concept_row_id: u64,
        state: OkfConceptPublishState,
        reviewer_id: Option<u64>,
        review_note: Option<String>,
    ) -> Result<(), KnowledgeOkfCandidateStoreError>;

    async fn list_open_candidates(
        &self,
        space_id: Option<u64>,
    ) -> Result<Vec<KnowledgeOkfCandidateListItem>, KnowledgeOkfCandidateStoreError>;

    async fn list_open_candidates_page(
        &self,
        space_id: Option<u64>,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<
        (Vec<KnowledgeOkfCandidateListItem>, Option<String>, bool),
        KnowledgeOkfCandidateStoreError,
    > {
        let _ = (space_id, cursor, page_size);
        Err(KnowledgeOkfCandidateStoreError::Internal(
            "paginated OKF candidate listing is unsupported by this store".to_string(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertKnowledgeOkfCandidateRecord {
    pub space_id: u64,
    pub concept_row_id: u64,
    pub concept_id: String,
    pub candidate_type: OkfCandidateType,
    pub state: OkfConceptPublishState,
    pub markdown_object_ref_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOkfCandidateListItem {
    pub concept_row_id: u64,
    pub publish_state: OkfConceptPublishState,
}

#[derive(Debug, Error)]
pub enum KnowledgeOkfCandidateStoreError {
    #[error("internal knowledge okf candidate store error: {0}")]
    Internal(String),
}
