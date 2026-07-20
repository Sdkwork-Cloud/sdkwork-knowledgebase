use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_external_search_read, descriptor_for_mode, implementation_id_from_provider,
    KnowledgeEngineCapability, KnowledgeEngineId,
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
    assert!(descriptor.supports(KnowledgeEngineCapability::Search));
    assert!(descriptor.supports(KnowledgeEngineCapability::ReadDocument));
    assert!(descriptor.supports(KnowledgeEngineCapability::ListDocuments));
}

#[test]
fn rag_native_descriptor_maps_to_rag_mode() {
    let descriptor = descriptor_for_mode(KnowledgeAgentKnowledgeMode::Rag);
    assert_eq!(descriptor.implementation_id, KnowledgeEngineId::RAG_NATIVE);
    assert_eq!(descriptor.knowledge_mode, KnowledgeAgentKnowledgeMode::Rag);
    assert!(descriptor.native);
    assert!(descriptor.supports(KnowledgeEngineCapability::Search));
    assert!(descriptor.supports(KnowledgeEngineCapability::ReadDocument));
    assert!(descriptor.supports(KnowledgeEngineCapability::ListDocuments));
}

#[test]
fn external_mode_descriptor_is_non_native() {
    let descriptor = descriptor_for_mode(KnowledgeAgentKnowledgeMode::External);
    assert_eq!(
        descriptor.knowledge_mode,
        KnowledgeAgentKnowledgeMode::External
    );
    assert!(!descriptor.native);
    assert!(descriptor.capabilities.is_empty());
}

#[test]
fn external_search_read_descriptor_publishes_only_proven_capabilities() {
    let descriptor = descriptor_for_external_search_read("dify", "Dify");
    assert!(descriptor.supports(KnowledgeEngineCapability::Health));
    assert!(descriptor.supports(KnowledgeEngineCapability::Search));
    assert!(descriptor.supports(KnowledgeEngineCapability::ReadDocument));
    assert!(!descriptor.supports(KnowledgeEngineCapability::ListDocuments));
    assert!(!descriptor.supports(KnowledgeEngineCapability::Ingest));
    assert!(!descriptor.supports(KnowledgeEngineCapability::SyncSources));
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
fn connector_metadata_json_parses_workspace_slug() {
    use sdkwork_knowledgebase_contract::source::workspace_slug_from_connector_metadata_json;

    assert_eq!(
        workspace_slug_from_connector_metadata_json(Some(r#"{"workspaceSlug":"my-workspace"}"#)),
        Some("my-workspace".to_string())
    );
    assert_eq!(
        workspace_slug_from_connector_metadata_json(Some(r#"{"workspace_slug":"snake-ws"}"#)),
        Some("snake-ws".to_string())
    );
}

#[test]
fn compound_document_ref_parses_parent_and_child_ids() {
    use sdkwork_knowledgebase_contract::knowledge_engine::parse_compound_document_ref;

    assert_eq!(
        parse_compound_document_ref("doc-1#seg-9"),
        Some(("doc-1".to_string(), "seg-9".to_string()))
    );
    assert_eq!(parse_compound_document_ref("doc-1"), None);
    assert_eq!(parse_compound_document_ref("#seg"), None);
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
