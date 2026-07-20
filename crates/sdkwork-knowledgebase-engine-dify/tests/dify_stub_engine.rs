use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    ExternalKnowledgeEngine, KnowledgeEngine,
};
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineHealthStatus, KnowledgeEngineId,
};
use sdkwork_knowledgebase_engine_dify::{
    DifyKnowledgeEngine, DIFY_AGENT_PROVIDER_ID, DIFY_IMPLEMENTATION_ID,
};

#[tokio::test]
async fn dify_adapter_engine_registers_catalog_ids_when_unconfigured() {
    let engine = DifyKnowledgeEngine::stub();
    let descriptor = engine.descriptor();

    assert_eq!(descriptor.implementation_id, DIFY_IMPLEMENTATION_ID);
    assert_eq!(descriptor.agent_provider_id, DIFY_AGENT_PROVIDER_ID);
    assert!(!descriptor.native);
    assert!(descriptor.display_name.contains("unconfigured"));

    let health = engine.health().await.expect("health");
    assert_eq!(health.status, KnowledgeEngineHealthStatus::Degraded);

    let connector = engine.connector_health().await.expect("connector health");
    assert_eq!(
        connector.implementation_id,
        KnowledgeEngineId::external("dify").0
    );
    assert_eq!(connector.status, KnowledgeEngineHealthStatus::Degraded);
}
