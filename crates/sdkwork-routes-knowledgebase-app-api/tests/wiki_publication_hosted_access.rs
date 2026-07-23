use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    response::Response,
};
use sdkwork_routes_knowledgebase_app_api::{
    paths, KnowledgeAppRequestContext, KnowledgebaseRuntime,
};
use serde_json::{json, Value};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use tower::ServiceExt;

const TENANT_ID: u64 = 1;
const OWNER_ID: u64 = 42;
const WRITER_ID: u64 = 43;
const READER_ID: u64 = 44;
const MISSING_SOURCE_FILE_UUID: &str = "11111111-1111-4111-8111-111111111999";

#[tokio::test]
async fn hosted_wiki_routes_enforce_reader_writer_and_owner_roles() {
    let runtime = test_runtime().await;
    let space_id = create_space(&runtime).await;
    grant_role(&runtime, space_id, WRITER_ID, "writer").await;
    grant_role(&runtime, space_id, READER_ID, "reader").await;

    let publication_version = prepare_publication_for_activation(&runtime, space_id).await;
    let publication_path = paths::WIKI_PUBLICATION.replace("{space_id}", &space_id.to_string());

    for actor_id in [OWNER_ID, WRITER_ID, READER_ID] {
        assert_status(
            &runtime,
            actor_id,
            Method::GET,
            publication_path.clone(),
            None,
            StatusCode::OK,
        )
        .await;
    }

    let mut wrong_organization = app_context(READER_ID, "wrong-organization");
    wrong_organization.organization_id = Some(99);
    let response = send_with_context(
        &runtime,
        wrong_organization,
        Method::GET,
        publication_path.clone(),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let activate_path =
        paths::WIKI_PUBLICATION_ACTIVATE.replace("{space_id}", &space_id.to_string());
    for actor_id in [WRITER_ID, READER_ID] {
        assert_status(
            &runtime,
            actor_id,
            Method::POST,
            activate_path.clone(),
            Some(json!({"expectedVersion": publication_version.to_string()})),
            StatusCode::FORBIDDEN,
        )
        .await;
    }

    let activated = send(
        &runtime,
        OWNER_ID,
        Method::POST,
        activate_path,
        Some(json!({"expectedVersion": publication_version.to_string()})),
    )
    .await;
    assert_eq!(activated.status(), StatusCode::OK);
    let activated = response_json(activated).await;
    assert_eq!(
        activated.pointer("/data/item/status"),
        Some(&json!("active"))
    );
    let active_version = json_u64(activated.pointer("/data/item/version").unwrap());

    let publish_path = paths::WIKI_SOURCE_FILE_PUBLISH
        .replace("{space_id}", &space_id.to_string())
        .replace("{source_file_uuid}", MISSING_SOURCE_FILE_UUID);
    assert_status(
        &runtime,
        READER_ID,
        Method::POST,
        publish_path.clone(),
        Some(json!({
            "visibility": "public",
            "expectedPublicationVersion": active_version.to_string(),
            "expectedPageVersion": "0"
        })),
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_status(
        &runtime,
        WRITER_ID,
        Method::POST,
        publish_path,
        Some(json!({
            "visibility": "public",
            "expectedPublicationVersion": active_version.to_string(),
            "expectedPageVersion": "0"
        })),
        StatusCode::NOT_FOUND,
    )
    .await;

    let pause_path = paths::WIKI_PUBLICATION_PAUSE.replace("{space_id}", &space_id.to_string());
    assert_status(
        &runtime,
        WRITER_ID,
        Method::POST,
        pause_path.clone(),
        Some(json!({"expectedVersion": active_version.to_string()})),
        StatusCode::FORBIDDEN,
    )
    .await;
    let paused = send(
        &runtime,
        OWNER_ID,
        Method::POST,
        pause_path,
        Some(json!({"expectedVersion": active_version.to_string()})),
    )
    .await;
    assert_eq!(paused.status(), StatusCode::OK);
    assert_eq!(
        response_json(paused).await.pointer("/data/item/status"),
        Some(&json!("paused"))
    );

    let audits: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT event_type, actor_id, request_id FROM kb_audit_event WHERE tenant_id = $1 AND resource_type = 'wiki_publication' ORDER BY id ASC",
    )
    .bind(i64::try_from(TENANT_ID).unwrap())
    .fetch_all(runtime.pool())
    .await
    .expect("list Wiki publication audit events");
    assert_eq!(
        audits,
        [
            (
                "knowledge.wiki.publication.activated".to_string(),
                OWNER_ID.to_string(),
                format!("wiki-hosted-{OWNER_ID}"),
            ),
            (
                "knowledge.wiki.publication.paused".to_string(),
                OWNER_ID.to_string(),
                format!("wiki-hosted-{OWNER_ID}"),
            ),
        ]
    );
}

async fn create_space(runtime: &KnowledgebaseRuntime) -> u64 {
    let response = send(
        runtime,
        OWNER_ID,
        Method::POST,
        paths::SPACES.to_string(),
        Some(json!({
            "name": "Hosted Wiki Access",
            "description": "Wiki role enforcement integration fixture"
        })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    json_u64(body.pointer("/data/item/id").expect("created space id"))
}

async fn grant_role(runtime: &KnowledgebaseRuntime, space_id: u64, actor_id: u64, role: &str) {
    let path = paths::SPACE_MEMBERS.replace("{space_id}", &space_id.to_string());
    assert_status(
        runtime,
        OWNER_ID,
        Method::POST,
        path,
        Some(json!({
            "subjectType": "user",
            "subjectId": actor_id.to_string(),
            "role": role,
        })),
        StatusCode::OK,
    )
    .await;
}

async fn prepare_publication_for_activation(runtime: &KnowledgebaseRuntime, space_id: u64) -> u64 {
    let row: (Option<String>, Option<String>, String, i64) = sqlx::query_as(
        r#"
        SELECT source_root_node_uuid, source_scope_uuid, wiki_status, version
        FROM kb_site_publication
        WHERE tenant_id = $1 AND organization_id = 0 AND space_id = $2 AND status = 1
        "#,
    )
    .bind(i64::try_from(TENANT_ID).unwrap())
    .bind(i64::try_from(space_id).unwrap())
    .fetch_one(runtime.pool())
    .await
    .expect("Wiki publication initialized with knowledge space");
    assert!(row.0.as_deref().is_some_and(|value| !value.is_empty()));
    assert!(row.1.as_deref().is_some_and(|value| !value.is_empty()));
    assert_eq!(row.2, "READY");
    u64::try_from(row.3).unwrap()
}

async fn assert_status(
    runtime: &KnowledgebaseRuntime,
    actor_id: u64,
    method: Method,
    uri: String,
    body: Option<Value>,
    expected: StatusCode,
) {
    let response = send(runtime, actor_id, method, uri, body).await;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    assert_eq!(
        status,
        expected,
        "unexpected response: {}",
        String::from_utf8_lossy(&bytes)
    );
}

async fn send(
    runtime: &KnowledgebaseRuntime,
    actor_id: u64,
    method: Method,
    uri: String,
    body: Option<Value>,
) -> Response {
    send_with_context(
        runtime,
        app_context(actor_id, &format!("actor-{actor_id}")),
        method,
        uri,
        body,
    )
    .await
}

async fn send_with_context(
    runtime: &KnowledgebaseRuntime,
    context: KnowledgeAppRequestContext,
    method: Method,
    uri: String,
    body: Option<Value>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .extension(context);
    let body = if let Some(body) = body {
        request = request.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&body).unwrap())
    } else {
        Body::empty()
    };
    runtime
        .build_full_app_router()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap()
}

fn app_context(actor_id: u64, trace_suffix: &str) -> KnowledgeAppRequestContext {
    KnowledgeAppRequestContext {
        tenant_id: TENANT_ID,
        actor_id: Some(actor_id),
        organization_id: None,
        session_id: Some(format!("session-{actor_id}")),
        request_id: format!("wiki-hosted-{actor_id}"),
        trace_id: Some(format!("trace-wiki-hosted-{trace_suffix}")),
        idempotency_key: Some(format!("wiki-hosted-command-{trace_suffix}")),
    }
}

fn json_u64(value: &Value) -> u64 {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .expect("u64 JSON value")
}

async fn response_json(response: Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&body).expect("parse response body")
}

async fn test_runtime() -> KnowledgebaseRuntime {
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let work_dir = std::env::current_dir().expect("current directory");
    let test_root = work_dir
        .join("target")
        .join("wiki-publication-hosted-access-tests")
        .join(format!("{}-{nanos}-{sequence}", std::process::id()));
    std::fs::create_dir_all(&test_root).expect("create integration test directory");
    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create Drive storage root");
    std::env::set_var(
        "SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT",
        drive_root.to_string_lossy().as_ref(),
    );

    let database_path = test_root.join("knowledgebase.db");
    let relative_database_path = database_path
        .strip_prefix(&work_dir)
        .unwrap_or(&database_path)
        .display()
        .to_string()
        .replace('\\', "/");
    KnowledgebaseRuntime::connect(
        &format!("sqlite://{relative_database_path}?mode=rwc"),
        TENANT_ID,
    )
    .await
    .expect("initialize Knowledgebase runtime")
}
