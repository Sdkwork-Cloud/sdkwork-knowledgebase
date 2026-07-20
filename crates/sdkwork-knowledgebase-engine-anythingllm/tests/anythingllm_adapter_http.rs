use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineHealthStatus, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_engine_anythingllm::{
    AnythingLlmConnectorConfig, AnythingLlmKnowledgeEngine,
};
use std::collections::HashMap;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct MockSourceStore {
    sources: HashMap<u64, Vec<KnowledgeSource>>,
}

async fn assert_anythingllm_health(upstream_status: u16, expected: KnowledgeEngineHealthStatus) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspace/health-workspace"))
        .respond_with(ResponseTemplate::new(upstream_status))
        .expect(if upstream_status >= 500 { 3 } else { 1 })
        .mount(&mock_server)
        .await;
    let engine = AnythingLlmKnowledgeEngine::with_config(
        AnythingLlmConnectorConfig {
            base_url: mock_server.uri(),
            api_key: "health-key".to_string(),
            default_workspace_slug: Some("health-workspace".to_string()),
        },
        None,
    );

    assert_eq!(engine.health().await.expect("health").status, expected);
}

#[tokio::test]
async fn anythingllm_health_maps_upstream_availability() {
    assert_anythingllm_health(200, KnowledgeEngineHealthStatus::Available).await;
    assert_anythingllm_health(503, KnowledgeEngineHealthStatus::Degraded).await;
}

#[async_trait]
impl KnowledgeSourceStore for MockSourceStore {
    async fn create_source(
        &self,
        _record: CreateKnowledgeSourceRecord,
    ) -> Result<KnowledgeSource, KnowledgeSourceStoreError> {
        Err(KnowledgeSourceStoreError::Internal(
            "unsupported in test fake".to_string(),
        ))
    }

    async fn list_sources_for_space(
        &self,
        space_id: u64,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeSourceStoreError> {
        Ok(self.sources.get(&space_id).cloned().unwrap_or_default())
    }
}

#[tokio::test]
async fn anythingllm_search_uses_space_connector_metadata_workspace_slug() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspace/ws-space-42/vector-search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "chunk-9",
                "text": "space scoped answer",
                "score": 0.88,
                "metadata": {
                    "title": "Space Doc",
                    "url": "file://space-doc.txt"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = AnythingLlmConnectorConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
        default_workspace_slug: None,
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("anythingllm".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"workspaceSlug":"ws-space-42"}"#.to_string()),
            }],
        )]),
    });
    let engine = AnythingLlmKnowledgeEngine::with_config(config, Some(source_store));

    let result = engine
        .search(KnowledgeEngineSearchRequest {
            tenant_id: 1,
            space_id: 42,
            query: "hello".to_string(),
            top_k: 3,
        })
        .await
        .expect("search");

    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].document.document_id, "42/Space Doc#chunk-9");
    assert_eq!(result.hits[0].snippet, "space scoped answer");
}

#[tokio::test]
async fn anythingllm_read_document_resolves_chunk_from_vector_search() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspace/ws-space-42/vector-search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "id": "chunk-9",
                "text": "full chunk body",
                "score": 0.91,
                "metadata": {
                    "title": "Space Doc",
                    "url": "file://space-doc.txt"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = AnythingLlmConnectorConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
        default_workspace_slug: None,
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("anythingllm".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"workspaceSlug":"ws-space-42"}"#.to_string()),
            }],
        )]),
    });
    let engine = AnythingLlmKnowledgeEngine::with_config(config, Some(source_store));

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 42,
            document_id: "Space Doc#chunk-9".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full chunk body");
    assert_eq!(document.document_id, "Space Doc#chunk-9");
}
