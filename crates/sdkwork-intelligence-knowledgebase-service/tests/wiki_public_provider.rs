use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_wiki_drive_source::{
            EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveScope, KnowledgeWikiDriveSource,
            KnowledgeWikiDriveSourceError, KnowledgeWikiSourceResource, KnowledgebaseRawScope,
            ReadKnowledgeWikiSourceRequest, ResolveKnowledgeWikiSourceRequest,
        },
        knowledge_wiki_persistence::{
            WikiPersistenceError, WikiPersistenceScope, WikiSourceFileKind,
        },
        knowledge_wiki_public_provider::{
            ListWikiPublicNavigationRequest, SearchWikiPublicPagesRequest, WikiPublicPageKeyset,
            WikiPublicPageProjection, WikiPublicPageWindow, WikiPublicProviderStore,
            WikiPublicPublication, WikiPublicRouteMatch,
        },
    },
    wiki_public_provider::{
        KnowledgeWikiPublicProviderError, KnowledgeWikiPublicProviderService,
        ListWikiPublicNavigationPageRequest, ResolveWikiPublicRouteRequest,
        RetrieveWikiPublicContentRequest, SearchWikiPublicPageRequest, WikiPublicRouteResolution,
    },
};
use sdkwork_utils_rust::sha256_hash;

const SCOPE: WikiPersistenceScope = WikiPersistenceScope {
    tenant_id: 101,
    organization_id: 202,
};
const PUBLICATION_UUID: &str = "11111111-1111-4111-8111-111111111501";
const PROJECTION_UUID: &str = "11111111-1111-4111-8111-111111111601";
const SOURCE_BYTES: &[u8] = b"# Wiki";

struct FakePublicStore {
    public: Mutex<bool>,
}

impl Default for FakePublicStore {
    fn default() -> Self {
        Self {
            public: Mutex::new(true),
        }
    }
}

impl FakePublicStore {
    fn revoke(&self) {
        *self.public.lock().expect("public state lock") = false;
    }

    fn is_public(&self, scope: WikiPersistenceScope, publication_id: u64) -> bool {
        *self.public.lock().expect("public state lock") && scope == SCOPE && publication_id == 501
    }
}

#[async_trait]
impl WikiPublicProviderStore for FakePublicStore {
    async fn get_active_publication_by_uuid(
        &self,
        scope: WikiPersistenceScope,
        publication_uuid: &str,
    ) -> Result<Option<WikiPublicPublication>, WikiPersistenceError> {
        Ok((self.is_public(scope, 501) && publication_uuid == PUBLICATION_UUID).then(publication))
    }

    async fn resolve_public_route(
        &self,
        scope: WikiPersistenceScope,
        publication_id: u64,
        canonical_route: &str,
    ) -> Result<Option<WikiPublicRouteMatch>, WikiPersistenceError> {
        if !self.is_public(scope, publication_id) {
            return Ok(None);
        }
        match canonical_route {
            "/guide/" => Ok(Some(WikiPublicRouteMatch {
                page: page(),
                matched_previous_route: false,
                redirect_status: None,
            })),
            "/old-guide/" => Ok(Some(WikiPublicRouteMatch {
                page: page(),
                matched_previous_route: true,
                redirect_status: Some(308),
            })),
            _ => Ok(None),
        }
    }

    async fn get_public_content_projection(
        &self,
        scope: WikiPersistenceScope,
        publication_id: u64,
        projection_uuid: &str,
        page_public_version: u64,
    ) -> Result<Option<WikiPublicPageProjection>, WikiPersistenceError> {
        Ok((self.is_public(scope, publication_id)
            && projection_uuid == PROJECTION_UUID
            && page_public_version == 7)
            .then(page))
    }

    async fn list_public_navigation(
        &self,
        request: ListWikiPublicNavigationRequest,
    ) -> Result<WikiPublicPageWindow, WikiPersistenceError> {
        if !self.is_public(request.scope, request.publication_id) {
            return Ok(WikiPublicPageWindow {
                items: Vec::new(),
                next: None,
            });
        }
        if request.after.is_none() {
            return Ok(WikiPublicPageWindow {
                items: vec![page()],
                next: Some(WikiPublicPageKeyset {
                    canonical_route: "/guide/".to_string(),
                    page_id: 601,
                }),
            });
        }
        Ok(WikiPublicPageWindow {
            items: Vec::new(),
            next: None,
        })
    }

    async fn search_public_pages(
        &self,
        request: SearchWikiPublicPagesRequest,
    ) -> Result<WikiPublicPageWindow, WikiPersistenceError> {
        self.list_public_navigation(ListWikiPublicNavigationRequest {
            scope: request.scope,
            publication_id: request.publication_id,
            locale: request.locale,
            after: request.after,
            limit: request.limit,
        })
        .await
    }
}

#[derive(Default)]
struct FakeDriveSource;

#[async_trait]
impl KnowledgeWikiDriveScope for FakeDriveSource {
    async fn ensure_raw_scope(
        &self,
        _request: EnsureKnowledgebaseRawScopeRequest,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        Err(KnowledgeWikiDriveSourceError::InvalidRequest(
            "not used by public provider tests".to_string(),
        ))
    }

    async fn retrieve_raw_scope(
        &self,
        _subscription_uuid: &str,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        Err(KnowledgeWikiDriveSourceError::InvalidRequest(
            "not used by public provider tests".to_string(),
        ))
    }
}

#[async_trait]
impl KnowledgeWikiDriveSource for FakeDriveSource {
    async fn resolve_source(
        &self,
        request: ResolveKnowledgeWikiSourceRequest,
    ) -> Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError> {
        let version = request
            .pinned_node_version_id
            .ok_or_else(|| KnowledgeWikiDriveSourceError::InvalidRequest("pin required".into()))?;
        Ok(KnowledgeWikiSourceResource {
            scope_type: "ROOT_SCOPE_SUBSCRIPTION".to_string(),
            subscription_uuid: request.subscription_uuid,
            scope_generation: "9".to_string(),
            normalized_relative_path: request.relative_path,
            resource_type: "FILE".to_string(),
            drive_node_id: "drive-node-guide".to_string(),
            drive_node_version_id: version,
            version_no: "3".to_string(),
            checksum_sha256_hex: source_checksum(),
            etag: "source-etag".to_string(),
            content_type: "text/markdown".to_string(),
            content_length: SOURCE_BYTES.len() as u64,
            last_modified: "2026-07-21T00:00:00Z".to_string(),
            scope_status: "ACTIVE".to_string(),
            node_status: "ACTIVE".to_string(),
            eligibility: "ELIGIBLE".to_string(),
        })
    }

    async fn read_pinned_source(
        &self,
        request: ReadKnowledgeWikiSourceRequest,
    ) -> Result<Vec<u8>, KnowledgeWikiDriveSourceError> {
        assert!(request.maximum_bytes >= SOURCE_BYTES.len() as u64);
        Ok(SOURCE_BYTES.to_vec())
    }
}

#[tokio::test]
async fn exact_route_returns_bound_handle_and_reviewed_redirect() {
    let (service, _) = service();
    let exact = service
        .resolve_route(ResolveWikiPublicRouteRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            route: "/guide/".to_string(),
            locale: Some("zh-CN".to_string()),
        })
        .await
        .expect("resolve exact public route");
    let WikiPublicRouteResolution::Page(exact) = exact else {
        panic!("exact route must resolve a page");
    };
    assert_eq!(exact.page.page_public_version, 7);
    assert!(!exact.content_handle.contains(PUBLICATION_UUID));

    let redirect = service
        .resolve_route(ResolveWikiPublicRouteRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            route: "/old-guide/".to_string(),
            locale: None,
        })
        .await
        .expect("resolve previous route");
    assert!(matches!(
        redirect,
        WikiPublicRouteResolution::Redirect {
            status: 308,
            canonical_route,
            ..
        } if canonical_route == "/guide/"
    ));
}

#[tokio::test]
async fn content_revalidates_public_state_and_exact_pinned_version() {
    let (service, store) = service();
    let resolved = service
        .resolve_route(ResolveWikiPublicRouteRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            route: "/guide/".to_string(),
            locale: None,
        })
        .await
        .expect("resolve page");
    let WikiPublicRouteResolution::Page(resolved) = resolved else {
        panic!("expected page");
    };
    let request = RetrieveWikiPublicContentRequest {
        scope: SCOPE,
        publication_uuid: PUBLICATION_UUID.to_string(),
        content_handle: resolved.content_handle.clone(),
    };
    let content = service
        .retrieve_content(request.clone())
        .await
        .expect("retrieve exact pinned content");
    assert_eq!(content.bytes, SOURCE_BYTES);
    assert_eq!(content.content_sha256, source_checksum());

    let wrong_scope = service
        .retrieve_content(RetrieveWikiPublicContentRequest {
            scope: WikiPersistenceScope {
                tenant_id: 102,
                organization_id: 202,
            },
            ..request.clone()
        })
        .await
        .expect_err("content handle must be tenant-bound");
    assert!(matches!(
        wrong_scope,
        KnowledgeWikiPublicProviderError::InvalidRequest(_)
    ));

    store.revoke();
    assert_eq!(
        service
            .retrieve_content(request)
            .await
            .expect_err("revoked publication must reject an old handle"),
        KnowledgeWikiPublicProviderError::NotFoundOrNotPublic
    );
}

#[tokio::test]
async fn cursors_are_bound_to_publication_query_locale_and_scope() {
    let (service, _) = service();
    let first = service
        .search_pages(SearchWikiPublicPageRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            query: "guide".to_string(),
            locale: Some("zh-CN".to_string()),
            cursor: None,
            page_size: Some(1),
        })
        .await
        .expect("first search page");
    let cursor = first.next_cursor.expect("next cursor");
    assert!(!cursor.contains("guide"));

    for request in [
        SearchWikiPublicPageRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            query: "different".to_string(),
            locale: Some("zh-CN".to_string()),
            cursor: Some(cursor.clone()),
            page_size: Some(1),
        },
        SearchWikiPublicPageRequest {
            scope: SCOPE,
            publication_uuid: PUBLICATION_UUID.to_string(),
            query: "guide".to_string(),
            locale: Some("en-US".to_string()),
            cursor: Some(cursor.clone()),
            page_size: Some(1),
        },
    ] {
        assert!(matches!(
            service.search_pages(request).await,
            Err(KnowledgeWikiPublicProviderError::InvalidRequest(_))
        ));
    }

    assert!(matches!(
        service
            .list_navigation(ListWikiPublicNavigationPageRequest {
                scope: SCOPE,
                publication_uuid: PUBLICATION_UUID.to_string(),
                locale: None,
                cursor: None,
                page_size: Some(201),
            })
            .await,
        Err(KnowledgeWikiPublicProviderError::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn provider_rejects_non_normalized_routes_before_store_access() {
    let (service, _) = service();
    for route in [
        "guide",
        "/../private/",
        "/encoded%2fpath/",
        "/double//path/",
    ] {
        assert!(matches!(
            service
                .resolve_route(ResolveWikiPublicRouteRequest {
                    scope: SCOPE,
                    publication_uuid: PUBLICATION_UUID.to_string(),
                    route: route.to_string(),
                    locale: None,
                })
                .await,
            Err(KnowledgeWikiPublicProviderError::InvalidRequest(_))
        ));
    }
}

fn service() -> (KnowledgeWikiPublicProviderService, Arc<FakePublicStore>) {
    let store = Arc::new(FakePublicStore::default());
    (
        KnowledgeWikiPublicProviderService::new(store.clone(), Arc::new(FakeDriveSource)),
        store,
    )
}

fn publication() -> WikiPublicPublication {
    WikiPublicPublication {
        id: 501,
        uuid: PUBLICATION_UUID.to_string(),
        scope: SCOPE,
        source_scope_uuid: "11111111-1111-4111-8111-111111111701".to_string(),
        title: "SDKWork Wiki".to_string(),
        description: None,
        homepage_source_path: "index.md".to_string(),
        default_locale: "zh-CN".to_string(),
        supported_locales: vec!["zh-CN".to_string()],
        navigation_mode: "DIRECTORY".to_string(),
        theme_key: "sdkwork-wiki-default".to_string(),
        theme_version: "1".to_string(),
        renderer_policy_version: "1".to_string(),
        search_enabled: true,
        robots_policy: "NOINDEX_NOFOLLOW".to_string(),
        sitemap_enabled: false,
        provider_generation: 3,
        navigation_generation: 4,
        search_generation: 5,
    }
}

fn page() -> WikiPublicPageProjection {
    WikiPublicPageProjection {
        id: 601,
        uuid: PROJECTION_UUID.to_string(),
        source_path: "guide.md".to_string(),
        canonical_route: "/guide/".to_string(),
        file_kind: WikiSourceFileKind::Page,
        media_type: "text/markdown".to_string(),
        size_bytes: SOURCE_BYTES.len() as u64,
        content_sha256: source_checksum(),
        title: Some("Guide".to_string()),
        description: None,
        locale: Some("zh-CN".to_string()),
        nav_order: Some(1),
        public_drive_version_uuid: "11111111-1111-4111-8111-111111111801".to_string(),
        page_public_version: 7,
        public_updated_at: "2026-07-21T00:00:00Z".to_string(),
    }
}

fn source_checksum() -> String {
    format!("sha256:{}", sha256_hash(SOURCE_BYTES))
}
