use sdkwork_intelligence_knowledgebase_service::knowledge_engine::load_external_engines_from_catalog;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;

#[test]
fn catalog_loader_registers_catalog_and_stub_tier_vendors_only() {
    let engines = load_external_engines_from_catalog();
    assert_eq!(engines.len(), 9);

    let implementation_ids = engines
        .iter()
        .map(|engine| engine.descriptor().implementation_id.clone())
        .collect::<Vec<_>>();

    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("dify").0),
        "adapter-tier Dify must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("ragflow").0),
        "adapter-tier RAGFlow must register via runtime adapter crate, not catalog stub"
    );
    assert!(implementation_ids.contains(&KnowledgeEngineId::external("chroma").0));
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("onyx").0),
        "adapter-tier Onyx must register via runtime adapter crate, not catalog stub"
    );
}
