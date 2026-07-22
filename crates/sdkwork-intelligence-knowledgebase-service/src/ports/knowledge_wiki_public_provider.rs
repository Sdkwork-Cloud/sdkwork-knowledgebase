use async_trait::async_trait;

use super::knowledge_wiki_persistence::{
    WikiPersistenceError, WikiPersistenceScope, WikiSourceFileKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPublication {
    pub id: u64,
    pub uuid: String,
    pub scope: WikiPersistenceScope,
    pub source_scope_uuid: String,
    pub title: String,
    pub description: Option<String>,
    pub homepage_source_path: String,
    pub default_locale: String,
    pub supported_locales: Vec<String>,
    pub navigation_mode: String,
    pub theme_key: String,
    pub theme_version: String,
    pub renderer_policy_version: String,
    pub search_enabled: bool,
    pub robots_policy: String,
    pub sitemap_enabled: bool,
    pub provider_generation: u64,
    pub navigation_generation: u64,
    pub search_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPageProjection {
    pub id: u64,
    pub uuid: String,
    pub source_path: String,
    pub canonical_route: String,
    pub file_kind: WikiSourceFileKind,
    pub media_type: String,
    pub size_bytes: u64,
    pub content_sha256: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub locale: Option<String>,
    pub nav_order: Option<i32>,
    pub public_drive_version_uuid: String,
    pub page_public_version: u64,
    pub public_updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicRouteMatch {
    pub page: WikiPublicPageProjection,
    pub matched_previous_route: bool,
    pub redirect_status: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPageKeyset {
    pub canonical_route: String,
    pub page_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWikiPublicNavigationRequest {
    pub scope: WikiPersistenceScope,
    pub publication_id: u64,
    pub locale: Option<String>,
    pub after: Option<WikiPublicPageKeyset>,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchWikiPublicPagesRequest {
    pub scope: WikiPersistenceScope,
    pub publication_id: u64,
    pub query: String,
    pub locale: Option<String>,
    pub after: Option<WikiPublicPageKeyset>,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPageWindow {
    pub items: Vec<WikiPublicPageProjection>,
    pub next: Option<WikiPublicPageKeyset>,
}

#[async_trait]
pub trait WikiPublicProviderStore: Send + Sync {
    async fn get_active_publication_by_uuid(
        &self,
        scope: WikiPersistenceScope,
        publication_uuid: &str,
    ) -> Result<Option<WikiPublicPublication>, WikiPersistenceError>;

    async fn resolve_public_route(
        &self,
        scope: WikiPersistenceScope,
        publication_id: u64,
        canonical_route: &str,
    ) -> Result<Option<WikiPublicRouteMatch>, WikiPersistenceError>;

    async fn get_public_content_projection(
        &self,
        scope: WikiPersistenceScope,
        publication_id: u64,
        projection_uuid: &str,
        page_public_version: u64,
    ) -> Result<Option<WikiPublicPageProjection>, WikiPersistenceError>;

    async fn list_public_navigation(
        &self,
        request: ListWikiPublicNavigationRequest,
    ) -> Result<WikiPublicPageWindow, WikiPersistenceError>;

    async fn search_public_pages(
        &self,
        request: SearchWikiPublicPagesRequest,
    ) -> Result<WikiPublicPageWindow, WikiPersistenceError>;
}
