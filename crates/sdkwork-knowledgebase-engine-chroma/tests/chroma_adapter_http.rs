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
use sdkwork_knowledgebase_engine_chroma::{
    ChromaConnectorConfig, ChromaKnowledgeEngine, DEFAULT_CHROMA_DATABASE, DEFAULT_CHROMA_TENANT,
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

fn collection_base_path(collection_id: &str) -> String {
    format!(
        "/api/v2/tenants/{DEFAULT_CHROMA_TENANT}/databases/{DEFAULT_CHROMA_DATABASE}/collections/{collection_id}"
    )
}

#[tokio::test]
async fn chroma_search_uses_space_connector_metadata_collection_id() {
    let mock_server = MockServer::start().await;
    let collection_id = "603a7b51-ae7c-4b0a-8865-e454ed2f6766";
    Mock::given(method("POST"))
        .and(path(format!(
            "{}/query",
            collection_base_path(collection_id)
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ids": [["rec-9"]],
            "documents": [["space scoped answer"]],
            "metadatas": [[{
                "title": "Space Doc",
                "source": "file://space-doc.txt"
            }]],
            "distances": [[0.1]]
        })))
        .mount(&mock_server)
        .await;

    let config = ChromaConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_collection_id: None,
        tenant: DEFAULT_CHROMA_TENANT.to_string(),
        database: DEFAULT_CHROMA_DATABASE.to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("chroma".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(format!(r#"{{"datasetId":"{collection_id}"}}"#)),
            }],
        )]),
    });
    let engine = ChromaKnowledgeEngine::with_config(config, Some(source_store));

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
async fn chroma_read_document_fetches_record_by_id() {
    let mock_server = MockServer::start().await;
    let collection_id = "603a7b51-ae7c-4b0a-8865-e454ed2f6766";
    Mock::given(method("POST"))
        .and(path(format!("{}/get", collection_base_path(collection_id))))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ids": ["rec-9"],
            "documents": ["full record body"],
            "metadatas": [{
                "title": "Space Doc",
                "source": "file://space-doc.txt"
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = ChromaConnectorConfig {
        base_url: mock_server.uri(),
        api_key: Some("test-api-key".to_string()),
        default_collection_id: None,
        tenant: DEFAULT_CHROMA_TENANT.to_string(),
        database: DEFAULT_CHROMA_DATABASE.to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("chroma".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(format!(r#"{{"datasetId":"{collection_id}"}}"#)),
            }],
        )]),
    });
    let engine = ChromaKnowledgeEngine::with_config(config, Some(source_store));

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 42,
            document_id: "Space Doc#rec-9".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full record body");
    assert_eq!(document.document_id, "Space Doc#rec-9");
}

#[tokio::test]
async fn chroma_list_documents_is_explicitly_unsupported() {
    let collection_id = "603a7b51-ae7c-4b0a-8865-e454ed2f6766";
    let config = ChromaConnectorConfig {
        base_url: "http://localhost:8000".to_string(),
        api_key: None,
        default_collection_id: Some(collection_id.to_string()),
        tenant: DEFAULT_CHROMA_TENANT.to_string(),
        database: DEFAULT_CHROMA_DATABASE.to_string(),
    };
    let engine = ChromaKnowledgeEngine::with_config(config, None);

    let error = engine
        .list_documents(KnowledgeEngineListRequest {
            tenant_id: 1,
            space_id: 42,
            limit: 10,
        })
        .await
        .expect_err("list_documents must not synthesize collection descriptors");

    assert!(matches!(error, KnowledgeEngineError::Unsupported(_)));
}

async fn assert_chroma_health(upstream_status: u16, expected: KnowledgeEngineHealthStatus) {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/heartbeat"))
        .respond_with(ResponseTemplate::new(upstream_status))
        .expect(if upstream_status >= 500 { 3 } else { 1 })
        .mount(&mock_server)
        .await;
    let engine = ChromaKnowledgeEngine::with_config(
        ChromaConnectorConfig {
            base_url: mock_server.uri(),
            api_key: None,
            default_collection_id: Some("health-collection".to_string()),
            tenant: DEFAULT_CHROMA_TENANT.to_string(),
            database: DEFAULT_CHROMA_DATABASE.to_string(),
        },
        None,
    );

    assert_eq!(engine.health().await.expect("health").status, expected);
}

#[tokio::test]
async fn chroma_health_maps_upstream_availability() {
    assert_chroma_health(200, KnowledgeEngineHealthStatus::Available).await;
    assert_chroma_health(503, KnowledgeEngineHealthStatus::Degraded).await;
}
