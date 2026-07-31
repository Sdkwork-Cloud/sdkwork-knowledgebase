use async_trait::async_trait;
use sdkwork_knowledgebase_contract::market::KnowledgeMarketCatalogItem;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeMarketStoreError {
    #[error("invalid market request: {0}")]
    InvalidRequest(String),
    #[error("market listing not found")]
    NotFound,
    #[error("market store internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait KnowledgeMarketStore: Send + Sync {
    async fn list_catalog_page(
        &self,
        tenant_id: u64,
        subscriber_actor_id: Option<u64>,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(Vec<KnowledgeMarketCatalogItem>, Option<String>, bool), KnowledgeMarketStoreError>;

    async fn subscribe(
        &self,
        tenant_id: u64,
        subscriber_actor_id: u64,
        listing_id: u64,
    ) -> Result<(), KnowledgeMarketStoreError>;

    async fn unsubscribe(
        &self,
        tenant_id: u64,
        subscriber_actor_id: u64,
        listing_id: u64,
    ) -> Result<(), KnowledgeMarketStoreError>;
}

#[allow(clippy::too_many_arguments)]
pub fn map_catalog_item(
    listing_id: u64,
    title: String,
    icon: String,
    description: String,
    author: String,
    tags: Vec<String>,
    provider: String,
    model_name: String,
    subscribers_count: u32,
    documents_count: u32,
    is_subscribed: bool,
) -> KnowledgeMarketCatalogItem {
    KnowledgeMarketCatalogItem {
        id: listing_id.to_string(),
        title,
        icon,
        description,
        author,
        tags,
        subscribers_count,
        documents_count,
        provider,
        model_name,
        is_subscribed,
    }
}
