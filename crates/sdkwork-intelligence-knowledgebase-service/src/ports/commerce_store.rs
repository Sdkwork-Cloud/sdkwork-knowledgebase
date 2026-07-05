use async_trait::async_trait;
use sdkwork_knowledgebase_contract::market::KnowledgeMarketCatalogItem;
use sdkwork_knowledgebase_contract::site_deployment::{
    KnowledgeSiteDeploymentPreview, KnowledgeSiteDeploymentRequest, KnowledgeSiteDeploymentResult,
};
use sdkwork_utils_rust::is_blank;
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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeSiteDeploymentStoreError {
    #[error("invalid site deployment request: {0}")]
    InvalidRequest(String),
    #[error("site deployment not found")]
    NotFound,
    #[error("site deployment store internal error: {0}")]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSiteDeploymentRecord {
    pub tenant_id: u64,
    pub space_id: u64,
    pub platform: String,
    pub site_name: Option<String>,
    pub custom_domain: Option<String>,
    pub site_logo_data_url: Option<String>,
    pub deployed_url: String,
    pub preview_object_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteDeploymentRecord {
    pub id: u64,
    pub tenant_id: u64,
    pub space_id: u64,
    pub platform: String,
    pub site_name: Option<String>,
    pub custom_domain: Option<String>,
    pub deployed_url: String,
    pub preview_object_key: String,
}

#[async_trait]
pub trait KnowledgeSiteDeploymentStore: Send + Sync {
    async fn create_deployment(
        &self,
        record: CreateSiteDeploymentRecord,
    ) -> Result<SiteDeploymentRecord, KnowledgeSiteDeploymentStoreError>;

    async fn get_deployment(
        &self,
        tenant_id: u64,
        deployment_id: u64,
    ) -> Result<SiteDeploymentRecord, KnowledgeSiteDeploymentStoreError>;
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

pub fn deployment_result(record: &SiteDeploymentRecord) -> KnowledgeSiteDeploymentResult {
    KnowledgeSiteDeploymentResult {
        success: true,
        deployment_id: record.id,
        url: record.deployed_url.clone(),
    }
}

pub fn deployment_preview(html: String, deployment_id: u64) -> KnowledgeSiteDeploymentPreview {
    KnowledgeSiteDeploymentPreview {
        deployment_id,
        content_type: "text/html; charset=utf-8".to_string(),
        html,
    }
}

pub fn validate_site_deployment_request(
    request: &KnowledgeSiteDeploymentRequest,
) -> Result<(), KnowledgeSiteDeploymentStoreError> {
    if request.space_id == 0 {
        return Err(KnowledgeSiteDeploymentStoreError::InvalidRequest(
            "space_id is required".to_string(),
        ));
    }
    if is_blank(Some(request.platform.as_str())) {
        return Err(KnowledgeSiteDeploymentStoreError::InvalidRequest(
            "platform is required".to_string(),
        ));
    }
    Ok(())
}
