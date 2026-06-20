use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_router_knowledgebase_app_api::{dev_auth, KnowledgebaseRuntime};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

static EXTERNAL_ADAPTER_ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn hosted_app_router_lists_documents() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/app/v3/api/knowledge/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted app api must not return operation_not_implemented for documents.list"
    );

    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn hosted_backend_router_serves_provider_health() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/knowledge/provider_health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted backend must not return operation_not_implemented for providerHealth.retrieve"
    );

    let body = response_body_json(response).await;
    assert_eq!(body["status"], "ok");
    let provider_id = body["providerId"].as_str().expect("providerId");
    assert!(
        provider_id.contains("engine.knowledge.okf.native"),
        "providerId must include okf native engine: {provider_id}"
    );
    assert!(
        provider_id.contains("engine.knowledge.rag.native"),
        "providerId must include rag native engine: {provider_id}"
    );
    assert!(
        provider_id.contains("engine.knowledge.external.dify"),
        "providerId must include dify catalog engine: {provider_id}"
    );
    assert!(
        provider_id.contains("engine.knowledge.external.ragflow"),
        "providerId must include ragflow catalog engine: {provider_id}"
    );
}

#[tokio::test]
async fn hosted_backend_router_lists_sources() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/backend/v3/api/knowledge/sources")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn hosted_app_router_upserts_okf_concept() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"OKF Test Space","description":"Hosted upsert integration"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space = response_body_json(space_response).await;
    let space_id = space["id"].as_u64().expect("created space id");

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/app/v3/api/knowledge/okf/concepts/upsert")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r##"{{"spaceId":{space_id},"conceptId":"tables/users","markdown":"---\ntype: Entity\ntitle: Users\n---\n# Users\n","actor":"author","publish":false}}"##
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted app api must not return operation_not_implemented for okf.concepts.upsert"
    );

    let body = response_body_json(response).await;
    assert_eq!(body["conceptId"], "tables/users");
    assert_eq!(body["title"], "Users");
}

#[tokio::test]
async fn hosted_app_lists_okf_concepts_for_space() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let space_id = create_space(&app, "Concept List Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "tables/users",
        "---\ntype: Entity\ntitle: Users\n---\n# Users\n\nUser table.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/okf/concepts?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_json(response).await;
    assert!(body["items"]
        .as_array()
        .expect("concept items")
        .iter()
        .any(|item| item["conceptId"] == "tables/users"));
}

#[tokio::test]
async fn hosted_backend_lists_okf_candidates_for_space() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Candidate Space").await;
    let response = backend
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/knowledge/okf/candidates?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(response.status(), StatusCode::NOT_IMPLEMENTED);
    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

#[tokio::test]
async fn hosted_backend_registers_okf_profile_and_rebuilds_index() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Profile Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "entities/widget",
        "---\ntype: Entity\ntitle: Widget\n---\n# Widget\n\nWidget entity.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let profile_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/profile")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"profileVersion":"2026-06-19"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_response.status(), StatusCode::CREATED);

    let rebuild_response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/index/rebuild")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"spaceId":{space_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rebuild_response.status(), StatusCode::OK);
    let rebuild_body = response_body_json(rebuild_response).await;
    assert!(rebuild_body["markdown"]
        .as_str()
        .unwrap_or("")
        .contains("Widget"));
}

#[tokio::test]
async fn hosted_backend_imports_staged_export_bundle() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Backend Import Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "entities/gadget",
        "---\ntype: Entity\ntitle: Gadget\n---\n# Gadget\n\nA gadget.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let export_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/exports")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"exportType":"okf_strict","stageForImport":true,"importId":"backend-import"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_response.status(), StatusCode::CREATED);

    let import_response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/imports")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"importType":"okf_strict","importId":"backend-import"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(import_response.status(), StatusCode::CREATED);
    let import_body = response_body_json(import_response).await;
    assert!(import_body["importedConceptCount"].as_u64().unwrap_or(0) >= 1);
}

#[tokio::test]
async fn hosted_app_export_stages_bundle_for_import_roundtrip() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let source_space = create_space(&app, "Export Source").await;
    publish_okf_concept(
        &app,
        source_space,
        "entities/widget",
        "---\ntype: Entity\ntitle: Widget\n---\n# Widget\n\nA durable widget.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let export_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/okf/exports")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{source_space},"exportType":"okf_strict","stageForImport":true,"importId":"api-roundtrip"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_response.status(), StatusCode::CREATED);
    let export_body = response_body_json(export_response).await;
    assert_eq!(export_body["importId"], "api-roundtrip");
    assert_eq!(
        export_body["stagedImportRoot"],
        "inbox/drive-imports/api-roundtrip"
    );

    let import_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/okf/imports")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{source_space},"importType":"okf_strict","importId":"api-roundtrip"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let import_status = import_response.status();
    let import_body = response_body_json(import_response).await;
    if import_status != StatusCode::CREATED {
        panic!("import staged bundle failed: {import_body}");
    }
    assert!(import_body["importedConceptCount"].as_u64().unwrap_or(0) >= 1);

    let concepts_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/app/v3/api/knowledge/okf/concepts?spaceId={source_space}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(concepts_response.status(), StatusCode::OK);
    let concepts_body = response_body_json(concepts_response).await;
    assert!(concepts_body["items"]
        .as_array()
        .expect("concept list")
        .iter()
        .any(|item| item["conceptId"] == "entities/widget"));
}

#[tokio::test]
async fn hosted_backend_export_stages_bundle_for_import() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let source_space = create_space(&app, "Backend Export Source").await;
    publish_okf_concept(
        &app,
        source_space,
        "entities/widget",
        "---\ntype: Entity\ntitle: Widget\n---\n# Widget\n\nA durable widget.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let export_response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/exports")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{source_space},"exportType":"okf_strict","stageForImport":true,"importId":"backend-roundtrip"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(export_response.status(), StatusCode::CREATED);
    let export_body = response_body_json(export_response).await;
    assert_eq!(export_body["importId"], "backend-roundtrip");
    assert_eq!(
        export_body["stagedImportRoot"],
        "inbox/drive-imports/backend-roundtrip"
    );
}

#[tokio::test]
async fn hosted_backend_runs_okf_lint_job_for_space() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Lint Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "entities/lintable",
        "---\ntype: Entity\ntitle: Lintable\n---\n# Lintable\n\nLint target.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/lint_runs")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"spaceId":{space_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_body_json(response).await;
    assert_eq!(body["state"], "succeeded");
}

#[tokio::test]
async fn hosted_backend_runs_okf_compile_job_for_space() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Compile Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "entities/compilable",
        "---\ntype: Entity\ntitle: Compilable\n---\n# Compilable\n\nCompile target.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/okf/compile_jobs")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"spaceId":{space_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_body_json(response).await;
    assert_eq!(body["state"], "succeeded");
}

#[tokio::test]
async fn hosted_backend_approves_okf_candidate() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Approve Space").await;
    stage_okf_concept(
        &app,
        space_id,
        "entities/approve-me",
        "---\ntype: Entity\ntitle: Approve Me\n---\n# Approve Me\n\nPending review.\n",
    )
    .await;

    let candidates = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/knowledge/okf/candidates?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(candidates.status(), StatusCode::OK);
    let candidate_items = response_body_json(candidates).await["items"]
        .as_array()
        .expect("candidate items")
        .clone();
    let candidate_id = candidate_items
        .iter()
        .find(|item| item["state"] == "candidate_ready")
        .and_then(|item| item["id"].as_u64())
        .expect("open candidate id");

    let approve_response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/backend/v3/api/knowledge/okf/candidates/{candidate_id}/approve"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"reviewerId":99,"note":"approved in test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve_response.status(), StatusCode::CREATED);
    let approve_body = response_body_json(approve_response).await;
    assert_eq!(approve_body["state"], "published");
}

#[tokio::test]
async fn hosted_backend_rejects_okf_candidate() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_id = create_space(&app, "Reject Space").await;
    stage_okf_concept(
        &app,
        space_id,
        "entities/reject-me",
        "---\ntype: Entity\ntitle: Reject Me\n---\n# Reject Me\n\nNot ready.\n",
    )
    .await;

    let candidates = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/backend/v3/api/knowledge/okf/candidates?spaceId={space_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(candidates.status(), StatusCode::OK);
    let candidate_id = response_body_json(candidates).await["items"]
        .as_array()
        .expect("candidate items")
        .iter()
        .find(|item| item["state"] == "candidate_ready")
        .and_then(|item| item["id"].as_u64())
        .expect("open candidate id");

    let reject_response = backend
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/backend/v3/api/knowledge/okf/candidates/{candidate_id}/reject"
                ))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"reviewerId":99,"note":"rejected in test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reject_response.status(), StatusCode::CREATED);
    let reject_body = response_body_json(reject_response).await;
    assert_eq!(reject_body["state"], "rejected");
}

#[tokio::test]
async fn hosted_backend_resolves_external_space_to_catalog_engine() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"External Hosted Space","description":"External knowledge mode","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"dify"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);
    let source_body = response_body_json(source_response).await;
    assert_eq!(source_body["provider"], "dify");
    assert_eq!(source_body["sourceType"], "connector");

    let implementation_id = runtime
        .resolve_knowledge_engine_implementation_id_for_space(space_id)
        .await
        .expect("resolve external engine");
    assert_eq!(implementation_id, "engine.knowledge.external.dify");
}

#[tokio::test]
async fn hosted_backend_resolves_external_space_to_ragflow_engine() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"RAGFlow Hosted Space","description":"External RAGFlow mode","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"ragflow"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);
    let source_body = response_body_json(source_response).await;
    assert_eq!(source_body["provider"], "ragflow");
    assert_eq!(source_body["sourceType"], "connector");

    let implementation_id = runtime
        .resolve_knowledge_engine_implementation_id_for_space(space_id)
        .await
        .expect("resolve external engine");
    assert_eq!(implementation_id, "engine.knowledge.external.ragflow");
}

#[tokio::test]
async fn hosted_backend_resolves_external_space_to_onyx_engine() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Onyx Hosted Space","description":"External Onyx mode","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"onyx"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);

    let implementation_id = runtime
        .resolve_knowledge_engine_implementation_id_for_space(space_id)
        .await
        .expect("resolve external engine");
    assert_eq!(implementation_id, "engine.knowledge.external.onyx");
}

#[tokio::test]
async fn hosted_external_agent_chat_rejects_unconfigured_external_adapter() {
    let _env_guard = EXTERNAL_ADAPTER_ENV_TEST_LOCK
        .lock()
        .expect("external adapter env test lock");
    clear_dify_adapter_env();
    clear_ragflow_adapter_env();
    clear_onyx_adapter_env();
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"External Agent Space","description":"External agent chat","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"dify"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);

    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/agent_profiles")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","name":"External Agent","systemInstruction":"Answer with citations.","modelProviderId":"provider.model.knowledgebase-contract","modelId":"contract","agentImplementationId":"plugin.intelligence.knowledgebase-contract","knowledgeMode":"external","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_response.status(), StatusCode::CREATED);
    let profile_body = response_body_json(profile_response).await;
    let profile_id = json_u64_field(&profile_body, "profileId").expect("created profile id");

    let binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings"
                ))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenantId":"1","profileId":"{profile_id}","spaceId":"{space_id}","priority":0,"enabled":true}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(binding_response.status(), StatusCode::CREATED);

    let chat_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/chat"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","message":"What is in the external knowledge base?"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        chat_response.status(),
        StatusCode::BAD_REQUEST,
        "unconfigured external adapter must fail agent chat before LLM invocation"
    );
    let chat_body = response_body_json(chat_response).await;
    let detail = chat_body["detail"]
        .as_str()
        .or_else(|| chat_body["title"].as_str())
        .unwrap_or("");
    assert!(
        detail.to_ascii_lowercase().contains("unsupported")
            || detail.to_ascii_lowercase().contains("adapter")
            || detail.to_ascii_lowercase().contains("catalog")
            || detail.to_ascii_lowercase().contains("dify")
            || detail.to_ascii_lowercase().contains("dataset"),
        "expected unconfigured adapter rejection detail, got: {chat_body}"
    );
}

#[tokio::test]
async fn hosted_external_agent_chat_succeeds_with_configured_dify_adapter() {
    let _env_guard = EXTERNAL_ADAPTER_ENV_TEST_LOCK
        .lock()
        .expect("external adapter env test lock");
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/datasets/ds-hosted-e2e/retrieve"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [{
                    "segment": {
                        "id": "seg-hosted",
                        "content": "hosted external knowledge snippet",
                        "document": { "name": "Hosted External Doc" }
                    },
                    "score": 0.95
                }]
            })),
        )
        .mount(&mock_server)
        .await;

    let _dify_base_url = TempEnvVar::set(
        "SDKWORK_KNOWLEDGEBASE_DIFY_BASE_URL",
        mock_server.uri().as_str(),
    );
    let _dify_api_key = TempEnvVar::set("SDKWORK_KNOWLEDGEBASE_DIFY_API_KEY", "hosted-test-key");

    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Configured External Space","description":"Configured external agent chat","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"dify","connectorMetadataJson":"{{\"datasetId\":\"ds-hosted-e2e\"}}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);

    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/agent_profiles")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","name":"Configured External Agent","systemInstruction":"Answer with citations.","modelProviderId":"provider.model.knowledgebase-contract","modelId":"contract","agentImplementationId":"plugin.intelligence.knowledgebase-contract","knowledgeMode":"external","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_response.status(), StatusCode::CREATED);
    let profile_body = response_body_json(profile_response).await;
    let profile_id = json_u64_field(&profile_body, "profileId").expect("created profile id");

    let binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings"
                ))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenantId":"1","profileId":"{profile_id}","spaceId":"{space_id}","priority":0,"enabled":true}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(binding_response.status(), StatusCode::CREATED);

    let chat_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/chat"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","message":"What is in the external knowledge base?"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        chat_response.status(),
        StatusCode::CREATED,
        "configured external engine must complete agent chat"
    );
    let chat_body = response_body_json(chat_response).await;
    assert_eq!(chat_body["mode"], "external");
    assert!(chat_body["citations"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert_eq!(chat_body["citations"][0]["title"], "Hosted External Doc");
    assert!(chat_body["answer"]
        .as_str()
        .is_some_and(|answer| answer.contains("Hosted External Doc")));
}

#[tokio::test]
async fn hosted_external_agent_chat_succeeds_with_configured_ragflow_adapter() {
    let _env_guard = EXTERNAL_ADAPTER_ENV_TEST_LOCK
        .lock()
        .expect("external adapter env test lock");
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/api/v1/retrieval"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": 0,
                "data": {
                    "chunks": [{
                        "id": "chunk-hosted",
                        "content": "hosted ragflow knowledge snippet",
                        "document_id": "doc-hosted",
                        "document_keyword": "Hosted RAGFlow Doc",
                        "similarity": 0.95
                    }]
                }
            })),
        )
        .mount(&mock_server)
        .await;

    let _ragflow_base_url = TempEnvVar::set(
        "SDKWORK_KNOWLEDGEBASE_RAGFLOW_BASE_URL",
        mock_server.uri().as_str(),
    );
    let _ragflow_api_key = TempEnvVar::set(
        "SDKWORK_KNOWLEDGEBASE_RAGFLOW_API_KEY",
        "hosted-ragflow-key",
    );

    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Configured RAGFlow Space","description":"Configured RAGFlow agent chat","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"ragflow","connectorMetadataJson":"{{\"datasetId\":\"ds-ragflow-hosted\"}}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);

    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/agent_profiles")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","name":"Configured RAGFlow Agent","systemInstruction":"Answer with citations.","modelProviderId":"provider.model.knowledgebase-contract","modelId":"contract","agentImplementationId":"plugin.intelligence.knowledgebase-contract","knowledgeMode":"external","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_response.status(), StatusCode::CREATED);
    let profile_body = response_body_json(profile_response).await;
    let profile_id = json_u64_field(&profile_body, "profileId").expect("created profile id");

    let binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings"
                ))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenantId":"1","profileId":"{profile_id}","spaceId":"{space_id}","priority":0,"enabled":true}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(binding_response.status(), StatusCode::CREATED);

    let chat_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/chat"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","message":"What is in the RAGFlow knowledge base?"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        chat_response.status(),
        StatusCode::CREATED,
        "configured RAGFlow engine must complete agent chat"
    );
    let chat_body = response_body_json(chat_response).await;
    assert_eq!(chat_body["mode"], "external");
    assert!(chat_body["citations"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert_eq!(chat_body["citations"][0]["title"], "Hosted RAGFlow Doc");
    assert!(chat_body["answer"]
        .as_str()
        .is_some_and(|answer| answer.contains("Hosted RAGFlow Doc")));
}

#[tokio::test]
async fn hosted_external_agent_chat_succeeds_with_configured_onyx_adapter() {
    let _env_guard = EXTERNAL_ADAPTER_ENV_TEST_LOCK
        .lock()
        .expect("external adapter env test lock");
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/search"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "title": "Hosted Onyx Doc",
                    "url": "https://example.com/onyx-hosted",
                    "content": "hosted onyx knowledge snippet",
                    "source_type": "web"
                }]
            })),
        )
        .mount(&mock_server)
        .await;

    let _onyx_base_url = TempEnvVar::set(
        "SDKWORK_KNOWLEDGEBASE_ONYX_BASE_URL",
        mock_server.uri().as_str(),
    );
    let _onyx_api_key = TempEnvVar::set("SDKWORK_KNOWLEDGEBASE_ONYX_API_KEY", "hosted-onyx-key");

    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));
    let backend = dev_auth::with_dev_backend_auth(runtime.build_backend_router(), 1, Some(99));

    let space_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Configured Onyx Space","description":"Configured Onyx agent chat","knowledgeMode":"external"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(space_response.status(), StatusCode::CREATED);
    let space_id = response_body_json(space_response).await["id"]
        .as_u64()
        .expect("external space id");

    let source_response = backend
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/backend/v3/api/knowledge/sources")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"spaceId":{space_id},"sourceType":"connector","provider":"onyx"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(source_response.status(), StatusCode::CREATED);

    let profile_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/agent_profiles")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","name":"Configured Onyx Agent","systemInstruction":"Answer with citations.","modelProviderId":"provider.model.knowledgebase-contract","modelId":"contract","agentImplementationId":"plugin.intelligence.knowledgebase-contract","knowledgeMode":"external","status":"active"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(profile_response.status(), StatusCode::CREATED);
    let profile_body = response_body_json(profile_response).await;
    let profile_id = json_u64_field(&profile_body, "profileId").expect("created profile id");

    let binding_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings"
                ))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenantId":"1","profileId":"{profile_id}","spaceId":"{space_id}","priority":0,"enabled":true}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(binding_response.status(), StatusCode::CREATED);

    let chat_response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/app/v3/api/knowledge/agent_profiles/{profile_id}/chat"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"1","message":"What is in the Onyx knowledge base?"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        chat_response.status(),
        StatusCode::CREATED,
        "configured Onyx engine must complete agent chat"
    );
    let chat_body = response_body_json(chat_response).await;
    assert_eq!(chat_body["mode"], "external");
    assert!(chat_body["citations"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert_eq!(chat_body["citations"][0]["title"], "Hosted Onyx Doc");
    assert!(chat_body["answer"]
        .as_str()
        .is_some_and(|answer| answer.contains("Hosted Onyx Doc")));
}

#[tokio::test]
async fn hosted_app_runs_okf_lint_job_for_space() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_app_auth(runtime.build_full_app_router(), 1, Some(42));

    let space_id = create_space(&app, "App Lint Space").await;
    publish_okf_concept(
        &app,
        space_id,
        "entities/app-lint",
        "---\ntype: Entity\ntitle: App Lint\n---\n# App Lint\n\nApp lint target.\n\n# Citations\n\n[1] [Src](https://example.com)\n",
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/okf/lint_runs")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"spaceId":{space_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_body_json(response).await;
    assert_eq!(body["state"], "succeeded");
}

async fn create_space(app: &axum::Router, name: &str) -> u64 {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/app/v3/api/knowledge/spaces")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"name":"{name}","description":"Hosted OKF integration"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    response_body_json(response).await["id"]
        .as_u64()
        .expect("created space id")
}

async fn publish_okf_concept(app: &axum::Router, space_id: u64, concept_id: &str, markdown: &str) {
    upsert_okf_concept(app, space_id, concept_id, markdown, true).await;
}

async fn stage_okf_concept(app: &axum::Router, space_id: u64, concept_id: &str, markdown: &str) {
    upsert_okf_concept(app, space_id, concept_id, markdown, false).await;
}

async fn upsert_okf_concept(
    app: &axum::Router,
    space_id: u64,
    concept_id: &str,
    markdown: &str,
    publish: bool,
) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/app/v3/api/knowledge/okf/concepts/upsert")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r##"{{"spaceId":{space_id},"conceptId":"{concept_id}","markdown":{markdown_json},"actor":"author","publish":{publish}}}"##,
                    markdown_json = serde_json::to_string(markdown).expect("serialize markdown")
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    if response.status() != StatusCode::OK {
        let body = response_body_json(response).await;
        panic!("upsert okf concept failed: {body}");
    }
}

#[tokio::test]
async fn hosted_open_router_lists_documents() {
    let runtime = test_runtime().await;
    let app = dev_auth::with_dev_open_auth(runtime.build_open_api_router(), 1, Some(42));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/knowledge/v3/api/documents")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_ne!(
        response.status(),
        StatusCode::NOT_IMPLEMENTED,
        "hosted open api must not return operation_not_implemented for documents.list"
    );

    let body = response_body_json(response).await;
    assert!(body["items"].is_array());
}

struct TempEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl TempEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for TempEnvVar {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn clear_dify_adapter_env() {
    for key in [
        "SDKWORK_KNOWLEDGEBASE_DIFY_BASE_URL",
        "SDKWORK_KNOWLEDGEBASE_DIFY_API_KEY",
        "SDKWORK_KNOWLEDGEBASE_DIFY_DATASET_ID",
    ] {
        std::env::remove_var(key);
    }
}

fn clear_ragflow_adapter_env() {
    for key in [
        "SDKWORK_KNOWLEDGEBASE_RAGFLOW_BASE_URL",
        "SDKWORK_KNOWLEDGEBASE_RAGFLOW_API_KEY",
        "SDKWORK_KNOWLEDGEBASE_RAGFLOW_DATASET_ID",
    ] {
        std::env::remove_var(key);
    }
}

fn clear_onyx_adapter_env() {
    for key in [
        "SDKWORK_KNOWLEDGEBASE_ONYX_BASE_URL",
        "SDKWORK_KNOWLEDGEBASE_ONYX_API_KEY",
    ] {
        std::env::remove_var(key);
    }
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
        .join("hosted-runtime-tests")
        .join(format!("{}-{}-{}", std::process::id(), nanos, sequence));
    std::fs::create_dir_all(&test_root).expect("create hosted runtime test directory");

    let drive_root = test_root.join("drive-objects");
    std::fs::create_dir_all(&drive_root).expect("create hosted runtime drive storage root");
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
    let database_url = format!("sqlite://{relative_database_path}?mode=rwc");
    KnowledgebaseRuntime::connect(&database_url, 1)
        .await
        .expect("initialize hosted runtime")
}

async fn response_body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("parse response json")
}

fn json_u64_field(body: &Value, field: &str) -> Option<u64> {
    body.get(field)
        .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
}
