use sdkwork_drive_internal_sdk_generated_rust::{SdkworkConfig, SdkworkCustomClient};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_drive_source::{
    EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveScope, KnowledgeWikiDriveSource,
    KnowledgeWikiDriveSourceError, KnowledgeWikiSourceResource, ReadKnowledgeWikiSourceRequest,
    ResolveKnowledgeWikiSourceRequest, MAX_WIKI_SOURCE_READ_BYTES,
};
use sdkwork_knowledgebase_drive::KnowledgebaseDriveInternalSdkAdapter;
use sdkwork_utils_rust::sha256_hash;
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const INTERNAL_API_KEY: &str = "test-drive-internal-key";

#[tokio::test]
async fn generated_sdk_adapter_maps_root_scope_resolution_and_pinned_bytes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/internal/v3/api/drive/root_scope_subscriptions"))
        .and(header("X-API-Key", INTERNAL_API_KEY))
        .and(body_json(json!({
            "spaceId": "kb-space-001",
            "knowledgeBaseId": "knowledgebase-001",
            "rawFolderNodeId": "node-raw-root"
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
            raw_folder_node_id: "node-raw-root".to_string(),
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
    let checksum = sha256_hash(&body);
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
async fn generated_sdk_adapter_rejects_oversized_reads_before_network_io() {
    let server = MockServer::start().await;
    let adapter = adapter_for(&server);
    let error = adapter
        .read_pinned_source(ReadKnowledgeWikiSourceRequest {
            resource: source_resource(
                MAX_WIKI_SOURCE_READ_BYTES + 1,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
    json!({
        "code": 0,
        "message": "ok",
        "data": {
            "item": {
                "uuid": "root-scope-001",
                "spaceId": "kb-space-001",
                "consumerKind": "knowledgebase_raw",
                "consumerResourceId": "knowledgebase-001",
                "rootNodeId": "node-raw-root",
                "scopeStatus": "active",
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
                "scopeStatus": "active",
                "nodeStatus": "active",
                "eligibility": "eligible"
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
        scope_status: "active".to_string(),
        node_status: "active".to_string(),
        eligibility: "eligible".to_string(),
    }
}
