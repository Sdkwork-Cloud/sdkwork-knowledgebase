use async_trait::async_trait;
use sdkwork_knowledgebase_contract::wiki::{
    KnowledgeWikiPage, KnowledgeWikiPageRevision, WikiLogEntry, WikiPagePublishState,
    WikiPageSummary, WikiPageType, WikiRevisionReviewState,
};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeWikiPageStore: Send + Sync {
    async fn upsert_page(
        &self,
        record: UpsertKnowledgeWikiPageRecord,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError>;

    async fn create_revision(
        &self,
        record: CreateKnowledgeWikiPageRevisionRecord,
    ) -> Result<KnowledgeWikiPageRevision, KnowledgeWikiPageStoreError>;

    async fn next_revision_no(&self, page_id: u64) -> Result<u64, KnowledgeWikiPageStoreError>;

    async fn mark_current_revision(
        &self,
        record: MarkKnowledgeWikiCurrentRevisionRecord,
    ) -> Result<KnowledgeWikiPage, KnowledgeWikiPageStoreError>;

    async fn list_page_summaries(
        &self,
        space_id: u64,
    ) -> Result<Vec<WikiPageSummary>, KnowledgeWikiPageStoreError>;

    async fn append_log_entry(
        &self,
        record: AppendKnowledgeWikiLogEntryRecord,
    ) -> Result<WikiLogEntry, KnowledgeWikiPageStoreError>;

    async fn list_log_entries(
        &self,
        space_id: u64,
    ) -> Result<Vec<WikiLogEntry>, KnowledgeWikiPageStoreError>;

    async fn batch_page_projections_by_paths(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeWikiPageProjection>, KnowledgeWikiPageStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertKnowledgeWikiPageRecord {
    pub space_id: u64,
    pub slug: String,
    pub title: String,
    pub page_type: WikiPageType,
    pub logical_path: String,
    pub summary: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub publish_state: WikiPagePublishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeWikiPageRevisionRecord {
    pub page_id: u64,
    pub revision_no: u64,
    pub markdown_object_ref_id: u64,
    pub content_hash: String,
    pub review_state: WikiRevisionReviewState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkKnowledgeWikiCurrentRevisionRecord {
    pub page_id: u64,
    pub revision_id: u64,
    pub publish_state: WikiPagePublishState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendKnowledgeWikiLogEntryRecord {
    pub space_id: u64,
    pub event_type: String,
    pub event_time: String,
    pub title: String,
    pub actor: String,
    pub affected_pages: Vec<String>,
    pub audit_event_id: Option<String>,
    pub warnings: Vec<String>,
    pub privacy_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWikiPageProjection {
    pub logical_path: String,
    pub page_id: u64,
    pub current_revision_id: Option<u64>,
    pub publish_state: WikiPagePublishState,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeWikiPageStoreError {
    #[error("wiki page store internal error: {0}")]
    Internal(String),
}
