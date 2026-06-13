use async_trait::async_trait;
use sdkwork_knowledgebase_contract::space::KnowledgeSpace;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeSpaceStore: Send + Sync {
    async fn create_space(
        &self,
        record: CreateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError>;

    async fn get_space(&self, space_id: u64) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError>;

    async fn mark_drive_space_bound(
        &self,
        space_id: u64,
        drive_space_id: String,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError>;

    async fn mark_llm_wiki_initialized(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError>;

    async fn mark_space_deleted(&self, space_id: u64) -> Result<(), KnowledgeSpaceStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeSpaceRecord {
    pub name: String,
    pub description: Option<String>,
    pub llm_wiki_initialized: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSpaceStoreError {
    #[error("knowledge space store conflict: {0}")]
    Conflict(String),
    #[error("knowledge space store internal error: {0}")]
    Internal(String),
}
