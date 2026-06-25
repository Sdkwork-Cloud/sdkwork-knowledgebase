use sdkwork_intelligence_knowledgebase_service::knowledge_engine::load_external_engines_from_catalog;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineId;

#[test]
fn catalog_loader_registers_catalog_and_stub_tier_vendors_only() {
    let engines = load_external_engines_from_catalog();
    assert_eq!(engines.len(), 2);

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
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("anythingllm").0),
        "adapter-tier AnythingLLM must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("open-webui").0),
        "adapter-tier Open WebUI must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("flowise").0),
        "adapter-tier Flowise must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("chroma").0),
        "adapter-tier Chroma must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("qdrant").0),
        "adapter-tier Qdrant must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("weaviate").0),
        "adapter-tier Weaviate must register via runtime adapter crate, not catalog stub"
    );
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("haystack").0),
        "adapter-tier Haystack must register via runtime adapter crate, not catalog stub"
    );
    assert!(implementation_ids.contains(&KnowledgeEngineId::external("langchain").0));
    assert!(
        !implementation_ids.contains(&KnowledgeEngineId::external("onyx").0),
        "adapter-tier Onyx must register via runtime adapter crate, not catalog stub"
    );
}
