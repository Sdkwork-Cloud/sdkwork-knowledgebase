use async_trait::async_trait;
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeSourceStore: Send + Sync {
    async fn create_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError>;

    async fn create_or_get_source(
        &self,
        record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        self.create_source(record).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeSourceRecord {
    pub space_id: u64,
    pub source_type: KnowledgeSourceType,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSourceStoreError {
    #[error("knowledge source store internal error: {0}")]
    Internal(String),
}
