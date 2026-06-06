use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_knowledgebase_backend_api::{build_router_with_backend_api, KnowledgeBackendApi};
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn backend_router_mounts_every_backend_openapi_operation_path() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();
    let app = build_router_with_backend_api(DefaultBackendApi);

    let paths = spec["paths"].as_object().unwrap();
    for (template_path, methods) in paths {
        for (method_name, operation) in methods.as_object().unwrap() {
            let operation_id = operation["operationId"].as_str().unwrap();
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method_from_openapi(method_name))
                        .uri(concrete_uri(template_path))
                        .header("content-type", "application/json")
                        .body(Body::from(request_body(operation_id)))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "{operation_id} route from OpenAPI is not mounted: {method_name} {template_path}",
            );
        }
    }
}

#[test]
fn backend_openapi_uses_collection_schema_for_candidate_list_operation() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    assert_eq!(
        success_schema_ref(&spec, "wiki.candidates.list"),
        "#/components/schemas/WikiCandidateResultList"
    );
    assert!(
        spec["components"]["schemas"]["WikiCandidateResultList"].is_object(),
        "OpenAPI must define WikiCandidateResultList schema"
    );
}

#[test]
fn backend_openapi_exposes_drive_bound_contract_fields() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    assert_schema_properties(&spec, "KnowledgeSpace", &["driveSpaceId"]);
    assert_schema_properties(&spec, "KnowledgeDocument", &["originalFileDriveNodeId"]);
    assert_schema_properties(
        &spec,
        "KnowledgeDriveObjectRef",
        &["driveSpaceId", "driveNodeId", "logicalPath"],
    );
}

fn assert_schema_properties(spec: &Value, schema_name: &str, expected: &[&str]) {
    let properties = spec["components"]["schemas"][schema_name]["properties"]
        .as_object()
        .unwrap_or_else(|| panic!("OpenAPI schema {schema_name} must define properties"));

    for property in expected {
        assert!(
            properties.contains_key(*property),
            "OpenAPI schema {schema_name} must define property {property}"
        );
    }
}

fn success_schema_ref<'a>(spec: &'a Value, operation_id: &str) -> &'a str {
    for methods in spec["paths"].as_object().unwrap().values() {
        for operation in methods.as_object().unwrap().values() {
            if operation["operationId"] == operation_id {
                return operation["responses"]["200"]["content"]["application/json"]["schema"]
                    ["$ref"]
                    .as_str()
                    .unwrap();
            }
        }
    }
    panic!("missing operationId: {operation_id}");
}

fn method_from_openapi(method_name: &str) -> Method {
    match method_name {
        "delete" => Method::DELETE,
        "get" => Method::GET,
        "patch" => Method::PATCH,
        "post" => Method::POST,
        value => panic!("unsupported OpenAPI method: {value}"),
    }
}

fn concrete_uri(template_path: &str) -> String {
    template_path
        .replace("{candidateId}", "31")
        .replace("{pageId}", "17")
        .replace("{profileId}", "23")
        .replace("{exportId}", "29")
}

fn request_body(operation_id: &str) -> &'static str {
    match operation_id {
        "sources.create" => r#"{"spaceId":7,"sourceType":"api","provider":"app-api"}"#,
        "wiki.compileJobs.create" => r#"{"spaceId":7,"sourceId":11}"#,
        "wiki.candidates.approve" | "wiki.candidates.reject" => {
            r#"{"reviewerId":1001,"note":"reviewed"}"#
        }
        "wiki.pages.publish" => r#"{"publisherId":1001,"note":"publish"}"#,
        "wiki.schema.profiles.create" | "wiki.schema.profiles.update" => {
            r#"{"spaceId":7,"profileVersion":"2026-06-05"}"#
        }
        "wiki.index.rebuild" => r#"{"spaceId":7}"#,
        "wiki.log.entries.create" => {
            r#"{"occurredAt":"2026-06-05T00:00:00Z","eventType":"publish","title":"Published","actor":"system","affectedPages":[],"warnings":[]}"#
        }
        "wiki.exports.create" => r#"{"spaceId":7,"exportType":"snapshot"}"#,
        "wiki.lintRuns.create" | "wiki.evalRuns.create" => r#"{"spaceId":7}"#,
        _ => "",
    }
}

struct DefaultBackendApi;

impl KnowledgeBackendApi for DefaultBackendApi {}
