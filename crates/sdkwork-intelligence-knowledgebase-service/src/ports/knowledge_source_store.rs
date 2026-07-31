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
        Err(KnowledgeSourceStoreError::Unsupported(format!(
            "newest_lineage_activity_at is unsupported for space {space_id}"
        )))
    }

    async fn list_space_source_lineage(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSourceLineageSnapshot>, KnowledgeSourceStoreError> {
        Err(KnowledgeSourceStoreError::Unsupported(format!(
            "list_space_source_lineage is unsupported for space {space_id}"
        )))
    }

    async fn list_sources_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeSourceStoreError> {
        Err(KnowledgeSourceStoreError::Unsupported(format!(
            "list_sources_for_space is unsupported for space {space_id}"
        )))
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
    #[error("knowledge source store unsupported operation: {0}")]
    Unsupported(String),
    #[error("knowledge source store internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CreateOnlySourceStore;

    #[async_trait]
    impl KnowledgeSourceStore for CreateOnlySourceStore {
        async fn create_source(
            &self,
            _record: CreateKnowledgeSourceRecord,
        ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
            Err(KnowledgeSourceStoreError::Unsupported(
                "create_source is not used by this test".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn optional_reads_fail_closed_when_store_does_not_implement_them() {
        let store = CreateOnlySourceStore;
        for error in [
            store
                .newest_lineage_activity_at(42)
                .await
                .expect_err("lineage activity must not silently return none"),
            store
                .list_space_source_lineage(42)
                .await
                .expect_err("lineage list must not silently return empty"),
            store
                .list_sources_for_space(42)
                .await
                .expect_err("source list must not silently return empty"),
        ] {
            assert!(matches!(error, KnowledgeSourceStoreError::Unsupported(_)));
        }
    }
}
