use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineError, KnowledgeEngineHealthStatus, KnowledgeEngineListRequest,
    KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_engine_haystack::{
    HaystackConnectorConfig, HaystackDeploymentMode, HaystackKnowledgeEngine,
};
use sdkwork_knowledgebase_test_support::provider_execution::{
    knowledge_execution_context, provider_execution_context,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
// Wiremock fixtures run on loopback; the provider SSRF protection fails closed unless
// this explicit test-only allowance is set (never set it in a deployed environment).
fn allow_test_loopback() {
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_PROVIDER_RUNTIME_ALLOW_LOOPBACK", "1");
}

#[tokio::test]
async fn haystack_search_uses_configured_remote_resource_id() {
    allow_test_loopback();    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/retrieval_pipeline/run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retriever": {
                "documents": [{
                    "id": "doc-9",
                    "content": "space scoped answer",
                    "meta": {
                        "title": "Space Doc",
                        "source": "file://space-doc.txt"
                    },
                    "score": 0.88
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    let config = HaystackConnectorConfig {
        base_url: mock_server.uri(),
        api_key: None,
        default_pipeline: Some("retrieval_pipeline".to_string()),
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let engine = HaystackKnowledgeEngine::with_config(config);

    let result = engine
        .search(
            &provider_execution_context(1, 2, 42, 7, "trace-adapter-search"),
            KnowledgeEngineSearchRequest {
                tenant_id: 1,
                space_id: 42,
                query: "hello".to_string(),
                top_k: 3,
            },
        )
        .await
        .expect("search");

    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].document.document_id, "42/Space Doc#doc-9");
    assert_eq!(result.hits[0].snippet, "space scoped answer");
}

#[tokio::test]
async fn haystack_read_document_resolves_chunk_from_pipeline_run() {
    allow_test_loopback();    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/retrieval_pipeline/run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retriever": {
                "documents": [{
                    "id": "doc-9",
                    "content": "full chunk body",
                    "meta": {
                        "title": "Space Doc",
                        "source": "file://space-doc.txt"
                    }
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    let config = HaystackConnectorConfig {
        base_url: mock_server.uri(),
        api_key: None,
        default_pipeline: Some("retrieval_pipeline".to_string()),
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let engine = HaystackKnowledgeEngine::with_config(config);

    let document = engine
        .read_document(
            &provider_execution_context(1, 2, 42, 7, "trace-adapter-read"),
            KnowledgeEngineReadRequest {
                tenant_id: 1,
                space_id: 42,
                document_id: "Space Doc#doc-9".to_string(),
            },
        )
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full chunk body");
    assert_eq!(document.document_id, "Space Doc#doc-9");
}

#[tokio::test]
async fn haystack_list_documents_is_explicitly_unsupported() {
    allow_test_loopback();    let config = HaystackConnectorConfig {
        base_url: "http://localhost:1416".to_string(),
        api_key: None,
        default_pipeline: Some("retrieval_pipeline".to_string()),
        default_workspace: Some("my_workspace".to_string()),
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let engine = HaystackKnowledgeEngine::with_config(config);

    let error = engine
        .list_documents(
            &knowledge_execution_context(1, 1, 42, None, "trace-haystack-list"),
            KnowledgeEngineListRequest {
                tenant_id: 1,
                space_id: 42,
                limit: 10,
            },
        )
        .await
        .expect_err("list_documents must not synthesize pipeline descriptors");

    assert!(matches!(error, KnowledgeEngineError::Unsupported(_)));
}

async fn assert_haystack_health(upstream_status: u16, expected: KnowledgeEngineHealthStatus) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/status"))
        .respond_with(ResponseTemplate::new(upstream_status))
        .expect(if upstream_status >= 500 { 3 } else { 1 })
        .mount(&mock_server)
        .await;
    let engine = HaystackKnowledgeEngine::with_config(HaystackConnectorConfig {
        base_url: mock_server.uri(),
        api_key: None,
        default_pipeline: Some("health-pipeline".to_string()),
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    });

    assert_eq!(engine.health().await.expect("health").status, expected);
}

#[tokio::test]
async fn haystack_health_maps_upstream_availability() {
    allow_test_loopback();    assert_haystack_health(200, KnowledgeEngineHealthStatus::Available).await;
    assert_haystack_health(503, KnowledgeEngineHealthStatus::Degraded).await;
}

#[tokio::test]
async fn haystack_cloud_search_uses_configured_workspace_and_remote_resource() {
    allow_test_loopback();    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws-space-42/pipelines/cloud_pipeline/search",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "documents": [{
                    "id": "cloud-1",
                    "content": "cloud scoped answer",
                    "meta": {
                        "title": "Cloud Doc",
                        "source": "file://cloud.txt"
                    }
                }]
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = HaystackConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some(zeroize::Zeroizing::new("cloud-key".to_string())),
        default_pipeline: Some("cloud_pipeline".to_string()),
        default_workspace: Some("ws-space-42".to_string()),
        deployment_mode: HaystackDeploymentMode::DeepsetCloud,
        query_field: "query".to_string(),
    };
    let engine = HaystackKnowledgeEngine::with_config(config);

    let result = engine
        .search(
            &provider_execution_context(1, 2, 42, 7, "trace-adapter-cloud-search"),
            KnowledgeEngineSearchRequest {
                tenant_id: 1,
                space_id: 42,
                query: "hello".to_string(),
                top_k: 3,
            },
        )
        .await
        .expect("search");

    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].document.document_id, "42/Cloud Doc#cloud-1");
}
