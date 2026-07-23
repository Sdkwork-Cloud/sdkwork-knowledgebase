use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Method, Request, StatusCode};
use sdkwork_intelligence_knowledgebase_service::{
    ports::knowledge_wiki_persistence::{
        WikiDriveEventProcessingState, WikiDriveEventReceipt, WikiDriveEventReceiveDisposition,
        WikiDriveEventType, WikiDriveInboxEvent, WikiPersistenceScope, WikiSourceFileKind,
    },
    wiki_event_consumer::{
        KnowledgeWikiDriveEventConsumerError, ReceiveKnowledgeWikiDriveWebhookRequest,
    },
    wiki_public_provider::{
        KnowledgeWikiPublicProviderError, ListWikiPublicNavigationPageRequest,
        ResolveWikiPublicRouteRequest, RetrieveWikiPublicContentRequest,
        RetrieveWikiPublicPublicationRequest, SearchWikiPublicPageRequest, WikiPublicContent,
        WikiPublicPageList, WikiPublicPageListItem, WikiPublicPageMetadata,
        WikiPublicPublicationMetadata, WikiPublicRouteResolution, WikiResolvedPublicPage,
    },
};
use sdkwork_routes_knowledgebase_internal_api::{
    build_router_with_services, KnowledgebaseDriveEventReceiver, KnowledgebaseWikiPublicProvider,
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

const CHANNEL_ID: &str = "kbraw:11111111-1111-4111-8111-111111111501";
const EVENT_ID: &str = "event-version-1";

#[derive(Default)]
struct FakeReceiver {
    requests: Mutex<Vec<ReceiveKnowledgeWikiDriveWebhookRequest>>,
}

#[async_trait]
impl KnowledgebaseDriveEventReceiver for FakeReceiver {
    async fn receive_drive_webhook(
        &self,
        request: ReceiveKnowledgeWikiDriveWebhookRequest,
    ) -> Result<WikiDriveEventReceipt, KnowledgeWikiDriveEventConsumerError> {
        self.requests
            .lock()
            .expect("request lock")
            .push(request.clone());
        Ok(WikiDriveEventReceipt {
            event: WikiDriveInboxEvent {
                id: 1,
                uuid: "inbox-1".to_string(),
                scope: WikiPersistenceScope {
                    tenant_id: 101,
                    organization_id: 202,
                },
                site_publication_id: 501,
                checkpoint_id: 701,
                source_event_id: request.event_id,
                event_type: WikiDriveEventType::VersionCommitted,
                sequence_no: 1,
                drive_node_uuid: "node-1".to_string(),
                drive_version_uuid: Some("version-1".to_string()),
                payload_sha256: format!(
                    "sha256:{}",
                    sdkwork_utils_rust::sha256_hash(request.payload_json.as_bytes())
                ),
                payload_json: request.payload_json,
                source_event_time: "2026-07-21T00:00:00Z".to_string(),
                processing_state: WikiDriveEventProcessingState::Received,
                attempt_count: 0,
                lease_token: None,
                version: 1,
            },
            disposition: WikiDriveEventReceiveDisposition::Ready,
        })
    }
}

#[derive(Default)]
struct FakeWikiProvider {
    scopes: Mutex<Vec<WikiPersistenceScope>>,
}

#[async_trait]
impl KnowledgebaseWikiPublicProvider for FakeWikiProvider {
    async fn retrieve_publication(
        &self,
        request: RetrieveWikiPublicPublicationRequest,
    ) -> Result<WikiPublicPublicationMetadata, KnowledgeWikiPublicProviderError> {
        self.scopes.lock().expect("scope lock").push(request.scope);
        Ok(WikiPublicPublicationMetadata {
            publication_uuid: request.publication_uuid,
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
        })
    }

    async fn resolve_route(
        &self,
        request: ResolveWikiPublicRouteRequest,
    ) -> Result<WikiPublicRouteResolution, KnowledgeWikiPublicProviderError> {
        self.scopes.lock().expect("scope lock").push(request.scope);
        Ok(WikiPublicRouteResolution::Page(WikiResolvedPublicPage {
            page: public_wiki_page(),
            content_handle: "content-1".to_string(),
        }))
    }

    async fn retrieve_content(
        &self,
        request: RetrieveWikiPublicContentRequest,
    ) -> Result<WikiPublicContent, KnowledgeWikiPublicProviderError> {
        self.scopes.lock().expect("scope lock").push(request.scope);
        Ok(WikiPublicContent {
            bytes: b"# Wiki".to_vec(),
            media_type: "text/markdown".to_string(),
            content_sha256:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
            page_public_version: 7,
        })
    }

    async fn list_navigation(
        &self,
        request: ListWikiPublicNavigationPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        self.scopes.lock().expect("scope lock").push(request.scope);
        Ok(WikiPublicPageList {
            items: vec![WikiPublicPageListItem {
                page: public_wiki_page(),
            }],
            next_cursor: Some("next-page".to_string()),
            page_size: request.page_size.unwrap_or(20),
        })
    }

    async fn search_pages(
        &self,
        request: SearchWikiPublicPageRequest,
    ) -> Result<WikiPublicPageList, KnowledgeWikiPublicProviderError> {
        self.scopes.lock().expect("scope lock").push(request.scope);
        Ok(WikiPublicPageList {
            items: vec![WikiPublicPageListItem {
                page: public_wiki_page(),
            }],
            next_cursor: None,
            page_size: request.page_size.unwrap_or(20),
        })
    }
}

fn public_wiki_page() -> WikiPublicPageMetadata {
    WikiPublicPageMetadata {
        projection_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
        canonical_route: "/guide/".to_string(),
        file_kind: WikiSourceFileKind::Page,
        media_type: "text/markdown".to_string(),
        size_bytes: 6,
        content_sha256: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        title: Some("Guide".to_string()),
        description: None,
        locale: Some("zh-CN".to_string()),
        nav_order: Some(1),
        page_public_version: 7,
        public_updated_at: "2026-07-21T00:00:00Z".to_string(),
    }
}

fn ingress_token(app_id: &str) -> String {
    format!("api_key_id=internal-test;tenant_id=101;organization_id=202;user_id=internal-service;app_id={app_id}")
}

fn test_app(receiver: Arc<FakeReceiver>, provider: Arc<FakeWikiProvider>) -> axum::Router {
    build_router_with_services(receiver, provider, "sdkwork-drive", "sdkwork-web")
}

fn event_body() -> String {
    serde_json::json!({
        "specversion": "1.0",
        "id": EVENT_ID,
        "source": "sdkwork.drive",
        "type": "drive.node.version.committed.v1",
        "time": "2026-07-21T00:00:00Z",
        "tenantId": "101",
        "organizationId": "202",
        "sequenceNo": "1",
        "data": { "spaceId": "drive-space-501" }
    })
    .to_string()
}

fn request(app_id: Option<&str>, body: impl Into<Body>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/internal/v3/api/knowledgebase/drive_events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-sdkwork-event-id", EVENT_ID)
        .header("x-sdkwork-event-timestamp", "1999999999")
        .header(
            "x-sdkwork-event-signature",
            format!("v1={}", "a".repeat(64)),
        )
        .header("x-sdkwork-event-retry-count", "0")
        .header("x-sdkwork-drive-channel-id", CHANNEL_ID)
        .header(
            "x-sdkwork-idempotency-key",
            format!("outbox-1:{CHANNEL_ID}"),
        );
    if let Some(app_id) = app_id {
        builder = builder.header("x-api-key", ingress_token(app_id));
    }
    builder.body(body.into()).expect("request should be valid")
}

#[tokio::test]
async fn route_requires_ingress_auth_and_drive_service_identity() {
    let receiver = Arc::new(FakeReceiver::default());
    let app = test_app(receiver.clone(), Arc::new(FakeWikiProvider::default()));

    let unauthenticated = app
        .clone()
        .oneshot(request(None, event_body()))
        .await
        .expect("unauthenticated request should be handled");
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let wrong_caller = app
        .clone()
        .oneshot(request(Some("another-service"), event_body()))
        .await
        .expect("wrong caller request should be handled");
    assert_eq!(wrong_caller.status(), StatusCode::FORBIDDEN);
    assert!(receiver.requests.lock().expect("request lock").is_empty());
}

#[tokio::test]
async fn route_preserves_exact_body_and_returns_standard_receipt() {
    let receiver = Arc::new(FakeReceiver::default());
    let app = test_app(receiver.clone(), Arc::new(FakeWikiProvider::default()));
    let body = event_body();

    let response = app
        .oneshot(request(Some("sdkwork-drive"), body.clone()))
        .await
        .expect("signed event request should be handled");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let response_json: serde_json::Value =
        serde_json::from_slice(&response_body).expect("response should be JSON");
    assert_eq!(response_json["code"], 0);
    assert_eq!(response_json["data"]["item"]["eventId"], EVENT_ID);
    assert_eq!(response_json["data"]["item"]["checkpointId"], "701");
    assert_eq!(response_json["data"]["item"]["disposition"], "READY");
    assert!(response_json["traceId"].as_str().is_some());

    let received = receiver.requests.lock().expect("request lock");
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].payload_json.as_bytes(), body.as_bytes());
    assert_eq!(received[0].channel_id, CHANNEL_ID);
}

#[tokio::test]
async fn route_rejects_duplicate_headers_and_oversized_bodies() {
    let receiver = Arc::new(FakeReceiver::default());
    let app = test_app(receiver.clone(), Arc::new(FakeWikiProvider::default()));
    let mut duplicate = request(Some("sdkwork-drive"), event_body());
    duplicate.headers_mut().append(
        "x-sdkwork-event-id",
        "event-version-2".parse().expect("header value"),
    );
    let duplicate_response = app
        .clone()
        .oneshot(duplicate)
        .await
        .expect("duplicate header request should be handled");
    assert_eq!(duplicate_response.status(), StatusCode::BAD_REQUEST);

    let oversized = app
        .oneshot(request(Some("sdkwork-drive"), "x".repeat(65_537)))
        .await
        .expect("oversized request should be handled");
    assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(receiver.requests.lock().expect("request lock").is_empty());
}

#[tokio::test]
async fn wiki_provider_routes_require_web_server_identity_and_principal_scope() {
    let receiver = Arc::new(FakeReceiver::default());
    let provider = Arc::new(FakeWikiProvider::default());
    let app = test_app(receiver, provider.clone());
    let uri =
        "/internal/v3/api/knowledgebase/wiki_publications/11111111-1111-4111-8111-111111111501";

    let unauthenticated = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("unauthenticated response");
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let drive_caller = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("x-api-key", ingress_token("sdkwork-drive"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("wrong caller response");
    assert_eq!(drive_caller.status(), StatusCode::FORBIDDEN);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("x-api-key", ingress_token("sdkwork-web"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("provider response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("response JSON");
    assert_eq!(value["data"]["item"]["providerGeneration"], "3");
    assert_eq!(
        provider.scopes.lock().expect("scope lock").as_slice(),
        &[WikiPersistenceScope {
            tenant_id: 101,
            organization_id: 202,
        }]
    );
}

#[tokio::test]
async fn wiki_provider_content_and_lists_preserve_binary_and_page_contracts() {
    let app = test_app(
        Arc::new(FakeReceiver::default()),
        Arc::new(FakeWikiProvider::default()),
    );
    let token = ingress_token("sdkwork-web");
    let content = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/internal/v3/api/knowledgebase/wiki_publications/11111111-1111-4111-8111-111111111501/contents/content-1")
                .header("x-api-key", &token)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("content response");
    assert_eq!(content.status(), StatusCode::OK);
    assert_eq!(content.headers()[header::CONTENT_TYPE], "text/markdown");
    assert_eq!(
        to_bytes(content.into_body(), usize::MAX)
            .await
            .expect("content bytes"),
        "# Wiki"
    );

    let navigation = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/internal/v3/api/knowledgebase/wiki_publications/11111111-1111-4111-8111-111111111501/navigation?page_size=1")
                .header("x-api-key", token)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("navigation response");
    assert_eq!(navigation.status(), StatusCode::OK);
    let body = to_bytes(navigation.into_body(), usize::MAX)
        .await
        .expect("navigation body");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("navigation JSON");
    assert_eq!(value["data"]["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        value["data"]["items"][0]["publicUpdatedAt"],
        "2026-07-21T00:00:00Z"
    );
    assert_eq!(value["data"]["pageInfo"]["mode"], "cursor");
    assert_eq!(value["data"]["pageInfo"]["nextCursor"], "next-page");
}

#[test]
fn route_manifest_is_internal_and_ingress_token_only() {
    let manifest = sdkwork_routes_knowledgebase_internal_api::internal_route_manifest();
    assert_eq!(manifest.routes().len(), 6);
    assert!(manifest
        .routes()
        .iter()
        .all(|route| route.auth == sdkwork_web_core::RouteAuth::IngressToken));
    assert!(manifest
        .routes()
        .iter()
        .any(|route| route.operation_id == "wikiPublications.contents.retrieve"));
    assert!(manifest
        .routes()
        .iter()
        .any(|route| route.operation_id == "wikiPublications.pages.search"));
}
