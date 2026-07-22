use sdkwork_drive_internal_sdk_generated_rust::{SdkworkConfig, SdkworkCustomClient};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_drive_source::{
    EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveEventDeliveryMode,
    KnowledgeWikiDriveScope, KnowledgeWikiDriveSource, KnowledgeWikiDriveSourceError,
    KnowledgeWikiSourceResource, ReadKnowledgeWikiSourceRequest,
    RenewKnowledgebaseRawScopeEventDeliveryRequest, ResolveKnowledgeWikiSourceRequest,
    MAX_WIKI_SOURCE_READ_BYTES,
};
use sdkwork_knowledgebase_drive::{
    KnowledgebaseDriveEventDeliveryConfig, KnowledgebaseDriveInternalSdkAdapter,
};
use sdkwork_utils_rust::{hmac_sha256, sha256_hash};
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const INTERNAL_API_KEY: &str = "test-drive-internal-key";
const ROOT_SCOPE_UUID: &str = "11111111-1111-4111-8111-111111111501";
const EVENT_SIGNING_SECRET: &str = "knowledgebase-drive-event-signing-secret-501";

#[test]
fn ingress_token_constructor_rejects_ambiguous_or_unbounded_credentials() {
    assert!(KnowledgebaseDriveInternalSdkAdapter::from_ingress_token(
        "http://127.0.0.1:18080",
        INTERNAL_API_KEY,
    )
    .is_ok());
    for token in [
        "short",
        "token with spaces",
        " test-drive-internal-key",
        "test-drive-internal-key\n",
    ] {
        assert!(matches!(
            KnowledgebaseDriveInternalSdkAdapter::from_ingress_token(
                "http://127.0.0.1:18080",
                token,
            ),
            Err(KnowledgeWikiDriveSourceError::InvalidRequest(_))
        ));
    }
    assert!(matches!(
        KnowledgebaseDriveInternalSdkAdapter::from_ingress_token(
            "http://127.0.0.1:18080",
            "x".repeat(4_097),
        ),
        Err(KnowledgeWikiDriveSourceError::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn generated_sdk_adapter_maps_root_scope_resolution_and_pinned_bytes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/internal/v3/api/drive/root_scope_subscriptions"))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .and(body_json(json!({
            "spaceId": "kb-space-001",
            "knowledgeBaseId": "knowledgebase-001"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(root_scope_envelope()))
        .expect(1)
        .mount(&server)
        .await;

    let adapter = adapter_for(&server);
    let scope = adapter
        .ensure_raw_scope(EnsureKnowledgebaseRawScopeRequest {
            drive_space_id: "kb-space-001".to_string(),
            knowledgebase_uuid: "knowledgebase-001".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(scope.subscription_uuid, "root-scope-001");
    assert_eq!(scope.consumer_kind, "knowledgebase_raw");
    assert_eq!(scope.knowledgebase_uuid, "knowledgebase-001");

    Mock::given(method("GET"))
        .and(path(
            "/internal/v3/api/drive/root_scope_subscriptions/root-scope-001",
        ))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(root_scope_envelope()))
        .expect(1)
        .mount(&server)
        .await;

    let retrieved = adapter.retrieve_raw_scope("root-scope-001").await.unwrap();
    assert_eq!(retrieved, scope);

    let body = b"# Install\n\nPinned Wiki source.\n".to_vec();
    let checksum = format!("sha256:{}", sha256_hash(&body));
    Mock::given(method("POST"))
        .and(path("/internal/v3/api/drive/resource_resolutions"))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .and(body_json(json!({
            "scopeType": "ROOT_SCOPE_SUBSCRIPTION",
            "scopeUuid": "root-scope-001",
            "relativePath": "guides/install.md",
            "pinnedGeneration": "generation-7",
            "pinnedNodeVersionId": "node-version-9"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(resource_envelope(&checksum, body.len() as u64)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resource = adapter
        .resolve_source(ResolveKnowledgeWikiSourceRequest {
            subscription_uuid: "root-scope-001".to_string(),
            relative_path: "guides/install.md".to_string(),
            pinned_generation: Some("generation-7".to_string()),
            pinned_node_version_id: Some("node-version-9".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(resource.drive_node_id, "node-install");
    assert_eq!(resource.drive_node_version_id, "node-version-9");
    assert_eq!(resource.content_length, body.len() as u64);
    assert_eq!(resource.checksum_sha256_hex, checksum);

    Mock::given(method("GET"))
        .and(path(
            "/internal/v3/api/drive/node_versions/node-version-9/content",
        ))
        .and(query_param("scopeType", "ROOT_SCOPE_SUBSCRIPTION"))
        .and(query_param("scopeUuid", "root-scope-001"))
        .and(query_param("relativePath", "guides/install.md"))
        .and(query_param("pinnedGeneration", "generation-7"))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .and(header(
            "Range",
            format!("bytes=0-{}", body.len().saturating_sub(1)),
        ))
        .and(header("If-Match", "\"etag-node-version-9\""))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("Content-Type", "text/markdown; charset=utf-8")
                .set_body_bytes(body.clone()),
        )
        .expect(1)
        .mount(&server)
        .await;

    let opened = adapter
        .read_pinned_source(ReadKnowledgeWikiSourceRequest {
            resource,
            maximum_bytes: 1024,
        })
        .await
        .unwrap();
    assert_eq!(opened, body);
}

#[tokio::test]
async fn generated_sdk_adapter_registers_signed_event_delivery_with_derived_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/internal/v3/api/drive/root_scope_subscriptions"))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(root_scope_envelope_with_uuid(ROOT_SCOPE_UUID)),
        )
        .expect(1)
        .mount(&server)
        .await;
    let callback_url =
        "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events";
    let verification_token =
        hmac_sha256(ROOT_SCOPE_UUID.as_bytes(), EVENT_SIGNING_SECRET.as_bytes());
    Mock::given(method("PUT"))
        .and(path(format!(
            "/internal/v3/api/drive/root_scope_subscriptions/{ROOT_SCOPE_UUID}/event_delivery"
        )))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .and(body_partial_json(json!({
            "address": callback_url,
            "verificationToken": verification_token
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "code": 0,
            "data": {
                "item": {
                    "channelId": format!("kbraw:{ROOT_SCOPE_UUID}"),
                    "subscriptionUuid": ROOT_SCOPE_UUID,
                    "address": callback_url,
                    "expirationEpochMs": "1999999999999",
                    "lifecycleStatus": "ACTIVE",
                    "version": "1",
                    "createdAt": "2026-07-21T00:00:00Z",
                    "updatedAt": "2026-07-21T00:00:00Z"
                }
            },
            "traceId": "trace-event-delivery"
        })))
        .expect(2)
        .mount(&server)
        .await;

    let adapter = adapter_for(&server)
        .with_event_delivery(KnowledgebaseDriveEventDeliveryConfig {
            callback_url: callback_url.to_string(),
            signing_master_secret: EVENT_SIGNING_SECRET.to_string(),
            channel_ttl_seconds: 86_400,
        })
        .expect("valid event delivery config");
    let scope = adapter
        .ensure_raw_scope(EnsureKnowledgebaseRawScopeRequest {
            drive_space_id: "kb-space-001".to_string(),
            knowledgebase_uuid: "knowledgebase-001".to_string(),
        })
        .await
        .expect("scope and delivery should be registered");
    assert_eq!(scope.subscription_uuid, ROOT_SCOPE_UUID);
    let renewed = adapter
        .renew_raw_scope_event_delivery(RenewKnowledgebaseRawScopeEventDeliveryRequest {
            subscription_uuid: ROOT_SCOPE_UUID.to_string(),
        })
        .await
        .expect("event delivery should renew through the generated Drive SDK");
    assert_eq!(renewed.subscription_uuid, ROOT_SCOPE_UUID);
    assert_eq!(renewed.channel_id, format!("kbraw:{ROOT_SCOPE_UUID}"));
    assert_eq!(renewed.expiration_epoch_ms, Some(1_999_999_999_999));
    assert_eq!(
        renewed.mode,
        KnowledgeWikiDriveEventDeliveryMode::CloudWebhook
    );
}

#[tokio::test]
async fn generated_sdk_adapter_rejects_oversized_reads_before_network_io() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);
    let error = adapter
        .read_pinned_source(ReadKnowledgeWikiSourceRequest {
            resource: source_resource(
                MAX_WIKI_SOURCE_READ_BYTES + 1,
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            maximum_bytes: MAX_WIKI_SOURCE_READ_BYTES,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeWikiDriveSourceError::InvalidRequest(_)
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn generated_sdk_adapter_rejects_pinned_content_checksum_mismatch() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);
    let body = b"tampered".to_vec();
    let resource = source_resource(
        body.len() as u64,
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    Mock::given(method("GET"))
        .and(path(
            "/internal/v3/api/drive/node_versions/node-version-9/content",
        ))
        .respond_with(ResponseTemplate::new(206).set_body_bytes(body))
        .expect(1)
        .mount(&server)
        .await;

    let error = adapter
        .read_pinned_source(ReadKnowledgeWikiSourceRequest {
            resource,
            maximum_bytes: 1024,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeWikiDriveSourceError::IntegrityFailed(_)
    ));
}

#[tokio::test]
async fn generated_sdk_adapter_rejects_noncanonical_drive_checksum() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/internal/v3/api/drive/resource_resolutions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resource_envelope(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            128,
        )))
        .expect(1)
        .mount(&server)
        .await;

    let error = adapter_for(&server)
        .resolve_source(ResolveKnowledgeWikiSourceRequest {
            subscription_uuid: "root-scope-001".to_string(),
            relative_path: "guides/install.md".to_string(),
            pinned_generation: None,
            pinned_node_version_id: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeWikiDriveSourceError::IntegrityFailed(_)
    ));
}

#[tokio::test]
async fn generated_sdk_adapter_rejects_traversal_before_resource_resolution() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);
    let error = adapter
        .resolve_source(ResolveKnowledgeWikiSourceRequest {
            subscription_uuid: "root-scope-001".to_string(),
            relative_path: "../private.md".to_string(),
            pinned_generation: None,
            pinned_node_version_id: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        KnowledgeWikiDriveSourceError::InvalidRequest(_)
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}

fn adapter_for(server: &MockServer) -> KnowledgebaseDriveInternalSdkAdapter {
    let mut config = SdkworkConfig::new(server.uri());
    config.max_response_body_bytes = MAX_WIKI_SOURCE_READ_BYTES as usize;
    let client = SdkworkCustomClient::new(config).unwrap();
    client.set_api_key(INTERNAL_API_KEY);
    KnowledgebaseDriveInternalSdkAdapter::new(client)
}

fn root_scope_envelope() -> serde_json::Value {
    root_scope_envelope_with_uuid("root-scope-001")
}

fn root_scope_envelope_with_uuid(uuid: &str) -> serde_json::Value {
    json!({
        "code": 0,
        "message": "ok",
        "data": {
            "item": {
                "uuid": uuid,
                "spaceId": "kb-space-001",
                "consumerKind": "knowledgebase_raw",
                "consumerResourceId": "knowledgebase-001",
                "rootNodeId": "node-raw-root",
                "scopeStatus": "ACTIVE",
                "version": "7",
                "createdAt": "2026-07-21T00:00:00Z",
                "updatedAt": "2026-07-21T00:01:00Z"
            }
        },
        "traceId": "trace-root-scope"
    })
}

fn resource_envelope(checksum: &str, content_length: u64) -> serde_json::Value {
    json!({
        "code": 0,
        "message": "ok",
        "data": {
            "item": {
                "scopeType": "ROOT_SCOPE_SUBSCRIPTION",
                "scopeUuid": "root-scope-001",
                "scopeGeneration": "generation-7",
                "normalizedRelativePath": "guides/install.md",
                "resourceType": "FILE",
                "nodeId": "node-install",
                "logicalNodeVersionId": "node-version-9",
                "versionNo": "9",
                "checksumSha256Hex": checksum,
                "etag": "\"etag-node-version-9\"",
                "contentType": "text/markdown; charset=utf-8",
                "contentLength": content_length.to_string(),
                "lastModified": "Tue, 21 Jul 2026 00:00:00 GMT",
                "scopeStatus": "ACTIVE",
                "nodeStatus": "ACTIVE",
                "eligibility": "ELIGIBLE"
            }
        },
        "traceId": "trace-resource"
    })
}

fn source_resource(content_length: u64, checksum: &str) -> KnowledgeWikiSourceResource {
    KnowledgeWikiSourceResource {
        scope_type: "ROOT_SCOPE_SUBSCRIPTION".to_string(),
        subscription_uuid: "root-scope-001".to_string(),
        scope_generation: "generation-7".to_string(),
        normalized_relative_path: "guides/install.md".to_string(),
        resource_type: "FILE".to_string(),
        drive_node_id: "node-install".to_string(),
        drive_node_version_id: "node-version-9".to_string(),
        version_no: "9".to_string(),
        checksum_sha256_hex: checksum.to_string(),
        etag: String::new(),
        content_type: "text/markdown; charset=utf-8".to_string(),
        content_length,
        last_modified: "Tue, 21 Jul 2026 00:00:00 GMT".to_string(),
        scope_status: "ACTIVE".to_string(),
        node_status: "ACTIVE".to_string(),
        eligibility: "ELIGIBLE".to_string(),
    }
}
