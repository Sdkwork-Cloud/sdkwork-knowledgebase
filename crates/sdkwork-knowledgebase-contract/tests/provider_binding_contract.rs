use sdkwork_knowledgebase_contract::provider_binding::{
    KnowledgeEngineDataScope, KnowledgeEngineExecutionContext, KnowledgeEngineProviderBindingState,
    KnowledgeEngineProviderCredentialReference, KnowledgeEngineProviderCredentialRotationState,
    KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationState,
};

#[test]
fn provider_lifecycle_states_use_stable_snake_case_wire_values() {
    assert_eq!(
        serde_json::to_string(&KnowledgeEngineProviderBindingState::Active).expect("binding state"),
        "\"active\""
    );
    assert_eq!(
        serde_json::to_string(&KnowledgeEngineProviderMigrationState::RollingBack)
            .expect("migration state"),
        "\"rolling_back\""
    );
}

#[test]
fn provider_credential_read_model_cannot_serialize_secret_locator() {
    let credential = KnowledgeEngineProviderCredentialReference {
        id: 1,
        uuid: "credential-1".to_string(),
        tenant_id: 100_001,
        organization_id: 7,
        implementation_id: "engine.knowledge.external.dify".to_string(),
        display_name: "Dify credential".to_string(),
        rotation_state: KnowledgeEngineProviderCredentialRotationState::Current,
        last_rotated_at: None,
        created_by: "tenant-admin".to_string(),
        updated_by: "tenant-admin".to_string(),
        created_at: "2026-07-20T00:00:00Z".to_string(),
        updated_at: "2026-07-20T00:00:00Z".to_string(),
        version: 0,
    };

    let json = serde_json::to_string(&credential).expect("credential JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("credential value");
    assert_eq!(value["id"], "1");
    assert_eq!(value["tenantId"], "100001");
    assert_eq!(value["organizationId"], "7");
    assert_eq!(value["version"], "0");
    assert!(!json.contains("referenceLocator"));
    assert!(!json.contains("secret://"));
}

#[test]
fn execution_context_round_trips_all_security_scope_dimensions() {
    let context = KnowledgeEngineExecutionContext {
        tenant_id: 100_001,
        organization_id: 7,
        actor_id: "actor-42".to_string(),
        permission_scope: vec!["knowledge.platform.manage".to_string()],
        data_scope: KnowledgeEngineDataScope {
            allowed_space_ids: vec![42],
            allowed_source_ids: vec![8],
            allowed_document_ids: vec!["doc-9".to_string()],
        },
        space_id: 42,
        binding_id: Some(3),
        trace_id: "trace-provider-contract".to_string(),
        deadline_unix_ms: 1_800_000_000_000,
    };

    let json = serde_json::to_string(&context).expect("execution context JSON");
    let decoded: KnowledgeEngineExecutionContext =
        serde_json::from_str(&json).expect("execution context decode");
    assert_eq!(decoded, context);
}

#[test]
fn provider_migration_read_model_uses_string_int64_and_hides_worker_state() {
    let operation = KnowledgeEngineProviderMigrationOperation {
        id: 91,
        uuid: "f4d884f2-8d4b-43e0-a6f8-c02113c90c89".to_string(),
        tenant_id: 100_001,
        organization_id: 7,
        space_id: 42,
        source_binding_id: 81,
        target_binding_id: 82,
        operation_state: KnowledgeEngineProviderMigrationState::Observing,
        requested_by: "tenant-admin".to_string(),
        attempt_count: 4,
        cutover_at: Some("2026-07-20T00:00:00Z".to_string()),
        observation_until: Some("2026-07-20T00:01:00Z".to_string()),
        completed_at: None,
        last_error_category: None,
        created_at: "2026-07-19T23:59:00Z".to_string(),
        updated_at: "2026-07-20T00:00:00Z".to_string(),
        version: 8,
    };

    let value = serde_json::to_value(operation).expect("migration operation JSON");
    for (field, expected) in [
        ("id", "91"),
        ("tenantId", "100001"),
        ("organizationId", "7"),
        ("spaceId", "42"),
        ("sourceBindingId", "81"),
        ("targetBindingId", "82"),
        ("version", "8"),
    ] {
        assert_eq!(value[field], expected);
    }
    for internal_field in ["checkpoint", "claimOwner", "claimToken", "leaseExpiresAt"] {
        assert!(value.get(internal_field).is_none());
    }
}
