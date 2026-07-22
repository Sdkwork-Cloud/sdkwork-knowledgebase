use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_routes_knowledgebase_backend_api::{
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
    let app = build_router_with_backend_api(DefaultBackendApi, 1);

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

    assert_list_response_envelope(
        &spec,
        "okf.candidates.list",
        "#/components/schemas/OkfCandidateResult",
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
        ("indexes.list", "get", "/backend/v3/api/knowledge/indexes"),
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
            "providerHealth.list",
            "get",
            "/backend/v3/api/knowledge/provider_health",
        ),
        ("spaces.list", "get", "/backend/v3/api/knowledge/spaces"),
        (
            "spaces.members.list",
            "get",
            "/backend/v3/api/knowledge/spaces/{spaceId}/members",
        ),
        (
            "compliance.auditEvents.export.create",
            "post",
            "/backend/v3/api/knowledge/compliance/audit_events/export",
        ),
        (
            "compliance.auditEvents.anonymizeActor.create",
            "post",
            "/backend/v3/api/knowledge/compliance/audit_events/anonymize_actor",
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
        "KnowledgeTenantQuotaStatus",
        "ExportKnowledgeAuditEventsRequest",
        "KnowledgeAuditEventExport",
        "AnonymizeKnowledgeAuditSubjectRequest",
        "AnonymizeKnowledgeAuditSubjectResult",
    ] {
        assert!(
            spec["components"]["schemas"][schema_name].is_object(),
            "OpenAPI must define {schema_name}"
        );
    }
}

#[test]
fn backend_openapi_exposes_secret_safe_provider_management_contracts() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    for (operation_id, method, path) in [
        (
            "providerCredentialReferences.list",
            "get",
            "/backend/v3/api/knowledge/provider_credential_references",
        ),
        (
            "providerCredentialReferences.create",
            "post",
            "/backend/v3/api/knowledge/provider_credential_references",
        ),
        (
            "providerCredentialReferences.retrieve",
            "get",
            "/backend/v3/api/knowledge/provider_credential_references/{credentialReferenceId}",
        ),
        (
            "providerCredentialReferences.rotate",
            "post",
            "/backend/v3/api/knowledge/provider_credential_references/{credentialReferenceId}/rotate",
        ),
        (
            "providerCredentialReferences.revoke",
            "post",
            "/backend/v3/api/knowledge/provider_credential_references/{credentialReferenceId}/revoke",
        ),
        (
            "spaces.providerBindings.list",
            "get",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings",
        ),
        (
            "spaces.providerBindings.create",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings",
        ),
        (
            "spaces.providerBindings.retrieve",
            "get",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings/{bindingId}",
        ),
        (
            "spaces.providerBindings.update",
            "patch",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings/{bindingId}",
        ),
        (
            "spaces.providerBindings.test",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings/{bindingId}/test",
        ),
        (
            "spaces.providerBindings.activate",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings/{bindingId}/activate",
        ),
        (
            "spaces.providerBindings.disable",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_bindings/{bindingId}/disable",
        ),
        (
            "spaces.providerMigrations.list",
            "get",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_migrations",
        ),
        (
            "spaces.providerMigrations.create",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_migrations",
        ),
        (
            "spaces.providerMigrations.retrieve",
            "get",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_migrations/{migrationOperationId}",
        ),
        (
            "spaces.providerMigrations.rollback",
            "post",
            "/backend/v3/api/knowledge/spaces/{spaceId}/provider_migrations/{migrationOperationId}/rollback",
        ),
    ] {
        let operation = &spec["paths"][path][method];
        assert_eq!(operation["operationId"], operation_id);
        assert_eq!(
            operation["x-sdkwork-permission"],
            "knowledge.platform.manage"
        );
        assert_eq!(
            operation["x-sdkwork-request-context"],
            "WebRequestContext"
        );
    }

    let credential_schema =
        &spec["components"]["schemas"]["KnowledgeEngineProviderCredentialReference"];
    let credential_properties = credential_schema["properties"]
        .as_object()
        .expect("credential read model properties");
    assert!(!credential_properties.contains_key("referenceLocator"));
    assert!(!credential_properties.contains_key("referenceFingerprint"));
    assert_eq!(
        spec["components"]["schemas"]["CreateKnowledgeEngineProviderCredentialReferenceRequest"]
            ["properties"]["referenceLocator"]["writeOnly"],
        true
    );
    assert_eq!(
        spec["components"]["schemas"]["RotateKnowledgeEngineProviderCredentialReferenceRequest"]
            ["properties"]["referenceLocator"]["writeOnly"],
        true
    );
    assert_named_list_response_envelope(
        &spec,
        "providerCredentialReferences.list",
        "#/components/schemas/KnowledgeEngineProviderCredentialReferencePage",
    );
    assert_named_list_response_envelope(
        &spec,
        "spaces.providerBindings.list",
        "#/components/schemas/KnowledgeEngineProviderBindingPage",
    );
    assert_named_list_response_envelope(
        &spec,
        "spaces.providerMigrations.list",
        "#/components/schemas/KnowledgeEngineProviderMigrationOperationPage",
    );
    let migration_properties = spec["components"]["schemas"]
        ["KnowledgeEngineProviderMigrationOperation"]["properties"]
        .as_object()
        .expect("Provider migration read model properties");
    for internal_field in ["checkpoint", "claimOwner", "claimToken", "leaseExpiresAt"] {
        assert!(!migration_properties.contains_key(internal_field));
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

#[test]
fn backend_openapi_exposes_tenant_quota_on_status_schema() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
    ))
    .unwrap();

    assert_schema_properties(&spec, "KnowledgeTenantStatus", &["quota"]);
    assert_schema_properties(
        &spec,
        "KnowledgeTenantQuotaStatus",
        &[
            "maxDocuments",
            "documentCount",
            "maxConcurrentIngestJobs",
            "inflightIngestJobs",
            "maxRetrievalsPerMinute",
            "maxStorageBytes",
            "storageBytesUsed",
        ],
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

fn operation_response_schema<'a>(spec: &'a Value, operation_id: &str) -> &'a Value {
    for methods in spec["paths"].as_object().unwrap().values() {
        for operation in methods.as_object().unwrap().values() {
            if operation["operationId"] == operation_id {
                return &operation["responses"]["200"]["content"]["application/json"]["schema"];
            }
        }
    }
    panic!("missing operationId: {operation_id}");
}

fn assert_list_response_envelope(spec: &Value, operation_id: &str, item_schema_ref: &str) {
    let schema = operation_response_schema(spec, operation_id);
    let all_of = schema["allOf"]
        .as_array()
        .unwrap_or_else(|| panic!("{operation_id} must use SdkWorkApiResponse allOf envelope"));
    assert_eq!(
        all_of[0]["$ref"], "#/components/schemas/SdkWorkApiResponse",
        "{operation_id} must extend SdkWorkApiResponse"
    );

    let data = &all_of[1]["properties"]["data"];
    assert_eq!(
        data["properties"]["items"]["items"]["$ref"], item_schema_ref,
        "{operation_id} must list {item_schema_ref} in data.items"
    );
    assert_eq!(
        data["properties"]["pageInfo"]["$ref"], "#/components/schemas/PageInfo",
        "{operation_id} must expose data.pageInfo"
    );
}

fn assert_named_list_response_envelope(spec: &Value, operation_id: &str, data_schema_ref: &str) {
    let schema = operation_response_schema(spec, operation_id);
    let all_of = schema["allOf"]
        .as_array()
        .unwrap_or_else(|| panic!("{operation_id} must use SdkWorkApiResponse allOf envelope"));
    assert_eq!(
        all_of[0]["$ref"], "#/components/schemas/SdkWorkApiResponse",
        "{operation_id} must extend SdkWorkApiResponse"
    );
    assert_eq!(
        all_of[1]["properties"]["data"]["$ref"], data_schema_ref,
        "{operation_id} must expose typed list data"
    );
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
    let path = template_path
        .replace("{candidateId}", "31")
        .replace("{conceptId}", "17")
        .replace("{profileId}", "23")
        .replace("{exportId}", "29")
        .replace("{indexId}", "37")
        .replace("{traceId}", "41")
        .replace("{credentialReferenceId}", "43")
        .replace("{bindingId}", "47")
        .replace("{spaceId}", "7");
    if path.ends_with("/okf/candidates") {
        format!("{path}?spaceId=7")
    } else {
        path
    }
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
        "okf.bundle.export.create" => {
            r#"{"spaceId":7,"exportType":"okf_strict","stageForImport":true,"importId":"openapi-roundtrip"}"#
        }
        "okf.bundle.import.create" => r#"{"spaceId":7,"importType":"okf_strict"}"#,
        "okf.lintRuns.create" | "okf.evalRuns.create" => r#"{"spaceId":7}"#,
        "indexes.create" => {
            r#"{"tenantId":"100001","spaceId":"7","indexKind":"hybrid","embeddingProviderId":"provider.embedding.openai","embeddingModel":"text-embedding-3-large","dimension":3072,"metric":"cosine"}"#
        }
        "indexes.rebuild" => r#"{"spaceId":7}"#,
        "retrievalProfiles.create" | "retrievalProfiles.update" => {
            r#"{"tenantId":"100001","name":"Default Hybrid","strategy":"hybrid","topK":8,"minScore":0.4,"rerankEnabled":true,"contextBudgetTokens":2048,"status":"active"}"#
        }
        "compliance.auditEvents.export.create" | "compliance.auditEvents.anonymizeActor.create" => {
            r#"{"actorId":"42"}"#
        }
        _ => "",
    }
}

struct DefaultBackendApi;

impl KnowledgeBackendApi for DefaultBackendApi {}
