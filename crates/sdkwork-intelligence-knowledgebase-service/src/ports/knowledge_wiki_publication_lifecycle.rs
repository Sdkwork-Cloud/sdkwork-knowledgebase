use async_trait::async_trait;

use super::knowledge_wiki_persistence::{
    WikiPersistenceError, WikiPersistenceScope, WikiPublication, WikiSourceProjection,
    WikiVisibility,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiPublicationLifecycleAction {
    Activate,
    Pause,
}

impl WikiPublicationLifecycleAction {
    pub const fn as_operation(self) -> &'static str {
        match self {
            Self::Activate => "ACTIVATE",
            Self::Pause => "PAUSE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiLifecycleDisposition {
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLifecycleAuditContext {
    pub request_id: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeWikiPublicationStatusRequest {
    pub scope: WikiPersistenceScope,
    pub space_id: u64,
    pub expected_version: u64,
    pub actor_id: u64,
    pub action: WikiPublicationLifecycleAction,
    pub audit: WikiLifecycleAuditContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicationLifecycleResult {
    pub publication: WikiPublication,
    pub disposition: WikiLifecycleDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishWikiPageRequest {
    pub scope: WikiPersistenceScope,
    pub space_id: u64,
    pub source_file_uuid: String,
    pub visibility: WikiVisibility,
    pub expected_publication_version: u64,
    pub expected_page_version: u64,
    pub actor_id: u64,
    pub audit: WikiLifecycleAuditContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpublishWikiPageRequest {
    pub scope: WikiPersistenceScope,
    pub space_id: u64,
    pub source_file_uuid: String,
    pub expected_publication_version: u64,
    pub expected_page_version: u64,
    pub actor_id: u64,
    pub audit: WikiLifecycleAuditContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeWikiPageVisibilityRequest {
    pub scope: WikiPersistenceScope,
    pub space_id: u64,
    pub source_file_uuid: String,
    pub visibility: WikiVisibility,
    pub expected_publication_version: u64,
    pub expected_page_version: u64,
    pub actor_id: u64,
    pub audit: WikiLifecycleAuditContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPageLifecycleResult {
    pub publication: WikiPublication,
    pub page: WikiSourceProjection,
    pub disposition: WikiLifecycleDisposition,
}

#[async_trait]
pub trait WikiPublicationLifecycleStore: Send + Sync {
    async fn change_publication_status(
        &self,
        request: ChangeWikiPublicationStatusRequest,
    ) -> Result<WikiPublicationLifecycleResult, WikiPersistenceError>;

    async fn publish_page(
        &self,
        request: PublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError>;

    async fn unpublish_page(
        &self,
        request: UnpublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError>;

    async fn change_page_visibility(
        &self,
        request: ChangeWikiPageVisibilityRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError>;
}
