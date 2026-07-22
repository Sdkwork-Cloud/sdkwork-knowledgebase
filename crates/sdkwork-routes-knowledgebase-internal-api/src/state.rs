use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::WikiDriveEventReceipt;
use sdkwork_intelligence_knowledgebase_service::wiki_event_consumer::{
    KnowledgeWikiDriveEventConsumerError, ReceiveKnowledgeWikiDriveWebhookRequest,
};
use sdkwork_intelligence_knowledgebase_service::wiki_public_provider::{
    KnowledgeWikiPublicProviderError, ListWikiPublicNavigationPageRequest,
    ResolveWikiPublicRouteRequest, RetrieveWikiPublicContentRequest,
    RetrieveWikiPublicPublicationRequest, SearchWikiPublicPageRequest, WikiPublicContent,
    WikiPublicPageList, WikiPublicPublicationMetadata, WikiPublicRouteResolution,
};
use std::sync::Arc;

#[async_trait]
pub trait KnowledgebaseDriveEventReceiver: Send + Sync {
    async fn receive_drive_webhook(
        &self,
        request: ReceiveKnowledgeWikiDriveWebhookRequest,
    ) -> Result<WikiDriveEventReceipt, KnowledgeWikiDriveEventConsumerError>;
}

#[async_trait]
pub trait KnowledgebaseWikiPublicProvider: Send + Sync {
    async fn retrieve_publication(
        &self,
        request: RetrieveWikiPublicPublicationRequest,
    ) -> Result<WikiPublicPublicationMetadata, KnowledgeWikiPublicProviderError>;

    async fn resolve_route(
        &self,
        request: ResolveWikiPublicRouteRequest,
    ) -> Result<WikiPublicRouteResolution, KnowledgeWikiPublicProviderError>;

    async fn retrieve_content(
        &self,
        request: RetrieveWikiPublicContentRequest,
    ) -> Result<WikiPublicContent, KnowledgeWikiPublicProviderError>;

    async fn list_navigation(
        &self,
        request: ListWikiPublicNavigationPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError>;

    async fn search_pages(
        &self,
        request: SearchWikiPublicPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError>;
}

#[async_trait]
impl KnowledgebaseWikiPublicProvider
    for sdkwork_intelligence_knowledgebase_service::wiki_public_provider::KnowledgeWikiPublicProviderService
{
    async fn retrieve_publication(
        &self,
        request: RetrieveWikiPublicPublicationRequest,
    ) -> Result<WikiPublicPublicationMetadata, KnowledgeWikiPublicProviderError> {
        Self::retrieve_publication(self, request).await
    }

    async fn resolve_route(
        &self,
        request: ResolveWikiPublicRouteRequest,
    ) -> Result<WikiPublicRouteResolution, KnowledgeWikiPublicProviderError> {
        Self::resolve_route(self, request).await
    }

    async fn retrieve_content(
        &self,
        request: RetrieveWikiPublicContentRequest,
    ) -> Result<WikiPublicContent, KnowledgeWikiPublicProviderError> {
        Self::retrieve_content(self, request).await
    }

    async fn list_navigation(
        &self,
        request: ListWikiPublicNavigationPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        Self::list_navigation(self, request).await
    }

    async fn search_pages(
        &self,
        request: SearchWikiPublicPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        Self::search_pages(self, request).await
    }
}

#[derive(Clone)]
pub struct InternalApiState {
    pub receiver: Arc<dyn KnowledgebaseDriveEventReceiver>,
    pub wiki_provider: Arc<dyn KnowledgebaseWikiPublicProvider>,
    pub drive_event_caller_app_id: Arc<str>,
    pub wiki_provider_caller_app_id: Arc<str>,
    pub max_body_bytes: usize,
}

impl InternalApiState {
    pub fn new(
        receiver: Arc<dyn KnowledgebaseDriveEventReceiver>,
        wiki_provider: Arc<dyn KnowledgebaseWikiPublicProvider>,
        drive_event_caller_app_id: impl Into<String>,
        wiki_provider_caller_app_id: impl Into<String>,
    ) -> Self {
        Self {
            receiver,
            wiki_provider,
            drive_event_caller_app_id: Arc::from(drive_event_caller_app_id.into()),
            wiki_provider_caller_app_id: Arc::from(wiki_provider_caller_app_id.into()),
            max_body_bytes: 65_536,
        }
    }
}
