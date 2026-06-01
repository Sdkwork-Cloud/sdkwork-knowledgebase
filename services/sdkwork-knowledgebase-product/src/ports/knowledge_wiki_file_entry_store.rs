use async_trait::async_trait;
use sdkwork_knowledgebase_contract::wiki_file::{KnowledgeWikiFileEntry, WikiFileEntryType};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeWikiFileEntryStore: Send + Sync {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<KnowledgeWikiFileEntry, KnowledgeWikiFileEntryStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKnowledgeWikiFileEntryRecord {
    pub space_id: u64,
    pub logical_path: String,
    pub entry_type: WikiFileEntryType,
    pub artifact_role: String,
    pub drive_bucket: String,
    pub drive_object_key: String,
    pub checksum_sha256_hex: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeWikiFileEntryStoreError {
    #[error("wiki file entry store internal error: {0}")]
    Internal(String),
}
