use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_knowledgebase_contract::browser::{KnowledgeBrowserPage, ListKnowledgeBrowserRequest};
use sdkwork_router_knowledgebase_app_api::{
    build_router_with_browser, ApiResult, KnowledgeAppRequestContext, KnowledgeBrowserApi,
};
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn app_router_mounts_every_app_openapi_operation_path() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
    ))
    .unwrap();
    let app = build_router_with_browser(EmptyBrowserApi);

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
fn app_openapi_uses_collection_schemas_for_okf_list_operations() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
    ))
    .unwrap();

    assert_eq!(
        success_schema_ref(&spec, "okf.concepts.list"),
        "#/components/schemas/OkfConceptSummaryList"
    );
    assert_eq!(
        success_schema_ref(&spec, "okf.concepts.revisions.list"),
        "#/components/schemas/KnowledgeOkfConceptRevisionList"
    );
    assert!(
        spec["components"]["schemas"]["OkfConceptSummaryList"].is_object(),
        "OpenAPI must define OkfConceptSummaryList schema"
    );
    assert!(
        spec["components"]["schemas"]["KnowledgeOkfConceptRevisionList"].is_object(),
        "OpenAPI must define KnowledgeOkfConceptRevisionList schema"
    );
}

#[test]
fn app_openapi_exposes_drive_bound_contract_fields() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
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
fn app_openapi_exposes_standard_rag_and_knowledge_agent_operations() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
    ))
    .unwrap();

    for (operation_id, method, path) in [
        (
            "retrievals.create",
            "post",
            "/app/v3/api/knowledge/retrievals",
        ),
        (
            "retrievals.retrieve",
            "get",
            "/app/v3/api/knowledge/retrievals/{retrievalId}",
        ),
        (
            "contextPacks.create",
            "post",
            "/app/v3/api/knowledge/context_packs",
        ),
        (
            "agentProfiles.create",
            "post",
            "/app/v3/api/knowledge/agent_profiles",
        ),
        (
            "agentProfiles.retrieve",
            "get",
            "/app/v3/api/knowledge/agent_profiles/{profileId}",
        ),
        (
            "agentProfiles.update",
            "patch",
            "/app/v3/api/knowledge/agent_profiles/{profileId}",
        ),
        (
            "agentProfiles.delete",
            "delete",
            "/app/v3/api/knowledge/agent_profiles/{profileId}",
        ),
        (
            "agentProfiles.bindings.list",
            "get",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings",
        ),
        (
            "agentProfiles.bindings.create",
            "post",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings",
        ),
        (
            "agentProfiles.bindings.update",
            "patch",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}",
        ),
        (
            "agentProfiles.bindings.delete",
            "delete",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}",
        ),
        (
            "agentProfiles.retrievalPreview.create",
            "post",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/retrieval_preview",
        ),
        (
            "agentProfiles.chat.create",
            "post",
            "/app/v3/api/knowledge/agent_profiles/{profileId}/chat",
        ),
    ] {
        assert_eq!(
            spec["paths"][path][method]["operationId"], operation_id,
            "missing app RAG operation {operation_id}: {method} {path}"
        );
        assert_eq!(
            spec["paths"][path][method]["x-sdkwork-owner"],
            "sdkwork-knowledgebase"
        );
        assert_eq!(
            spec["paths"][path][method]["x-sdkwork-api-authority"],
            "sdkwork-knowledgebase-app-api"
        );
    }

    for schema_name in [
        "KnowledgeRetrievalRequest",
        "KnowledgeRetrievalResult",
        "KnowledgeContextPackRequest",
        "KnowledgeContextPack",
        "KnowledgeMemoryContextFragment",
        "KnowledgeAgentProfile",
        "KnowledgeAgentBinding",
    ] {
        assert!(
            spec["components"]["schemas"][schema_name].is_object(),
            "OpenAPI must define {schema_name}"
        );
    }
}

#[test]
fn app_openapi_keeps_memory_context_fragments_separate_from_knowledge_chunks() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
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
    let path = template_path
        .replace("{spaceId}", "7")
        .replace("{ingestId}", "11")
        .replace("{documentId}", "13")
        .replace("{conceptId}", "17")
        .replace("{queryId}", "19");
    let path = path
        .replace("{retrievalId}", "23")
        .replace("{profileId}", "41")
        .replace("{bindingId}", "61")
        .replace("{spaceId}", "7");

    if path.ends_with("/browser") {
        format!("{path}?view=files&pageSize=1")
    } else {
        path
    }
}

fn request_body(operation_id: &str) -> &'static str {
    match operation_id {
        "spaces.create" => r#"{"name":"Knowledge Space","description":"Demo"}"#,
        "driveImports.create" => {
            r#"{"spaceId":7,"title":"Quarterly Report","driveStorageProviderId":"provider-kb","driveBucket":"knowledgebase-source","driveObjectKey":"incoming/report.md","idempotencyKey":"drive-report"}"#
        }
        "ingests.create" => {
            r##"{"spaceId":7,"title":"API Note","payloadMarkdown":"# API Note","idempotencyKey":"api-note"}"##
        }
        "documents.create" | "documents.update" => {
            r#"{"spaceId":7,"collectionId":0,"title":"Document","mimeType":"text/markdown"}"#
        }
        "documents.versions.create" => {
            r#"{"documentId":13,"originalObjectRefId":23,"sizeBytes":128,"mimeType":"text/markdown"}"#
        }
        "okf.queries.create" => r#"{"spaceId":7,"query":"What changed?"}"#,
        "okf.queries.fileAnswer" => r##"{"title":"Answer","answerMarkdown":"# Answer"}"##,
        "okf.contextPacks.create" => r#"{"spaceId":7,"query":"Quarterly report"}"#,
        "retrievals.create" => {
            r#"{"tenantId":"20001","query":"Quarterly report","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#
        }
        "contextPacks.create" => {
            r#"{"tenantId":"20001","query":"Quarterly report","bindings":[{"spaceId":"7","priority":10}],"contextBudgetTokens":1200,"includeCitations":true}"#
        }
        "agentProfiles.create" | "agentProfiles.update" => {
            r#"{"tenantId":"20001","name":"Support Agent","systemInstruction":"Answer with citations.","modelProviderId":"provider.model.openai","modelId":"gpt-4.1","status":"active"}"#
        }
        "agentProfiles.bindings.create" | "agentProfiles.bindings.update" => {
            r#"{"tenantId":"20001","profileId":"41","spaceId":"7","priority":10,"enabled":true}"#
        }
        "agentProfiles.retrievalPreview.create" => {
            r#"{"tenantId":"20001","query":"Quarterly report","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#
        }
        "agentProfiles.chat.create" => {
            r#"{"tenantId":"20001","message":"What changed in the quarterly report?","mode":"okf_bundle"}"#
        }
        "spaces.contextBindings.create" => {
            r#"{"spaceId":"7","contextType":"chat_group","contextId":"grp-ops","accessLevel":"reader"}"#
        }
        "contextBindings.update" => r#"{"accessLevel":"writer"}"#,
        _ => "",
    }
}

struct EmptyBrowserApi;

#[async_trait]
impl KnowledgeBrowserApi for EmptyBrowserApi {
    async fn list_browser(
        &self,
        _context: KnowledgeAppRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        Ok(KnowledgeBrowserPage {
            space_id: request.space_id,
            drive_space_id: "drv-kb-001".to_string(),
            parent_id: request.parent_id,
            view: request.view,
            page_size: request.page_size.unwrap_or(1),
            items: vec![],
            next_cursor: None,
        })
    }
}
