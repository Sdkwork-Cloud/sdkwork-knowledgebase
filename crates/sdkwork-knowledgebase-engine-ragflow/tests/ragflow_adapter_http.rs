use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineSearchRequest;
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_engine_ragflow::{
    RagflowConnectorConfig, RagflowKnowledgeEngine, RAGFLOW_IMPLEMENTATION_ID,
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
            "not implemented".to_string(),
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
async fn ragflow_search_uses_space_connector_metadata_dataset_id() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/retrieval"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 0,
            "data": {
                "chunks": [{
                    "id": "chunk-9",
                    "content": "space scoped ragflow answer",
                    "document_id": "doc-42",
                    "document_keyword": "Space Doc",
                    "similarity": 0.91
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    let config = RagflowConnectorConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
        default_dataset_id: None,
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("ragflow".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"datasetId":"ds-space-42"}"#.to_string()),
            }],
        )]),
    });
    let engine = RagflowKnowledgeEngine::with_config(config, Some(source_store));

    let result = engine
        .search(KnowledgeEngineSearchRequest {
            tenant_id: 1,
            space_id: 42,
            query: "hello".to_string(),
            top_k: 3,
        })
        .await
        .expect("search");

    assert_eq!(result.implementation_id, RAGFLOW_IMPLEMENTATION_ID);
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].document.document_id, "42/doc-42#chunk-9");
    assert_eq!(result.hits[0].snippet, "space scoped ragflow answer");
}
