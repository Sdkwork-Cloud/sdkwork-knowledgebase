use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineError, KnowledgeEngineHealthStatus, KnowledgeEngineListRequest,
    KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_engine_weaviate::{
    WeaviateConnectorConfig, WeaviateKnowledgeEngine, DEFAULT_WEAVIATE_CONTENT_PROPERTY,
    DEFAULT_WEAVIATE_TITLE_PROPERTY,
};
use std::collections::HashMap;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct MockSourceStore {
    sources: HashMap<u64, Vec<KnowledgeSource>>,
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
async fn weaviate_search_uses_space_connector_metadata_class_name() {
    let mock_server = MockServer::start().await;
    let class_name = "KnowledgeChunk";
    Mock::given(method("POST"))
        .and(path("/v1/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "Get": {
                    class_name: [{
                        "title": "Space Doc",
                        "content": "space scoped answer",
                        "_additional": {
                            "id": "rec-9",
                            "certainty": 0.88
                        }
                    }]
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let config = WeaviateConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_class_name: None,
        title_property: DEFAULT_WEAVIATE_TITLE_PROPERTY.to_string(),
        content_property: DEFAULT_WEAVIATE_CONTENT_PROPERTY.to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("weaviate".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(format!(r#"{{"datasetId":"{class_name}"}}"#)),
            }],
        )]),
    });
    let engine = WeaviateKnowledgeEngine::with_config(config, Some(source_store));

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
    assert_eq!(result.hits[0].document.document_id, "42/Space Doc#rec-9");
    assert_eq!(result.hits[0].snippet, "space scoped answer");
}

#[tokio::test]
async fn weaviate_read_document_fetches_object_by_id() {
    let mock_server = MockServer::start().await;
    let class_name = "KnowledgeChunk";
    Mock::given(method("GET"))
        .and(path(format!("/v1/objects/{class_name}/rec-9")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "properties": {
                "title": "Space Doc",
                "content": "full object body",
                "source": "file://space-doc.txt"
            }
        })))
        .mount(&mock_server)
        .await;

    let config = WeaviateConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_class_name: None,
        title_property: DEFAULT_WEAVIATE_TITLE_PROPERTY.to_string(),
        content_property: DEFAULT_WEAVIATE_CONTENT_PROPERTY.to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("weaviate".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(format!(r#"{{"datasetId":"{class_name}"}}"#)),
            }],
        )]),
    });
    let engine = WeaviateKnowledgeEngine::with_config(config, Some(source_store));

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 42,
            document_id: "Space Doc#rec-9".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full object body");
    assert_eq!(document.document_id, "Space Doc#rec-9");
}

#[tokio::test]
async fn weaviate_list_documents_is_explicitly_unsupported() {
    let config = WeaviateConnectorConfig {
        base_url: "http://localhost:8080".to_string(),
        api_key: None,
        default_class_name: Some("KnowledgeChunk".to_string()),
        title_property: DEFAULT_WEAVIATE_TITLE_PROPERTY.to_string(),
        content_property: DEFAULT_WEAVIATE_CONTENT_PROPERTY.to_string(),
    };
    let engine = WeaviateKnowledgeEngine::with_config(config, None);

    let error = engine
        .list_documents(KnowledgeEngineListRequest {
            tenant_id: 1,
            space_id: 42,
            limit: 10,
        })
        .await
        .expect_err("list_documents must not synthesize class descriptors");

    assert!(matches!(error, KnowledgeEngineError::Unsupported(_)));
}

async fn assert_weaviate_health(upstream_status: u16, expected: KnowledgeEngineHealthStatus) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/.well-known/ready"))
        .respond_with(ResponseTemplate::new(upstream_status))
        .expect(if upstream_status >= 500 { 3 } else { 1 })
        .mount(&mock_server)
        .await;
    let engine = WeaviateKnowledgeEngine::with_config(
        WeaviateConnectorConfig {
            base_url: mock_server.uri(),
            api_key: None,
            default_class_name: Some("HealthClass".to_string()),
            title_property: DEFAULT_WEAVIATE_TITLE_PROPERTY.to_string(),
            content_property: DEFAULT_WEAVIATE_CONTENT_PROPERTY.to_string(),
        },
        None,
    );

    assert_eq!(engine.health().await.expect("health").status, expected);
}

#[tokio::test]
async fn weaviate_health_maps_upstream_availability() {
    assert_weaviate_health(200, KnowledgeEngineHealthStatus::Available).await;
    assert_weaviate_health(503, KnowledgeEngineHealthStatus::Degraded).await;
}
