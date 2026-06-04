use async_trait::async_trait;
use sdkwork_knowledgebase_contract::wiki::WikiPagePublishState;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeBrowserProjectionStore: Send + Sync {
    async fn batch_document_projections(
        &self,
        space_id: u64,
        drive_node_ids: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserDocumentProjection>, KnowledgeBrowserProjectionStoreError>;

    async fn batch_wiki_page_projections(
        &self,
        space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserWikiPageProjection>, KnowledgeBrowserProjectionStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBrowserDocumentProjection {
    pub drive_node_id: String,
    pub document_id: u64,
    pub current_version_id: Option<u64>,
    pub ingest_state: String,
    pub parse_state: String,
    pub index_state: String,
    pub wiki_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBrowserWikiPageProjection {
    pub logical_path: String,
    pub page_id: u64,
    pub current_revision_id: Option<u64>,
    pub publish_state: WikiPagePublishState,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeBrowserProjectionStoreError {
    #[error("knowledge browser projection invalid request: {0}")]
    InvalidRequest(String),
    #[error("knowledge browser projection internal error: {0}")]
    Internal(String),
}
