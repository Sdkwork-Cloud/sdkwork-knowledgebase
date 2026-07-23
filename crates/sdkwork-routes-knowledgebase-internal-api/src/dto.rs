use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_persistence::{
    WikiDriveEventReceipt, WikiDriveEventReceiveDisposition,
};
use sdkwork_intelligence_knowledgebase_service::wiki_public_provider::{
    WikiPublicPageListItem, WikiPublicPageMetadata, WikiPublicPublicationMetadata,
    WikiPublicRouteResolution,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveEventReceiptResponse {
    pub event_id: String,
    pub checkpoint_id: String,
    pub sequence_no: String,
    pub disposition: &'static str,
}

impl From<WikiDriveEventReceipt> for DriveEventReceiptResponse {
    fn from(receipt: WikiDriveEventReceipt) -> Self {
        Self {
            event_id: receipt.event.source_event_id,
            checkpoint_id: receipt.event.checkpoint_id.to_string(),
            sequence_no: receipt.event.sequence_no.to_string(),
            disposition: match receipt.disposition {
                WikiDriveEventReceiveDisposition::Ready => "READY",
                WikiDriveEventReceiveDisposition::DeferredGap => "DEFERRED_GAP",
                WikiDriveEventReceiveDisposition::Duplicate => "DUPLICATE",
                WikiDriveEventReceiveDisposition::IgnoredStale => "IGNORED_STALE",
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveWikiRouteBody {
    pub route: String,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WikiNavigationQuery {
    pub locale: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WikiSearchQuery {
    pub q: String,
    pub locale: Option<String>,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiPublicationResponse {
    pub publication_uuid: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub provider_generation: String,
    pub navigation_generation: String,
    pub search_generation: String,
}

impl From<WikiPublicPublicationMetadata> for WikiPublicationResponse {
    fn from(value: WikiPublicPublicationMetadata) -> Self {
        Self {
            publication_uuid: value.publication_uuid,
            title: value.title,
            description: value.description,
            homepage_source_path: value.homepage_source_path,
            default_locale: value.default_locale,
            supported_locales: value.supported_locales,
            navigation_mode: value.navigation_mode,
            theme_key: value.theme_key,
            theme_version: value.theme_version,
            renderer_policy_version: value.renderer_policy_version,
            search_enabled: value.search_enabled,
            robots_policy: value.robots_policy,
            sitemap_enabled: value.sitemap_enabled,
            provider_generation: value.provider_generation.to_string(),
            navigation_generation: value.navigation_generation.to_string(),
            search_generation: value.search_generation.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiPublicPageResponse {
    pub projection_uuid: String,
    pub canonical_route: String,
    pub file_kind: &'static str,
    pub media_type: String,
    pub size_bytes: String,
    pub content_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nav_order: Option<i32>,
    pub page_public_version: String,
    pub public_updated_at: String,
}

impl From<WikiPublicPageMetadata> for WikiPublicPageResponse {
    fn from(value: WikiPublicPageMetadata) -> Self {
        Self {
            projection_uuid: value.projection_uuid,
            canonical_route: value.canonical_route,
            file_kind: value.file_kind.as_str(),
            media_type: value.media_type,
            size_bytes: value.size_bytes.to_string(),
            content_sha256: value.content_sha256,
            title: value.title,
            description: value.description,
            locale: value.locale,
            nav_order: value.nav_order,
            page_public_version: value.page_public_version.to_string(),
            public_updated_at: value.public_updated_at,
        }
    }
}

impl From<WikiPublicPageListItem> for WikiPublicPageResponse {
    fn from(value: WikiPublicPageListItem) -> Self {
        value.page.into()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "disposition", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WikiRouteResolutionResponse {
    Page {
        page: Box<WikiPublicPageResponse>,
        #[serde(rename = "contentHandle")]
        content_handle: String,
    },
    Redirect {
        #[serde(rename = "requestedRoute")]
        requested_route: String,
        #[serde(rename = "canonicalRoute")]
        canonical_route: String,
        status: u16,
        #[serde(rename = "pagePublicVersion")]
        page_public_version: String,
    },
}

impl From<WikiPublicRouteResolution> for WikiRouteResolutionResponse {
    fn from(value: WikiPublicRouteResolution) -> Self {
        match value {
            WikiPublicRouteResolution::Page(value) => Self::Page {
                page: Box::new(value.page.into()),
                content_handle: value.content_handle,
            },
            WikiPublicRouteResolution::Redirect {
                requested_route,
                canonical_route,
                status,
                page_public_version,
            } => Self::Redirect {
                requested_route,
                canonical_route,
                status,
                page_public_version: page_public_version.to_string(),
            },
        }
    }
}
