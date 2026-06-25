use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    ExternalKnowledgeEngine, KnowledgeEngine,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineHealthStatus, KnowledgeEngineId,
};
use sdkwork_knowledgebase_engine_ragflow::{
    dataset_id_from_connector_metadata, RagflowKnowledgeEngine, RAGFLOW_AGENT_PROVIDER_ID,
    RAGFLOW_IMPLEMENTATION_ID,
};

#[tokio::test]
async fn ragflow_adapter_engine_registers_catalog_ids_when_unconfigured() {
    let engine = RagflowKnowledgeEngine::stub();
    let descriptor = engine.descriptor();

    assert_eq!(descriptor.implementation_id, RAGFLOW_IMPLEMENTATION_ID);
    assert_eq!(descriptor.agent_provider_id, RAGFLOW_AGENT_PROVIDER_ID);
    assert!(!descriptor.native);
    assert!(descriptor.display_name.contains("unconfigured"));

    let health = engine.health().await.expect("health");
    assert_eq!(health.status, KnowledgeEngineHealthStatus::Degraded);

    let connector = engine.connector_health().await.expect("connector health");
    assert_eq!(
        connector.implementation_id,
        KnowledgeEngineId::external("ragflow").0
    );
    assert_eq!(connector.status, KnowledgeEngineHealthStatus::Degraded);
}

#[test]
fn ragflow_connector_metadata_parses_dataset_id() {
    assert_eq!(
        dataset_id_from_connector_metadata(Some(r#"{"datasetId":"ds-123"}"#)),
        Some("ds-123".to_string())
    );
    assert_eq!(
        dataset_id_from_connector_metadata(Some(r#"{"dataset_id":"ds-456"}"#)),
        Some("ds-456".to_string())
    );
    assert_eq!(dataset_id_from_connector_metadata(None), None);
}
