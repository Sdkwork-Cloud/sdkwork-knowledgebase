use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_postgres_and_install_schema, is_postgres_database_url, knowledgebase_health_check,
    KnowledgeAuditEventRecord, SqliteKnowledgeAuditEventStore, SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use std::time::{SystemTime, UNIX_EPOCH};

fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_KNOWLEDGEBASE_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|url| is_postgres_database_url(url))
}

#[tokio::test]
async fn postgres_repository_health_check_when_database_url_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping postgres integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };

    let pool = connect_postgres_and_install_schema(&database_url)
        .await
        .expect("connect postgres knowledgebase schema");
    knowledgebase_health_check(&pool)
        .await
        .expect("postgres health check");
}

#[tokio::test]
async fn postgres_audit_event_table_accepts_append_when_database_url_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping postgres audit integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };

    let pool = connect_postgres_and_install_schema(&database_url)
        .await
        .expect("connect postgres knowledgebase schema");
    let store = SqliteKnowledgeAuditEventStore::new(pool.clone(), 100_001);
    let request_id = format!(
        "req-postgres-audit-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    store
        .append_event(KnowledgeAuditEventRecord {
            id: None,
            uuid: None,
            event_type: "knowledge.backend.admin_operation".to_string(),
            actor_type: "user".to_string(),
            actor_id: "99".to_string(),
            resource_type: "backend_operation".to_string(),
            resource_id: None,
            result: "success".to_string(),
            request_id: Some(request_id.clone()),
            trace_id: None,
            payload: Some(serde_json::json!({"probe": "postgres-audit"})),
            created_at: None,
        })
        .await
        .expect("append audit event");

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kb_audit_event WHERE tenant_id = 100001 AND request_id = $1",
    )
    .bind(&request_id)
    .fetch_one(&pool)
    .await
    .expect("count audit rows");
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn postgres_web_audit_event_table_accepts_append_when_database_url_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping postgres web audit integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };

    let pool = connect_postgres_and_install_schema(&database_url)
        .await
        .expect("connect postgres knowledgebase schema");

    let request_id = format!(
        "req-web-audit-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );

    sqlx::query(
        "INSERT INTO web_audit_event \
         (request_id, tenant_id, user_id, api_surface, path, method, operation_id, status_code, duration_ms, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&request_id)
    .bind("100001")
    .bind("99")
    .bind("App")
    .bind("/app/v3/api/knowledge/spaces")
    .bind("GET")
    .bind(Option::<String>::None)
    .bind(200_i32)
    .bind(12_i32)
    .bind(1_700_000_000_i64)
    .execute(&pool)
    .await
    .expect("insert web audit row");

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM web_audit_event WHERE request_id = $1")
            .bind(&request_id)
            .fetch_one(&pool)
            .await
            .expect("count web audit rows");
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn postgres_agent_profile_create_when_database_url_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!(
            "skipping postgres agent profile integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL or DATABASE_URL to a postgres URL"
        );
        return;
    };

    use sdkwork_intelligence_knowledgebase_repository_sqlx::SqliteKnowledgeAgentProfileStore;
    use sdkwork_intelligence_knowledgebase_service::ports::knowledge_agent_profile_store::KnowledgeAgentProfileStore;
    use sdkwork_knowledgebase_contract::rag::{KnowledgeAgentProfileRequest, KnowledgeAgentStatus};

    let pool = connect_postgres_and_install_schema(&database_url)
        .await
        .expect("connect postgres knowledgebase schema");
    let store = SqliteKnowledgeAgentProfileStore::new(pool, 100_001);
    let created = store
        .create_profile(KnowledgeAgentProfileRequest {
            tenant_id: 100_001,
            name: format!(
                "postgres-agent-profile-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ),
            description: Some("postgres agent profile integration".to_string()),
            system_instruction: "Answer with citations.".to_string(),
            model_provider_id: "provider.model.knowledgebase-contract".to_string(),
            model_id: "contract".to_string(),
            model_parameters: Some(r#"{"temperature":0.7}"#.to_string()),
            retrieval_profile_id: None,
            citation_policy: None,
            memory_policy_ref: None,
            tool_policy_ref: None,
            answer_policy: None,
            status: KnowledgeAgentStatus::Active,
            knowledge_mode: Default::default(),
            agent_implementation_id:
                sdkwork_knowledgebase_contract::default_agent_implementation_id(),
        })
        .await
        .expect("create agent profile on postgres");
    assert!(created.profile_id > 0);
}

#[tokio::test]
async fn postgres_create_space_when_database_url_configured() {
    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping postgres create space integration test: set SDKWORK_KNOWLEDGEBASE_DATABASE_URL or DATABASE_URL to a postgres URL");
        return;
    };

    let pool = connect_postgres_and_install_schema(&database_url)
        .await
        .expect("connect postgres knowledgebase schema");
    let store = SqliteKnowledgeSpaceStore::new(pool, 100_001, 0);
    let created = store
        .create_space(CreateKnowledgeSpaceRecord {
            name: "postgres integration space".to_string(),
            description: Some("created by postgres_optional_integration".to_string()),
            okf_bundle_initialized: false,
            knowledge_mode: KnowledgeAgentKnowledgeMode::OkfBundle,
        })
        .await
        .expect("create knowledge space on postgres");
    assert!(created.id > 0);
}
