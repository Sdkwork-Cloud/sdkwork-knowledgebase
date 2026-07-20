use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_knowledgebase_and_install_schema, SqliteKnowledgeSpaceStore,
    SqlxKnowledgeEngineProviderBindingStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::{
    KnowledgeEngineProviderBindingStore, KnowledgeEngineProviderBindingStoreError,
    KnowledgeEngineProviderScope, RecordKnowledgeEngineProviderTestResult,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineCapability;
use sdkwork_knowledgebase_contract::provider_binding::{
    CreateKnowledgeEngineProviderBindingRequest,
    CreateKnowledgeEngineProviderCredentialReferenceRequest, KnowledgeEngineProviderBindingState,
    ListKnowledgeEngineProviderBindingsRequest, UpdateKnowledgeEngineProviderBindingRequest,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn provider_binding_lifecycle_is_scoped_versioned_and_atomically_switchable() {
    let pool = provider_test_pool("provider-binding-lifecycle").await;
    let scope = KnowledgeEngineProviderScope {
        tenant_id: 100_001,
        organization_id: 7_001,
    };
    let space_store =
        SqliteKnowledgeSpaceStore::new(pool.clone(), scope.tenant_id, scope.organization_id);
    let space = space_store
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Provider Binding Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode:
                sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode::External,
        })
        .await
        .expect("create external space");
    let store = SqlxKnowledgeEngineProviderBindingStore::new(pool);

    let credential = store
        .create_credential_reference(
            scope,
            "tenant-admin",
            CreateKnowledgeEngineProviderCredentialReferenceRequest {
                implementation_id: "engine.knowledge.external.dify".to_string(),
                display_name: "Dify production credential".to_string(),
                reference_locator: "secret://knowledgebase/dify/primary".to_string(),
            },
        )
        .await
        .expect("create credential reference");
    let credential_json = serde_json::to_string(&credential).expect("serialize credential view");
    assert!(!credential_json.contains("secret://"));
    assert!(!credential_json.contains("referenceLocator"));

    let first = create_tested_binding(
        &store,
        scope,
        space.id,
        "engine.knowledge.external.dify",
        "dataset",
        "dataset-a",
        Some(credential.id),
    )
    .await;
    let first = store
        .activate_binding(scope, first.id, "tenant-admin", first.version)
        .await
        .expect("activate first binding");
    assert_eq!(
        first.lifecycle_state,
        KnowledgeEngineProviderBindingState::Active
    );
    assert_eq!(
        store
            .get_active_binding_for_space(scope, space.id)
            .await
            .expect("resolve active")
            .expect("active binding")
            .id,
        first.id
    );

    let second = create_tested_binding(
        &store,
        scope,
        space.id,
        "engine.knowledge.external.ragflow",
        "dataset",
        "dataset-b",
        None,
    )
    .await;
    let second = store
        .activate_binding(scope, second.id, "tenant-admin", second.version)
        .await
        .expect("atomically switch binding");
    assert_eq!(
        second.lifecycle_state,
        KnowledgeEngineProviderBindingState::Active
    );
    assert_eq!(
        store
            .get_binding(scope, first.id)
            .await
            .expect("retained predecessor")
            .lifecycle_state,
        KnowledgeEngineProviderBindingState::Disabled
    );

    let wrong_organization = KnowledgeEngineProviderScope {
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id + 1,
    };
    assert!(store
        .get_active_binding_for_space(wrong_organization, space.id)
        .await
        .expect("isolated lookup")
        .is_none());

    let page = store
        .list_bindings(
            scope,
            ListKnowledgeEngineProviderBindingsRequest {
                space_id: Some(space.id),
                lifecycle_state: None,
                cursor: None,
                page_size: Some(1),
            },
        )
        .await
        .expect("list first page");
    assert_eq!(page.items.len(), 1);
    assert!(page.next_cursor.is_some());
}

#[tokio::test]
async fn provider_binding_rejects_stale_versions_and_cross_implementation_credentials() {
    let pool = provider_test_pool("provider-binding-concurrency").await;
    let scope = KnowledgeEngineProviderScope {
        tenant_id: 100_002,
        organization_id: 7_002,
    };
    let space_store =
        SqliteKnowledgeSpaceStore::new(pool.clone(), scope.tenant_id, scope.organization_id);
    let space = space_store
        .create_space(CreateKnowledgeSpaceRecord {
            name: "Provider Concurrency Space".to_string(),
            description: None,
            okf_bundle_initialized: false,
            knowledge_mode:
                sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode::External,
        })
        .await
        .expect("create external space");
    let store = SqlxKnowledgeEngineProviderBindingStore::new(pool);
    let credential = store
        .create_credential_reference(
            scope,
            "tenant-admin",
            CreateKnowledgeEngineProviderCredentialReferenceRequest {
                implementation_id: "engine.knowledge.external.dify".to_string(),
                display_name: "Dify credential".to_string(),
                reference_locator: "secret://knowledgebase/dify/concurrency".to_string(),
            },
        )
        .await
        .expect("create credential");

    let mismatch = store
        .create_binding(
            scope,
            "tenant-admin",
            CreateKnowledgeEngineProviderBindingRequest {
                space_id: space.id,
                implementation_id: "engine.knowledge.external.ragflow".to_string(),
                remote_resource_type: "dataset".to_string(),
                remote_resource_id: "dataset-ragflow".to_string(),
                credential_reference_id: Some(credential.id),
            },
        )
        .await;
    assert!(matches!(
        mismatch,
        Err(KnowledgeEngineProviderBindingStoreError::CredentialUnavailable(id)) if id == credential.id
    ));

    let binding = store
        .create_binding(
            scope,
            "tenant-admin",
            CreateKnowledgeEngineProviderBindingRequest {
                space_id: space.id,
                implementation_id: "engine.knowledge.external.dify".to_string(),
                remote_resource_type: "dataset".to_string(),
                remote_resource_id: "dataset-dify".to_string(),
                credential_reference_id: Some(credential.id),
            },
        )
        .await
        .expect("create binding");
    let updated = store
        .update_draft_binding(
            scope,
            binding.id,
            "tenant-admin",
            UpdateKnowledgeEngineProviderBindingRequest {
                remote_resource_type: None,
                remote_resource_id: Some("dataset-dify-v2".to_string()),
                credential_reference_id: None,
                clear_credential_reference: false,
                expected_version: binding.version,
            },
        )
        .await
        .expect("update binding");
    assert_eq!(updated.remote_resource_id, "dataset-dify-v2");

    let stale = store
        .update_draft_binding(
            scope,
            binding.id,
            "tenant-admin",
            UpdateKnowledgeEngineProviderBindingRequest {
                remote_resource_type: None,
                remote_resource_id: Some("stale-write".to_string()),
                credential_reference_id: None,
                clear_credential_reference: false,
                expected_version: binding.version,
            },
        )
        .await;
    assert!(matches!(
        stale,
        Err(KnowledgeEngineProviderBindingStoreError::Conflict(_))
    ));
}

async fn create_tested_binding(
    store: &SqlxKnowledgeEngineProviderBindingStore,
    scope: KnowledgeEngineProviderScope,
    space_id: u64,
    implementation_id: &str,
    remote_resource_type: &str,
    remote_resource_id: &str,
    credential_reference_id: Option<u64>,
) -> sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineProviderBinding {
    let binding = store
        .create_binding(
            scope,
            "tenant-admin",
            CreateKnowledgeEngineProviderBindingRequest {
                space_id,
                implementation_id: implementation_id.to_string(),
                remote_resource_type: remote_resource_type.to_string(),
                remote_resource_id: remote_resource_id.to_string(),
                credential_reference_id,
            },
        )
        .await
        .expect("create binding");
    let testing = store
        .begin_binding_test(scope, binding.id, "tenant-admin", binding.version)
        .await
        .expect("begin test");
    store
        .record_binding_test_result(
            scope,
            binding.id,
            RecordKnowledgeEngineProviderTestResult {
                expected_version: testing.version,
                capabilities: vec![
                    KnowledgeEngineCapability::Health,
                    KnowledgeEngineCapability::Search,
                    KnowledgeEngineCapability::ReadDocument,
                ],
                error_category: None,
                updated_by: "tenant-admin".to_string(),
            },
        )
        .await
        .expect("record successful test")
}

async fn provider_test_pool(test_name: &str) -> sqlx::AnyPool {
    let database_url = sqlite_test_database_url(test_name);
    connect_knowledgebase_and_install_schema(&database_url)
        .await
        .expect("install provider binding schema")
}

fn sqlite_test_database_url(test_name: &str) -> String {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("repository-sqlite-tests")
        .join(format!(
            "{}-{}-{}-{}",
            test_name,
            std::process::id(),
            nanos,
            sequence
        ));
    std::fs::create_dir_all(&test_root).expect("create provider binding test directory");
    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    format!("sqlite://{relative_database_path}?mode=rwc")
}
