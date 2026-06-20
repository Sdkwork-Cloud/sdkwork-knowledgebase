use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeOkfConceptLinkStore: Send + Sync {
    async fn replace_outbound_links(
        &self,
        record: ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), KnowledgeOkfConceptLinkStoreError>;

    async fn list_inbound_concept_ids(
        &self,
        space_id: u64,
        to_concept_id: &str,
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError>;

    async fn list_orphan_concept_ids(
        &self,
        space_id: u64,
        published_concept_ids: &[String],
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceKnowledgeOkfConceptLinksRecord {
    pub space_id: u64,
    pub from_concept_id: String,
    pub links: Vec<KnowledgeOkfConceptLinkRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOkfConceptLinkRecord {
    pub to_concept_id: String,
    pub anchor_text: String,
}

#[derive(Debug, Error)]
pub enum KnowledgeOkfConceptLinkStoreError {
    #[error("internal knowledge okf concept link store error: {0}")]
    Internal(String),
}
