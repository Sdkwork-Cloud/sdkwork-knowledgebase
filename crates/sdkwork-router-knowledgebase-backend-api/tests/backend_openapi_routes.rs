use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_router_knowledgebase_backend_api::{
    build_router_with_backend_api, KnowledgeBackendApi,
};
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
        success_schema_ref(&spec, "okf.candidates.list"),
        "#/components/schemas/OkfCandidateResultList"
    );
    assert!(
        spec["components"]["schemas"]["OkfCandidateResultList"].is_object(),
        "OpenAPI must define OkfCandidateResultList schema"
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
        "KnowledgeDriveImportRequest",
        &["driveStorageProviderId"],
    );
    assert_schema_properties(
        &spec,
        "KnowledgeDriveObjectRef",
        &[
            "driveSpaceId",
            "driveNodeId",
            "driveStorageProviderId",
            "logicalPath",
        ],
    );
}

#[test]
fn backend_openapi_exposes_standard_rag_admin_operations() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    for (operation_id, method, path) in [
        (
            "indexes.create",
            "post",
            "/backend/v3/api/knowledge/indexes",
        ),
        (
            "indexes.retrieve",
            "get",
            "/backend/v3/api/knowledge/indexes/{indexId}",
        ),
        (
            "indexes.rebuild",
            "post",
            "/backend/v3/api/knowledge/indexes/{indexId}/rebuild",
        ),
        (
            "retrievalProfiles.create",
            "post",
            "/backend/v3/api/knowledge/retrieval_profiles",
        ),
        (
            "retrievalProfiles.retrieve",
            "get",
            "/backend/v3/api/knowledge/retrieval_profiles/{profileId}",
        ),
        (
            "retrievalProfiles.update",
            "patch",
            "/backend/v3/api/knowledge/retrieval_profiles/{profileId}",
        ),
        (
            "retrievalTraces.list",
            "get",
            "/backend/v3/api/knowledge/retrieval_traces",
        ),
        (
            "retrievalTraces.retrieve",
            "get",
            "/backend/v3/api/knowledge/retrieval_traces/{traceId}",
        ),
        (
            "providerHealth.retrieve",
            "get",
            "/backend/v3/api/knowledge/provider_health",
        ),
    ] {
        assert_eq!(
            spec["paths"][path][method]["operationId"], operation_id,
            "missing backend RAG operation {operation_id}: {method} {path}"
        );
        assert_eq!(
            spec["paths"][path][method]["x-sdkwork-owner"],
            "sdkwork-knowledgebase"
        );
        assert_eq!(
            spec["paths"][path][method]["x-sdkwork-api-authority"],
            "sdkwork-knowledgebase-backend-api"
        );
    }

    for schema_name in [
        "KnowledgeIndex",
        "KnowledgeIndexRequest",
        "KnowledgeRetrievalProfile",
        "KnowledgeRetrievalTrace",
        "KnowledgeRetrievalTraceList",
        "KnowledgeMemoryContextFragment",
        "KnowledgeProviderHealth",
    ] {
        assert!(
            spec["components"]["schemas"][schema_name].is_object(),
            "OpenAPI must define {schema_name}"
        );
    }
}

#[test]
fn backend_openapi_keeps_memory_context_fragments_separate_from_knowledge_chunks() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    assert_schema_properties(&spec, "KnowledgeContextPackRequest", &["memoryPolicyRef"]);
    assert_schema_properties(&spec, "KnowledgeContextPack", &["memoryFragments"]);
    assert_schema_properties(
        &spec,
        "KnowledgeMemoryContextFragment",
        &["memoryId", "content", "rank", "policyRef"],
    );

    let memory_properties = spec["components"]["schemas"]["KnowledgeMemoryContextFragment"]
        ["properties"]
        .as_object()
        .expect("KnowledgeMemoryContextFragment must define properties");
    assert!(
        !memory_properties.contains_key("chunkId"),
        "Memory fragments must not masquerade as knowledge chunks"
    );
    assert_eq!(
        spec["components"]["schemas"]["KnowledgeContextPack"]["properties"]["memoryFragments"]
            ["items"]["$ref"],
        "#/components/schemas/KnowledgeMemoryContextFragment"
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
        .replace("{conceptId}", "17")
        .replace("{profileId}", "23")
        .replace("{exportId}", "29")
        .replace("{indexId}", "37")
        .replace("{traceId}", "41")
}

fn request_body(operation_id: &str) -> &'static str {
    match operation_id {
        "sources.create" => r#"{"spaceId":7,"sourceType":"api","provider":"app-api"}"#,
        "okf.compileJobs.create" => r#"{"spaceId":7,"sourceId":11}"#,
        "okf.candidates.approve" | "okf.candidates.reject" => {
            r#"{"reviewerId":1001,"note":"reviewed"}"#
        }
        "okf.concepts.publish" => r#"{"publisherId":1001,"note":"publish"}"#,
        "okf.profile.create" | "okf.profile.update" => {
            r#"{"spaceId":7,"profileVersion":"2026-06-05"}"#
        }
        "okf.bundle.index.rebuild" => r#"{"spaceId":7}"#,
        "okf.log.entries.create" => {
            r#"{"occurredAt":"2026-06-05T00:00:00Z","eventType":"publish","title":"Published","actor":"system","affectedConcepts":[],"warnings":[]}"#
        }
        "okf.bundle.export.create" => r#"{"spaceId":7,"exportType":"snapshot"}"#,
        "okf.lintRuns.create" | "okf.evalRuns.create" => r#"{"spaceId":7}"#,
        "indexes.create" => {
            r#"{"tenantId":"20001","spaceId":"7","indexKind":"hybrid","embeddingProviderId":"provider.embedding.openai","embeddingModel":"text-embedding-3-large","dimension":3072,"metric":"cosine"}"#
        }
        "indexes.rebuild" => r#"{"spaceId":7}"#,
        "retrievalProfiles.create" | "retrievalProfiles.update" => {
            r#"{"tenantId":"20001","name":"Default Hybrid","strategy":"hybrid","topK":8,"minScore":0.4,"rerankEnabled":true,"contextBudgetTokens":2048,"status":"active"}"#
        }
        _ => "",
    }
}

struct DefaultBackendApi;

impl KnowledgeBackendApi for DefaultBackendApi {}
