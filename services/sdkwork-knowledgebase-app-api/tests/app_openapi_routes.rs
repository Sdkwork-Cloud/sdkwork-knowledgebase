use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sdkwork_knowledgebase_app_api::{build_router_with_browser, KnowledgeBrowserApi};
use sdkwork_knowledgebase_contract::browser::{KnowledgeBrowserPage, ListKnowledgeBrowserRequest};
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
fn app_openapi_uses_collection_schemas_for_wiki_list_operations() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
    ))
    .unwrap();

    assert_eq!(
        success_schema_ref(&spec, "wiki.pages.list"),
        "#/components/schemas/WikiPageSummaryList"
    );
    assert_eq!(
        success_schema_ref(&spec, "wiki.pages.revisions.list"),
        "#/components/schemas/KnowledgeWikiPageRevisionList"
    );
    assert!(
        spec["components"]["schemas"]["WikiPageSummaryList"].is_object(),
        "OpenAPI must define WikiPageSummaryList schema"
    );
    assert!(
        spec["components"]["schemas"]["KnowledgeWikiPageRevisionList"].is_object(),
        "OpenAPI must define KnowledgeWikiPageRevisionList schema"
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
    let path = template_path
        .replace("{spaceId}", "7")
        .replace("{ingestId}", "11")
        .replace("{documentId}", "13")
        .replace("{pageId}", "17")
        .replace("{queryId}", "19");

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
            r#"{"spaceId":7,"title":"Quarterly Report","driveBucket":"knowledgebase-source","driveObjectKey":"incoming/report.md","idempotencyKey":"drive-report"}"#
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
        "wiki.queries.create" => r#"{"spaceId":7,"query":"What changed?"}"#,
        "wiki.queries.fileAnswer" => r##"{"title":"Answer","answerMarkdown":"# Answer"}"##,
        "wiki.contextPacks.create" => r#"{"spaceId":7,"query":"Quarterly report"}"#,
        _ => "",
    }
}

struct EmptyBrowserApi;

#[async_trait]
impl KnowledgeBrowserApi for EmptyBrowserApi {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> Result<KnowledgeBrowserPage, String> {
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
