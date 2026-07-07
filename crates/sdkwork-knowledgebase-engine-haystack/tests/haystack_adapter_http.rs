use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::{
    CreateKnowledgeSourceRecord, KnowledgeSourceStore, KnowledgeSourceStoreError,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_contract::source::{KnowledgeSource, KnowledgeSourceType};
use sdkwork_knowledgebase_engine_haystack::{
    HaystackConnectorConfig, HaystackDeploymentMode, HaystackKnowledgeEngine,
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
async fn haystack_search_uses_space_connector_metadata_pipeline_name() {
    let mock_server = MockServer::start().await;
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
        default_pipeline: None,
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("haystack".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"datasetId":"retrieval_pipeline"}"#.to_string()),
            }],
        )]),
    });
    let engine = HaystackKnowledgeEngine::with_config(config, Some(source_store));

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
    assert_eq!(result.hits[0].document.document_id, "42/Space Doc#doc-9");
    assert_eq!(result.hits[0].snippet, "space scoped answer");
}

#[tokio::test]
async fn haystack_read_document_resolves_chunk_from_pipeline_run() {
    let mock_server = MockServer::start().await;
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
        default_pipeline: None,
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("haystack".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(r#"{"datasetId":"retrieval_pipeline"}"#.to_string()),
            }],
        )]),
    });
    let engine = HaystackKnowledgeEngine::with_config(config, Some(source_store));

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 42,
            document_id: "Space Doc#doc-9".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Space Doc");
    assert_eq!(document.content, "full chunk body");
    assert_eq!(document.document_id, "Space Doc#doc-9");
}

#[tokio::test]
async fn haystack_list_documents_returns_pipeline_descriptor() {
    let config = HaystackConnectorConfig {
        base_url: "http://localhost:1416".to_string(),
        api_key: None,
        default_pipeline: Some("retrieval_pipeline".to_string()),
        default_workspace: Some("my_workspace".to_string()),
        deployment_mode: HaystackDeploymentMode::Hayhooks,
        query_field: "query".to_string(),
    };
    let engine = HaystackKnowledgeEngine::with_config(config, None);

    let list = engine
        .list_documents(KnowledgeEngineListRequest {
            tenant_id: 1,
            space_id: 42,
            limit: 10,
        })
        .await
        .expect("list");

    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].document_id, "42/retrieval_pipeline");
    assert_eq!(list.items[0].title, "my_workspace/retrieval_pipeline");
}

#[tokio::test]
async fn haystack_cloud_search_uses_workspace_and_pipeline_from_metadata() {
    let mock_server = MockServer::start().await;
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
        api_key: Some("cloud-key".to_string()),
        default_pipeline: None,
        default_workspace: None,
        deployment_mode: HaystackDeploymentMode::DeepsetCloud,
        query_field: "query".to_string(),
    };
    let source_store = Arc::new(MockSourceStore {
        sources: HashMap::from([(
            42,
            vec![KnowledgeSource {
                id: 1,
                space_id: 42,
                source_type: KnowledgeSourceType::Connector,
                provider: Some("haystack".to_string()),
                drive_bucket: None,
                drive_prefix: None,
                connector_metadata_json: Some(
                    r#"{"datasetId":"cloud_pipeline","workspaceSlug":"ws-space-42"}"#.to_string(),
                ),
            }],
        )]),
    });
    let engine = HaystackKnowledgeEngine::with_config(config, Some(source_store));

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
    assert_eq!(result.hits[0].document.document_id, "42/Cloud Doc#cloud-1");
}
