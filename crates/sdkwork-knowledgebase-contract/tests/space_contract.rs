use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::{KnowledgeSpace, KnowledgeSpaceStatus};

#[test]
fn knowledge_space_serializes_snowflake_id_as_json_string() {
    let space = KnowledgeSpace {
        id: 332_473_351_878_475_800,
        uuid: "kb-test-uuid".to_string(),
        name: "Snowflake Space".to_string(),
        description: None,
        drive_space_id: Some("kb-drive-space".to_string()),
        status: KnowledgeSpaceStatus::Active,
        okf_bundle_initialized: true,
        knowledge_mode: KnowledgeAgentKnowledgeMode::Rag,
    };

    let json = serde_json::to_value(&space).expect("serialize knowledge space");
    assert_eq!(
        json.get("id").and_then(|value| value.as_str()),
        Some("332473351878475800")
    );
}

#[test]
fn knowledge_space_deserializes_snowflake_id_from_json_string() {
    let json = serde_json::json!({
        "id": "332473351878475800",
        "uuid": "kb-test-uuid",
        "name": "Snowflake Space",
        "status": "active",
        "okfBundleInitialized": true,
        "knowledgeMode": "rag"
    });

    let space: KnowledgeSpace =
        serde_json::from_value(json).expect("deserialize knowledge space from string id");
    assert_eq!(space.id, 332_473_351_878_475_800);
}
