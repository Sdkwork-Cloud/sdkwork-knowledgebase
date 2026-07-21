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
    icon: Option<String>,
    description: Option<String>,
    author: Option<String>,
    tags_json: String,
    provider: Option<String>,
    model_name: Option<String>,
    subscribers_count: u32,
    documents_count: u32,
    is_subscribed: bool,
) -> KnowledgeMarketCatalogItem {
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();
    KnowledgeMarketCatalogItem {
        id: listing_id.to_string(),
        title,
        icon: icon.unwrap_or_else(|| "📘".to_string()),
        description: description.unwrap_or_default(),
        author: author.unwrap_or_else(|| "SDKWork".to_string()),
        tags,
        subscribers_count,
        documents_count,
        provider: provider.unwrap_or_else(|| "Google".to_string()),
        model_name: model_name.unwrap_or_else(|| "gemini-3.5-flash".to_string()),
        is_subscribed,
    }
}
