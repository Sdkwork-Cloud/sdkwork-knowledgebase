use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;
use sdkwork_knowledgebase_engine_open_webui::OpenWebuiKnowledgeEngine;

#[test]
fn open_webui_adapter_engine_registers_catalog_ids_when_unconfigured() {
    let engine = OpenWebuiKnowledgeEngine::stub();
    assert_eq!(
        engine.descriptor().implementation_id,
        KnowledgeEngineId::external("open-webui").0
    );
    assert_eq!(
        engine.descriptor().agent_provider_id,
        KnowledgeEngineId::external_agent_provider("open-webui")
    );
}
