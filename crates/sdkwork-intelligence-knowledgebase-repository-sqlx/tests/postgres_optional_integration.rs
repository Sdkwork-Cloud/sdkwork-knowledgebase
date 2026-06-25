use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_postgres_and_install_schema, is_postgres_database_url, knowledgebase_health_check,
    KnowledgeAuditEventRecord, SqliteKnowledgeAuditEventStore,
};
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
    store
        .append_event(KnowledgeAuditEventRecord {
            event_type: "knowledge.backend.admin_operation".to_string(),
            actor_type: "user".to_string(),
            actor_id: "99".to_string(),
            resource_type: "backend_operation".to_string(),
            resource_id: None,
            result: "success".to_string(),
            request_id: Some("req-postgres-1".to_string()),
            trace_id: None,
            payload: None,
        })
        .await
        .expect("append audit event");

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM kb_audit_event WHERE tenant_id = 100001 AND event_type = 'knowledge.backend.admin_operation'",
    )
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

    sqlx::query(
        "INSERT INTO web_audit_event \
         (request_id, tenant_id, user_id, api_surface, path, method, operation_id, status_code, duration_ms, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind("req-web-audit-1")
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
        sqlx::query_as("SELECT COUNT(*) FROM web_audit_event WHERE request_id = 'req-web-audit-1'")
            .fetch_one(&pool)
            .await
            .expect("count web audit rows");
    assert_eq!(count.0, 1);
}
