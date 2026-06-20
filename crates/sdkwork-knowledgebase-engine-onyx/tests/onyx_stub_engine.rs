use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;
use sdkwork_knowledgebase_engine_onyx::OnyxKnowledgeEngine;

#[test]
fn onyx_adapter_engine_registers_catalog_ids_when_unconfigured() {
    let engine = OnyxKnowledgeEngine::stub();
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::external("onyx").0
    );
    assert_eq!(
        engine.descriptor().agent_provider_id,
        KnowledgeEngineId::external_agent_provider("onyx")
    );
}
