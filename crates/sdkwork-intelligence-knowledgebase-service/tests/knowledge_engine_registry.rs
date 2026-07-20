use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    load_external_engines_from_catalog, InMemoryKnowledgeEngineRegistry, KnowledgeEngineRegistry,
};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError;

#[test]
fn registry_rejects_duplicate_implementation_ids_without_replacing_engine() {
    let engine = load_external_engines_from_catalog()
        .into_iter()
        .next()
        .expect("catalog fixture engine");
    let implementation_id = engine.descriptor().implementation_id;
    let mut registry = InMemoryKnowledgeEngineRegistry::new();

    registry
        .register(engine.clone())
        .expect("first registration");
    let error = registry
        .register(engine.clone())
        .expect_err("duplicate registration must be rejected");

    assert!(matches!(
        error,
        KnowledgeEngineError::Validation(message)
            if message.contains(&implementation_id)
    ));
    let resolved = registry
        .resolve_by_id(&implementation_id)
        .expect("original engine remains registered");
    assert!(std::sync::Arc::ptr_eq(&engine, &resolved));
}
