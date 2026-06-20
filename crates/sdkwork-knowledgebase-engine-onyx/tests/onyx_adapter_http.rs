use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
};
use sdkwork_knowledgebase_engine_onyx::{OnyxConnectorConfig, OnyxKnowledgeEngine};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn onyx_search_maps_unified_search_results() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "title": "Policy Guide",
                "url": "https://example.com/policy",
                "content": "policy content",
                "source_type": "web"
            }]
        })))
        .mount(&mock_server)
        .await;

    let engine = OnyxKnowledgeEngine::with_config(OnyxConnectorConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
    });

    let result = engine
        .search(KnowledgeEngineSearchRequest {
            tenant_id: 1,
            space_id: 3,
            query: "policy".to_string(),
            top_k: 4,
        })
        .await
        .expect("search");

    assert_eq!(result.hits.len(), 1);
    assert_eq!(
        result.hits[0].document.document_id,
        "3/url:https://example.com/policy"
    );
    assert_eq!(result.hits[0].snippet, "policy content");
}

#[tokio::test]
async fn onyx_read_document_uses_open_urls() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/open_urls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "title": "Policy Guide",
                "content": "full policy body"
            }]
        })))
        .mount(&mock_server)
        .await;

    let engine = OnyxKnowledgeEngine::with_config(OnyxConnectorConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
    });

    let document = engine
        .read_document(KnowledgeEngineReadRequest {
            tenant_id: 1,
            space_id: 3,
            document_id: "url:https://example.com/policy".to_string(),
        })
        .await
        .expect("read");

    assert_eq!(document.title, "Policy Guide");
    assert_eq!(document.content, "full policy body");
}
