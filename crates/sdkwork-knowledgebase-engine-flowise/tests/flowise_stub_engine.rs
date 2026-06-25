use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;
use sdkwork_knowledgebase_engine_flowise::FlowiseKnowledgeEngine;

#[test]
fn flowise_adapter_engine_registers_catalog_ids_when_unconfigured() {
    let engine = FlowiseKnowledgeEngine::stub();
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::external("flowise").0
    );
    assert_eq!(
        engine.descriptor().agent_provider_id,
        KnowledgeEngineId::external_agent_provider("flowise")
    );
}
