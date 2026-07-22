use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ports::{
    knowledge_wiki_drive_source::{
        KnowledgeWikiDriveSource, KnowledgeWikiDriveSourceError, ReadKnowledgeWikiSourceRequest,
        ResolveKnowledgeWikiSourceRequest, MAX_WIKI_SOURCE_READ_BYTES,
    },
    knowledge_wiki_persistence::{WikiPersistenceError, WikiPersistenceScope, WikiSourceFileKind},
    knowledge_wiki_public_provider::{
        ListWikiPublicNavigationRequest, SearchWikiPublicPagesRequest, WikiPublicPageKeyset,
        WikiPublicPageProjection, WikiPublicProviderStore, WikiPublicPublication,
    },
};
use sdkwork_utils_rust::{
    base64url_decode, base64url_encode, sha256_hash, DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE,
};

const CONTENT_HANDLE_KIND: &str = "wiki-content";
const NAVIGATION_CURSOR_KIND: &str = "wiki-navigation";
const SEARCH_CURSOR_KIND: &str = "wiki-search";
const PROVIDER_TOKEN_VERSION: u8 = 1;
const MAX_PROVIDER_TOKEN_LENGTH: usize = 4_096;
const MAX_PUBLICATION_UUID_BYTES: usize = 64;
const MAX_ROUTE_BYTES: usize = 2_048;
const MAX_QUERY_BYTES: usize = 256;
const MAX_LOCALE_BYTES: usize = 35;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveWikiPublicPublicationRequest {
    pub scope: WikiPersistenceScope,
    pub publication_uuid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveWikiPublicRouteRequest {
    pub scope: WikiPersistenceScope,
    pub publication_uuid: String,
    pub route: String,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveWikiPublicContentRequest {
    pub scope: WikiPersistenceScope,
    pub publication_uuid: String,
    pub content_handle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListWikiPublicNavigationPageRequest {
    pub scope: WikiPersistenceScope,
    pub publication_uuid: String,
    pub locale: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchWikiPublicPageRequest {
    pub scope: WikiPersistenceScope,
    pub publication_uuid: String,
    pub query: String,
    pub locale: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPublicationMetadata {
    pub publication_uuid: String,
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
pub struct WikiPublicPageMetadata {
    pub projection_uuid: String,
    pub canonical_route: String,
    pub file_kind: WikiSourceFileKind,
    pub media_type: String,
    pub size_bytes: u64,
    pub content_sha256: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub locale: Option<String>,
    pub nav_order: Option<i32>,
    pub page_public_version: u64,
    pub public_updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiResolvedPublicPage {
    pub page: WikiPublicPageMetadata,
    pub content_handle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiPublicRouteResolution {
    Page(WikiResolvedPublicPage),
    Redirect {
        requested_route: String,
        canonical_route: String,
        status: u16,
        page_public_version: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicContent {
    pub bytes: Vec<u8>,
    pub media_type: String,
    pub content_sha256: String,
    pub page_public_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPageListItem {
    pub page: WikiPublicPageMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicPageList {
    pub items: Vec<WikiPublicPageListItem>,
    pub next_cursor: Option<String>,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentHandle {
    version: u8,
    kind: String,
    tenant_id: u64,
    organization_id: u64,
    publication_uuid: String,
    projection_uuid: String,
    page_public_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PageCursor {
    version: u8,
    kind: String,
    tenant_id: u64,
    organization_id: u64,
    publication_uuid: String,
    query_sha256: Option<String>,
    locale: Option<String>,
    canonical_route: String,
    page_id: u64,
}

pub struct KnowledgeWikiPublicProviderService {
    store: Arc<dyn WikiPublicProviderStore>,
    drive_source: Arc<dyn KnowledgeWikiDriveSource>,
}

impl KnowledgeWikiPublicProviderService {
    pub fn new(
        store: Arc<dyn WikiPublicProviderStore>,
        drive_source: Arc<dyn KnowledgeWikiDriveSource>,
    ) -> Self {
        Self {
            store,
            drive_source,
        }
    }

    pub async fn retrieve_publication(
        &self,
        request: RetrieveWikiPublicPublicationRequest,
    ) -> Result<WikiPublicPublicationMetadata, KnowledgeWikiPublicProviderError> {
        let publication = self
            .active_publication(request.scope, &request.publication_uuid)
            .await?;
        Ok(publication_metadata(publication))
    }

    pub async fn resolve_route(
        &self,
        request: ResolveWikiPublicRouteRequest,
    ) -> Result<WikiPublicRouteResolution, KnowledgeWikiPublicProviderError> {
        validate_route(&request.route)?;
        validate_locale(request.locale.as_deref())?;
        let publication = self
            .active_publication(request.scope, &request.publication_uuid)
            .await?;
        let route_match = self
            .store
            .resolve_public_route(request.scope, publication.id, &request.route)
            .await?
            .ok_or(KnowledgeWikiPublicProviderError::NotFoundOrNotPublic)?;
        if route_match.matched_previous_route {
            return Ok(WikiPublicRouteResolution::Redirect {
                requested_route: request.route,
                canonical_route: route_match.page.canonical_route,
                status: route_match.redirect_status.unwrap_or(308),
                page_public_version: route_match.page.page_public_version,
            });
        }
        let content_handle = encode_content_handle(
            request.scope,
            &publication.uuid,
            &route_match.page.uuid,
            route_match.page.page_public_version,
        )?;
        Ok(WikiPublicRouteResolution::Page(WikiResolvedPublicPage {
            page: page_metadata(route_match.page),
            content_handle,
        }))
    }

    pub async fn retrieve_content(
        &self,
        request: RetrieveWikiPublicContentRequest,
    ) -> Result<WikiPublicContent, KnowledgeWikiPublicProviderError> {
        let handle = decode_content_handle(
            &request.content_handle,
            request.scope,
            &request.publication_uuid,
        )?;
        let publication = self
            .active_publication(request.scope, &request.publication_uuid)
            .await?;
        let page = self
            .store
            .get_public_content_projection(
                request.scope,
                publication.id,
                &handle.projection_uuid,
                handle.page_public_version,
            )
            .await?
            .ok_or(KnowledgeWikiPublicProviderError::NotFoundOrNotPublic)?;
        if page.size_bytes > MAX_WIKI_SOURCE_READ_BYTES {
            return Err(KnowledgeWikiPublicProviderError::ContentUnavailable);
        }
        let resource = self
            .drive_source
            .resolve_source(ResolveKnowledgeWikiSourceRequest {
                subscription_uuid: publication.source_scope_uuid,
                relative_path: page.source_path.clone(),
                pinned_generation: None,
                pinned_node_version_id: Some(page.public_drive_version_uuid.clone()),
            })
            .await?;
        if resource.normalized_relative_path != page.source_path
            || resource.drive_node_version_id != page.public_drive_version_uuid
            || resource.content_length != page.size_bytes
            || resource.checksum_sha256_hex != page.content_sha256
        {
            return Err(KnowledgeWikiPublicProviderError::IntegrityFailed);
        }
        let bytes = self
            .drive_source
            .read_pinned_source(ReadKnowledgeWikiSourceRequest {
                resource,
                maximum_bytes: MAX_WIKI_SOURCE_READ_BYTES,
            })
            .await?;
        if bytes.len() as u64 != page.size_bytes
            || format!("sha256:{}", sha256_hash(&bytes)) != page.content_sha256
        {
            return Err(KnowledgeWikiPublicProviderError::IntegrityFailed);
        }
        Ok(WikiPublicContent {
            bytes,
            media_type: page.media_type,
            content_sha256: page.content_sha256,
            page_public_version: page.page_public_version,
        })
    }

    pub async fn list_navigation(
        &self,
        request: ListWikiPublicNavigationPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        validate_locale(request.locale.as_deref())?;
        let page_size = normalize_page_size(request.page_size)?;
        let publication = self
            .active_publication(request.scope, &request.publication_uuid)
            .await?;
        let after = decode_page_cursor(
            request.cursor.as_deref(),
            NAVIGATION_CURSOR_KIND,
            request.scope,
            &publication.uuid,
            None,
            request.locale.as_deref(),
        )?;
        let window = self
            .store
            .list_public_navigation(ListWikiPublicNavigationRequest {
                scope: request.scope,
                publication_id: publication.id,
                locale: request.locale.clone(),
                after,
                limit: page_size,
            })
            .await?;
        let next_cursor = window
            .next
            .map(|next| {
                encode_page_cursor(
                    NAVIGATION_CURSOR_KIND,
                    request.scope,
                    &publication.uuid,
                    None,
                    request.locale.as_deref(),
                    next,
                )
            })
            .transpose()?;
        Ok(WikiPublicPageList {
            items: window
                .items
                .into_iter()
                .map(|page| WikiPublicPageListItem {
                    page: page_metadata(page),
                })
                .collect(),
            next_cursor,
            page_size,
        })
    }

    pub async fn search_pages(
        &self,
        request: SearchWikiPublicPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        let query = normalize_query(&request.query)?;
        validate_locale(request.locale.as_deref())?;
        let page_size = normalize_page_size(request.page_size)?;
        let publication = self
            .active_publication(request.scope, &request.publication_uuid)
            .await?;
        if !publication.search_enabled {
            return Err(KnowledgeWikiPublicProviderError::NotFoundOrNotPublic);
        }
        let query_sha256 = sha256_hash(query.as_bytes());
        let after = decode_page_cursor(
            request.cursor.as_deref(),
            SEARCH_CURSOR_KIND,
            request.scope,
            &publication.uuid,
            Some(&query_sha256),
            request.locale.as_deref(),
        )?;
        let window = self
            .store
            .search_public_pages(SearchWikiPublicPagesRequest {
                scope: request.scope,
                publication_id: publication.id,
                query,
                locale: request.locale.clone(),
                after,
                limit: page_size,
            })
            .await?;
        let next_cursor = window
            .next
            .map(|next| {
                encode_page_cursor(
                    SEARCH_CURSOR_KIND,
                    request.scope,
                    &publication.uuid,
                    Some(&query_sha256),
                    request.locale.as_deref(),
                    next,
                )
            })
            .transpose()?;
        Ok(WikiPublicPageList {
            items: window
                .items
                .into_iter()
                .map(|page| WikiPublicPageListItem {
                    page: page_metadata(page),
                })
                .collect(),
            next_cursor,
            page_size,
        })
    }

    async fn active_publication(
        &self,
        scope: WikiPersistenceScope,
        publication_uuid: &str,
    ) -> Result<WikiPublicPublication, KnowledgeWikiPublicProviderError> {
        validate_scope(scope)?;
        validate_publication_uuid(publication_uuid)?;
        self.store
            .get_active_publication_by_uuid(scope, publication_uuid)
            .await?
            .ok_or(KnowledgeWikiPublicProviderError::NotFoundOrNotPublic)
    }
}

fn publication_metadata(publication: WikiPublicPublication) -> WikiPublicPublicationMetadata {
    WikiPublicPublicationMetadata {
        publication_uuid: publication.uuid,
        title: publication.title,
        description: publication.description,
        homepage_source_path: publication.homepage_source_path,
        default_locale: publication.default_locale,
        supported_locales: publication.supported_locales,
        navigation_mode: publication.navigation_mode,
        theme_key: publication.theme_key,
        theme_version: publication.theme_version,
        renderer_policy_version: publication.renderer_policy_version,
        search_enabled: publication.search_enabled,
        robots_policy: publication.robots_policy,
        sitemap_enabled: publication.sitemap_enabled,
        provider_generation: publication.provider_generation,
        navigation_generation: publication.navigation_generation,
        search_generation: publication.search_generation,
    }
}

fn page_metadata(page: WikiPublicPageProjection) -> WikiPublicPageMetadata {
    WikiPublicPageMetadata {
        projection_uuid: page.uuid,
        canonical_route: page.canonical_route,
        file_kind: page.file_kind,
        media_type: page.media_type,
        size_bytes: page.size_bytes,
        content_sha256: page.content_sha256,
        title: page.title,
        description: page.description,
        locale: page.locale,
        nav_order: page.nav_order,
        page_public_version: page.page_public_version,
        public_updated_at: page.public_updated_at,
    }
}

fn validate_scope(scope: WikiPersistenceScope) -> Result<(), KnowledgeWikiPublicProviderError> {
    if scope.tenant_id == 0 {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "tenant_id must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

fn validate_publication_uuid(value: &str) -> Result<(), KnowledgeWikiPublicProviderError> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_PUBLICATION_UUID_BYTES {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "publication_uuid is outside its length limit".to_string(),
        ));
    }
    Ok(())
}

fn validate_route(route: &str) -> Result<(), KnowledgeWikiPublicProviderError> {
    if route.is_empty()
        || route.len() > MAX_ROUTE_BYTES
        || !route.starts_with('/')
        || route.contains('\\')
        || route.contains('%')
        || route.contains('?')
        || route.contains('#')
        || route.contains("//")
        || route.chars().any(char::is_control)
        || route
            .split('/')
            .any(|segment| segment == "." || segment == "..")
    {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "route must be a normalized absolute provider route".to_string(),
        ));
    }
    Ok(())
}

fn validate_locale(locale: Option<&str>) -> Result<(), KnowledgeWikiPublicProviderError> {
    if locale.is_some_and(|value| {
        value.is_empty()
            || value.len() > MAX_LOCALE_BYTES
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    }) {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "locale is invalid".to_string(),
        ));
    }
    Ok(())
}

fn normalize_query(query: &str) -> Result<String, KnowledgeWikiPublicProviderError> {
    let query = query.trim();
    if query.is_empty() || query.len() > MAX_QUERY_BYTES || query.chars().any(char::is_control) {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "q must contain between 1 and 256 bytes".to_string(),
        ));
    }
    Ok(query.to_string())
}

fn normalize_page_size(page_size: Option<u32>) -> Result<u32, KnowledgeWikiPublicProviderError> {
    let page_size = page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE as u32);
    if !(1..=MAX_LIST_PAGE_SIZE as u32).contains(&page_size) {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(format!(
            "page_size must be between 1 and {MAX_LIST_PAGE_SIZE}"
        )));
    }
    Ok(page_size)
}

fn encode_content_handle(
    scope: WikiPersistenceScope,
    publication_uuid: &str,
    projection_uuid: &str,
    page_public_version: u64,
) -> Result<String, KnowledgeWikiPublicProviderError> {
    encode_token(&ContentHandle {
        version: PROVIDER_TOKEN_VERSION,
        kind: CONTENT_HANDLE_KIND.to_string(),
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id,
        publication_uuid: publication_uuid.to_string(),
        projection_uuid: projection_uuid.to_string(),
        page_public_version,
    })
}

fn decode_content_handle(
    handle: &str,
    scope: WikiPersistenceScope,
    publication_uuid: &str,
) -> Result<ContentHandle, KnowledgeWikiPublicProviderError> {
    let payload: ContentHandle = decode_token(handle)?;
    if payload.version != PROVIDER_TOKEN_VERSION
        || payload.kind != CONTENT_HANDLE_KIND
        || payload.tenant_id != scope.tenant_id
        || payload.organization_id != scope.organization_id
        || payload.publication_uuid != publication_uuid
        || payload.projection_uuid.is_empty()
        || payload.page_public_version == 0
    {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "content handle is invalid for this publication scope".to_string(),
        ));
    }
    Ok(payload)
}

fn encode_page_cursor(
    kind: &str,
    scope: WikiPersistenceScope,
    publication_uuid: &str,
    query_sha256: Option<&str>,
    locale: Option<&str>,
    after: WikiPublicPageKeyset,
) -> Result<String, KnowledgeWikiPublicProviderError> {
    encode_token(&PageCursor {
        version: PROVIDER_TOKEN_VERSION,
        kind: kind.to_string(),
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id,
        publication_uuid: publication_uuid.to_string(),
        query_sha256: query_sha256.map(str::to_string),
        locale: locale.map(str::to_string),
        canonical_route: after.canonical_route,
        page_id: after.page_id,
    })
}

fn decode_page_cursor(
    cursor: Option<&str>,
    kind: &str,
    scope: WikiPersistenceScope,
    publication_uuid: &str,
    query_sha256: Option<&str>,
    locale: Option<&str>,
) -> Result<Option<WikiPublicPageKeyset>, KnowledgeWikiPublicProviderError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let payload: PageCursor = decode_token(cursor)?;
    if payload.version != PROVIDER_TOKEN_VERSION
        || payload.kind != kind
        || payload.tenant_id != scope.tenant_id
        || payload.organization_id != scope.organization_id
        || payload.publication_uuid != publication_uuid
        || payload.query_sha256.as_deref() != query_sha256
        || payload.locale.as_deref() != locale
        || payload.canonical_route.is_empty()
        || payload.page_id == 0
    {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "cursor is invalid for this publication, query, and locale".to_string(),
        ));
    }
    Ok(Some(WikiPublicPageKeyset {
        canonical_route: payload.canonical_route,
        page_id: payload.page_id,
    }))
}

fn encode_token<T: Serialize>(value: &T) -> Result<String, KnowledgeWikiPublicProviderError> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        KnowledgeWikiPublicProviderError::InvalidRequest(
            "provider token could not be encoded".to_string(),
        )
    })?;
    let encoded = base64url_encode(&bytes);
    if encoded.len() > MAX_PROVIDER_TOKEN_LENGTH {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "provider token exceeds the maximum length".to_string(),
        ));
    }
    Ok(encoded)
}

fn decode_token<T: for<'de> Deserialize<'de>>(
    value: &str,
) -> Result<T, KnowledgeWikiPublicProviderError> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > MAX_PROVIDER_TOKEN_LENGTH
        || value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(KnowledgeWikiPublicProviderError::InvalidRequest(
            "provider token is malformed".to_string(),
        ));
    }
    let decoded = base64url_decode(value).ok_or_else(|| {
        KnowledgeWikiPublicProviderError::InvalidRequest("provider token is malformed".to_string())
    })?;
    serde_json::from_slice(&decoded).map_err(|_| {
        KnowledgeWikiPublicProviderError::InvalidRequest("provider token is malformed".to_string())
    })
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeWikiPublicProviderError {
    #[error("Wiki public provider invalid request: {0}")]
    InvalidRequest(String),
    #[error("Wiki public provider resource was not found or is not public")]
    NotFoundOrNotPublic,
    #[error("Wiki public provider content is not available through the bounded reader")]
    ContentUnavailable,
    #[error("Wiki public provider integrity validation failed")]
    IntegrityFailed,
    #[error("Wiki public provider is temporarily unavailable")]
    TemporarilyUnavailable,
}

impl KnowledgeWikiPublicProviderError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "wiki_public_provider_invalid_request",
            Self::NotFoundOrNotPublic => "wiki_not_found_or_not_public",
            Self::ContentUnavailable => "wiki_public_content_unavailable",
            Self::IntegrityFailed => "wiki_public_content_integrity_failed",
            Self::TemporarilyUnavailable => "wiki_public_provider_unavailable",
        }
    }
}

impl From<WikiPersistenceError> for KnowledgeWikiPublicProviderError {
    fn from(error: WikiPersistenceError) -> Self {
        match error {
            WikiPersistenceError::InvalidRequest(detail) => Self::InvalidRequest(detail),
            WikiPersistenceError::NotFound { .. }
            | WikiPersistenceError::Conflict(_)
            | WikiPersistenceError::StaleVersion { .. } => Self::NotFoundOrNotPublic,
            WikiPersistenceError::Internal(_) => Self::TemporarilyUnavailable,
        }
    }
}

impl From<KnowledgeWikiDriveSourceError> for KnowledgeWikiPublicProviderError {
    fn from(error: KnowledgeWikiDriveSourceError) -> Self {
        match error {
            KnowledgeWikiDriveSourceError::InvalidRequest(_) => Self::ContentUnavailable,
            KnowledgeWikiDriveSourceError::NotFound(_)
            | KnowledgeWikiDriveSourceError::Conflict(_) => Self::NotFoundOrNotPublic,
            KnowledgeWikiDriveSourceError::IntegrityFailed(_) => Self::IntegrityFailed,
            KnowledgeWikiDriveSourceError::Upstream(_) => Self::TemporarilyUnavailable,
        }
    }
}
