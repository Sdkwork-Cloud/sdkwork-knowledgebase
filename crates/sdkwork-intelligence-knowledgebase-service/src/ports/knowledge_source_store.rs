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

    async fn newest_lineage_activity_at(
        &self,
        space_id: u64,
    ) -> Result<Option<String>, KnowledgeSourceStoreError> {
        let _ = space_id;
        Ok(None)
    }

    async fn list_space_source_lineage(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSourceLineageSnapshot>, KnowledgeSourceStoreError> {
        let _ = space_id;
        Ok(Vec::new())
    }

    async fn list_sources_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeSourceStoreError> {
        let _ = space_id;
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeSourceRecord {
    pub space_id: u64,
    pub source_type: KnowledgeSourceType,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
    pub connector_metadata_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSourceLineageSnapshot {
    pub source_id: u64,
    pub updated_at: String,
    pub last_sync_at: Option<String>,
    pub provider: Option<String>,
    pub drive_bucket: Option<String>,
    pub drive_prefix: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSourceStoreError {
    #[error("knowledge source store internal error: {0}")]
    Internal(String),
}
