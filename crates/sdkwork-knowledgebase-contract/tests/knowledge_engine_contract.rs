use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_mode, implementation_id_from_provider, KnowledgeEngineId,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;

#[test]
fn okf_native_descriptor_maps_to_okf_mode() {
    let descriptor = descriptor_for_mode(KnowledgeAgentKnowledgeMode::OkfBundle);
    assert_eq!(descriptor.implementation_id, KnowledgeEngineId::OKF_NATIVE);
    assert_eq!(
        descriptor.knowledge_mode,
        KnowledgeAgentKnowledgeMode::OkfBundle
    );
    assert!(descriptor.native);
}

#[test]
fn rag_native_descriptor_maps_to_rag_mode() {
    let descriptor = descriptor_for_mode(KnowledgeAgentKnowledgeMode::Rag);
    assert_eq!(descriptor.implementation_id, KnowledgeEngineId::RAG_NATIVE);
    assert_eq!(descriptor.knowledge_mode, KnowledgeAgentKnowledgeMode::Rag);
    assert!(descriptor.native);
}

#[test]
fn external_mode_descriptor_is_non_native() {
    let descriptor = descriptor_for_mode(KnowledgeAgentKnowledgeMode::External);
    assert_eq!(
        descriptor.knowledge_mode,
        KnowledgeAgentKnowledgeMode::External
    );
    assert!(!descriptor.native);
}

#[test]
fn implementation_id_from_provider_accepts_catalog_forms() {
    assert_eq!(
        implementation_id_from_provider("dify").as_deref(),
        Some("engine.knowledge.external.dify")
    );
    assert_eq!(
        implementation_id_from_provider("engine.knowledge.external.dify").as_deref(),
        Some("engine.knowledge.external.dify")
    );
    assert_eq!(
        implementation_id_from_provider("provider.knowledge.external.dify").as_deref(),
        Some("engine.knowledge.external.dify")
    );
}

#[test]
fn external_engine_id_follows_vendor_pattern() {
    assert_eq!(
        KnowledgeEngineId::external("notion").0,
        "engine.knowledge.external.notion"
    );
    assert_eq!(
        KnowledgeEngineId::external_agent_provider("dify"),
        "provider.knowledge.external.dify"
    );
}

#[test]
fn connector_metadata_json_parses_dataset_id() {
    use sdkwork_knowledgebase_contract::source::dataset_id_from_connector_metadata_json;

    assert_eq!(
        dataset_id_from_connector_metadata_json(Some(r#"{"datasetId":"ds-42"}"#)),
        Some("ds-42".to_string())
    );
    assert_eq!(
        dataset_id_from_connector_metadata_json(Some(r#"{"dataset_id":"ds-snake"}"#)),
        Some("ds-snake".to_string())
    );
    assert_eq!(dataset_id_from_connector_metadata_json(None), None);
    assert_eq!(dataset_id_from_connector_metadata_json(Some("")), None);
}

#[test]
fn external_catalog_manifest_lists_registered_vendors() {
    const CATALOG: &str = include_str!("../../../external/knowledge-engines/catalog.manifest.json");
    let catalog: serde_json::Value =
        serde_json::from_str(CATALOG).expect("parse external engine catalog");
    let vendors = catalog["vendors"]
        .as_array()
        .expect("catalog vendors array");
    assert!(
        vendors.len() >= 12,
        "external catalog should register at least 12 OSS knowledge engines"
    );
    for entry in vendors {
        let vendor_id = entry["vendorId"].as_str().expect("vendorId");
        let implementation_id = entry["implementationId"]
            .as_str()
            .expect("implementationId");
        assert_eq!(
            implementation_id,
            format!("engine.knowledge.external.{vendor_id}")
        );
    }
}
