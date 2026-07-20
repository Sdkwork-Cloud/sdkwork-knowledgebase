use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_knowledgebase_and_install_schema,
    SqlxKnowledgeEngineProviderBindingReadinessStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_readiness_store::{
    KnowledgeEngineProviderBindingReadinessStore,
    KnowledgeEngineProviderBindingReadinessStoreError,
    ListKnowledgeEngineProviderBindingReadinessGapsRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::KnowledgeEngineProviderScope;
use sqlx::AnyPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const TENANT_ID: i64 = 100_001;
const ORGANIZATION_ID: i64 = 7_001;
const NOW: &str = "2026-07-20T00:00:00Z";

#[tokio::test]
async fn readiness_report_is_scoped_bounded_and_never_infers_from_sources() {
    let pool = readiness_test_pool("provider-binding-readiness-scope").await;

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 900, "external", 1).await;
    insert_source_with_provider(&pool, TENANT_ID, 900, "engine.knowledge.external.dify").await;

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 800, "external", 1).await;
    for (index, lifecycle_state) in ["draft", "testing", "failed", "disabled"]
        .into_iter()
        .enumerate()
    {
        insert_binding(
            &pool,
            TENANT_ID,
            ORGANIZATION_ID,
            8_000 + index as i64,
            800,
            lifecycle_state,
            1,
        )
        .await;
    }

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 700, "external", 1).await;
    insert_binding(&pool, TENANT_ID, ORGANIZATION_ID, 7_000, 700, "degraded", 1).await;

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 600, "external", 1).await;
    insert_binding(&pool, TENANT_ID, ORGANIZATION_ID, 6_000, 600, "active", 1).await;

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 500, "rag", 1).await;
    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 400, "external", 0).await;

    insert_space(&pool, TENANT_ID, ORGANIZATION_ID, 300, "external", 1).await;
    insert_binding(&pool, TENANT_ID, ORGANIZATION_ID, 3_000, 300, "active", 0).await;

    insert_space(&pool, TENANT_ID + 1, ORGANIZATION_ID, 1_100, "external", 1).await;
    insert_space(&pool, TENANT_ID, ORGANIZATION_ID + 1, 1_000, "external", 1).await;

    let scope = KnowledgeEngineProviderScope {
        tenant_id: TENANT_ID as u64,
        organization_id: ORGANIZATION_ID as u64,
    };
    let store = SqlxKnowledgeEngineProviderBindingReadinessStore::new(pool);
    let first = store
        .list_spaces_missing_active_binding(
            scope,
            ListKnowledgeEngineProviderBindingReadinessGapsRequest {
                cursor: None,
                page_size: Some(2),
            },
        )
        .await
        .expect("first readiness page");

    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.space_id)
            .collect::<Vec<_>>(),
        vec![900, 800]
    );
    assert_eq!(first.items[0].non_active_binding_count, 0);
    assert_eq!(first.items[1].non_active_binding_count, 4);
    let first_json = serde_json::to_string(&first).expect("serialize readiness page");
    assert!(!first_json.contains("engine.knowledge.external.dify"));
    assert!(!first_json.contains("remoteResource"));
    assert!(!first_json.contains("credential"));

    let cursor = first.next_cursor.expect("bounded first page cursor");
    assert!(cursor.parse::<u64>().is_err(), "cursor must be opaque");
    let second = store
        .list_spaces_missing_active_binding(
            scope,
            ListKnowledgeEngineProviderBindingReadinessGapsRequest {
                cursor: Some(cursor.clone()),
                page_size: Some(2),
            },
        )
        .await
        .expect("second readiness page");
    assert_eq!(
        second
            .items
            .iter()
            .map(|item| (item.space_id, item.non_active_binding_count))
            .collect::<Vec<_>>(),
        vec![(700, 1), (300, 0)]
    );
    assert!(second.next_cursor.is_none());

    let wrong_scope = KnowledgeEngineProviderScope {
        tenant_id: scope.tenant_id,
        organization_id: scope.organization_id + 1,
    };
    assert!(matches!(
        store
            .list_spaces_missing_active_binding(
                wrong_scope,
                ListKnowledgeEngineProviderBindingReadinessGapsRequest {
                    cursor: Some(cursor),
                    page_size: Some(2),
                },
            )
            .await,
        Err(KnowledgeEngineProviderBindingReadinessStoreError::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn readiness_report_rejects_invalid_page_and_cursor_inputs() {
    let pool = readiness_test_pool("provider-binding-readiness-input").await;
    let store = SqlxKnowledgeEngineProviderBindingReadinessStore::new(pool);
    let scope = KnowledgeEngineProviderScope {
        tenant_id: TENANT_ID as u64,
        organization_id: ORGANIZATION_ID as u64,
    };

    for request in [
        ListKnowledgeEngineProviderBindingReadinessGapsRequest {
            cursor: None,
            page_size: Some(0),
        },
        ListKnowledgeEngineProviderBindingReadinessGapsRequest {
            cursor: None,
            page_size: Some(201),
        },
        ListKnowledgeEngineProviderBindingReadinessGapsRequest {
            cursor: Some("800".to_string()),
            page_size: Some(20),
        },
    ] {
        assert!(matches!(
            store
                .list_spaces_missing_active_binding(scope, request)
                .await,
            Err(KnowledgeEngineProviderBindingReadinessStoreError::InvalidRequest(_))
        ));
    }
}

async fn insert_space(
    pool: &AnyPool,
    tenant_id: i64,
    organization_id: i64,
    id: i64,
    knowledge_mode: &str,
    status: i32,
) {
    sqlx::query(
        r#"
        INSERT INTO kb_space (
            id, uuid, tenant_id, organization_id, name, status,
            okf_bundle_initialized, okf_log_sequence_counter, knowledge_mode,
            created_at, updated_at, version
        ) VALUES ($1, $2, $3, $4, $5, $6, 0, 0, $7, $8, $8, 0)
        "#,
    )
    .bind(id)
    .bind(format!("space-{tenant_id}-{organization_id}-{id}"))
    .bind(tenant_id)
    .bind(organization_id)
    .bind(format!("External space {id}"))
    .bind(status)
    .bind(knowledge_mode)
    .bind(NOW)
    .execute(pool)
    .await
    .expect("insert space fixture");
}

async fn insert_binding(
    pool: &AnyPool,
    tenant_id: i64,
    organization_id: i64,
    id: i64,
    space_id: i64,
    lifecycle_state: &str,
    status: i32,
) {
    sqlx::query(
        r#"
        INSERT INTO kb_provider_binding (
            id, uuid, tenant_id, organization_id, space_id, implementation_id,
            remote_resource_type, remote_resource_id, lifecycle_state,
            capability_snapshot, capability_snapshot_version, created_by, updated_by,
            status, created_at, updated_at, version
        ) VALUES (
            $1, $2, $3, $4, $5, 'engine.knowledge.external.dify',
            'dataset', $6, $7, '[]', 0, 'test', 'test', $8, $9, $9, 0
        )
        "#,
    )
    .bind(id)
    .bind(format!("binding-{tenant_id}-{organization_id}-{id}"))
    .bind(tenant_id)
    .bind(organization_id)
    .bind(space_id)
    .bind(format!("remote-{id}"))
    .bind(lifecycle_state)
    .bind(status)
    .bind(NOW)
    .execute(pool)
    .await
    .expect("insert Provider Binding fixture");
}

async fn insert_source_with_provider(
    pool: &AnyPool,
    tenant_id: i64,
    space_id: i64,
    provider: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO kb_source (
            id, uuid, tenant_id, space_id, source_type, provider, status,
            created_at, updated_at, version
        ) VALUES ($1, $2, $3, $4, 'connector', $5, 1, $6, $6, 0)
        "#,
    )
    .bind(90_000 + space_id)
    .bind(format!("source-{tenant_id}-{space_id}"))
    .bind(tenant_id)
    .bind(space_id)
    .bind(provider)
    .bind(NOW)
    .execute(pool)
    .await
    .expect("insert historic source fixture");
}

async fn readiness_test_pool(test_name: &str) -> AnyPool {
    let database_url = sqlite_test_database_url(test_name);
    connect_knowledgebase_and_install_schema(&database_url)
        .await
        .expect("install Provider Binding readiness schema")
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
            "{test_name}-{}-{nanos}-{sequence}",
            std::process::id()
        ));
    std::fs::create_dir_all(&test_root).expect("create readiness test directory");
    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    format!("sqlite://{relative_database_path}?mode=rwc")
}
