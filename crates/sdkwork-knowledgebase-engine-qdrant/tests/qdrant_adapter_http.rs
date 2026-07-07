use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_engine_qdrant::{QdrantConnectorConfig, QdrantKnowledgeEngine};
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
async fn qdrant_search_uses_space_connector_metadata_collection_name() {
    let mock_server = MockServer::start().await;
    let collection_name = "policies";
    Mock::given(method("POST"))
        .and(path(format!("/collections/{collection_name}/points/query")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "points": [{
                    "id": "pt-9",
                    "score": 0.88,
                    "payload": {
                        "title": "Space Doc",
                        "text": "space scoped answer",
                        "source": "file://space-doc.txt"
                    }
                }]
            },
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let config = QdrantConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_collection_name: None,
        query_model: Some("sentence-transformers/all-minilm-l6-v2".to_string()),
        using_vector: None,
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("qdrant".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"datasetId":"policies"}"#.to_string()),
            }],
        )]),
    });
    let engine = QdrantKnowledgeEngine::with_config(config, Some(source_store));

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
    assert_eq!(result.hits[0].document.document_id, "42/Space Doc#pt-9");
    assert_eq!(result.hits[0].snippet, "space scoped answer");
}

#[tokio::test]
async fn qdrant_read_document_fetches_point_by_id() {
    let mock_server = MockServer::start().await;
    let collection_name = "policies";
    Mock::given(method("POST"))
        .and(path(format!("/collections/{collection_name}/points")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": [{
                "id": "pt-9",
                "payload": {
                    "title": "Space Doc",
                    "text": "full point body",
                    "source": "file://space-doc.txt"
                }
            }],
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let config = QdrantConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_collection_name: None,
        query_model: Some("sentence-transformers/all-minilm-l6-v2".to_string()),
        using_vector: None,
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("qdrant".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"datasetId":"policies"}"#.to_string()),
            }],
        )]),
    });
    let engine = QdrantKnowledgeEngine::with_config(config, Some(source_store));

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 42,
            document_id: "Space Doc#pt-9".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full point body");
    assert_eq!(document.document_id, "Space Doc#pt-9");
}

#[tokio::test]
async fn qdrant_list_documents_returns_collection_descriptor() {
    let mock_server = MockServer::start().await;
    let collection_name = "policies";
    Mock::given(method("GET"))
        .and(path(format!("/collections/{collection_name}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "points_count": 42
            },
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let config = QdrantConnectorConfig {
        base_url: mock_server.uri(),
        api_key: None,
        default_collection_name: Some(collection_name.to_string()),
        query_model: Some("sentence-transformers/all-minilm-l6-v2".to_string()),
        using_vector: None,
    };
    let engine = QdrantKnowledgeEngine::with_config(config, None);

    let list = engine
        .list_documents(KnowledgeEngineListRequest {
            tenant_id: 1,
            space_id: 42,
            limit: 10,
        })
        .await
        .expect("list");

    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].document_id, format!("42/{collection_name}"));
    assert_eq!(list.items[0].title, collection_name);
}
