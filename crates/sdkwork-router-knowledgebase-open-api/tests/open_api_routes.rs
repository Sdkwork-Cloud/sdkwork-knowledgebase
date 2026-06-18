use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use sdkwork_iam_web_adapter::IamDatabaseWebRequestContextResolver;
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserNode, KnowledgeBrowserNodePermissions, KnowledgeBrowserNodeType,
    KnowledgeBrowserPage, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use sdkwork_knowledgebase_contract::document::{
    KnowledgeDocument, KnowledgeDocumentList, KnowledgeDocumentState,
    KnowledgeDocumentVersionState, KnowledgeDocumentVisibility,
};
use sdkwork_knowledgebase_contract::ingest::{
    IngestionJob, IngestionJobState, KnowledgeIngestRequest,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeContextFragment, KnowledgeContextPack, KnowledgeContextPackRequest,
    KnowledgeRetrievalMethod, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
    KnowledgeRetrievalTrace,
};
use sdkwork_router_knowledgebase_open_api::{
    build_router_with_open_api, manifest, open_route_manifest, wrap_router_with_web_framework,
    wrap_router_with_web_framework_from_env, ApiResult, KnowledgeOpenApi,
    KnowledgeOpenApiRequestContext, ProblemDetails,
};
use sdkwork_web_core::RouteAuth;
use serde_json::Value;
use std::sync::Mutex;
use tower::util::ServiceExt;

#[test]
fn open_api_manifest_uses_public_knowledge_prefix_and_api_key_auth() {
    assert_eq!(
        manifest::PACKAGE_NAME,
        "sdkwork-router-knowledgebase-open-api"
    );
    assert_eq!(manifest::SURFACE, "open-api");
    assert_eq!(manifest::OWNER, "sdkwork-knowledgebase");
    assert_eq!(manifest::DOMAIN, "intelligence");
    assert_eq!(manifest::CAPABILITY, "knowledgebase");
    assert_eq!(manifest::API_AUTHORITY, "sdkwork-knowledgebase-open-api");
    assert_eq!(manifest::SDK_FAMILY, "sdkwork-knowledgebase-sdk");
    assert_eq!(manifest::PREFIX, "/knowledge/v3/api");

    let routes = manifest::ROUTES;
    assert_eq!(routes.len(), 8);
    assert!(routes.iter().all(|route| route.auth_mode == "api-key"));
    assert!(routes
        .iter()
        .all(|route| route.path.starts_with("/knowledge/v3/api")));
    assert!(routes
        .iter()
        .all(|route| !route.path.starts_with("/app/v3/api")));
    assert!(routes
        .iter()
        .all(|route| !route.path.starts_with("/backend/v3/api")));

    assert_route("POST", "/knowledge/v3/api/retrievals", "retrievals.create");
    assert_route(
        "GET",
        "/knowledge/v3/api/retrievals/{retrievalId}",
        "retrievals.retrieve",
    );
    assert_route(
        "POST",
        "/knowledge/v3/api/context_packs",
        "contextPacks.create",
    );
    assert_route("POST", "/knowledge/v3/api/ingests", "ingests.create");
    assert_route(
        "GET",
        "/knowledge/v3/api/ingests/{ingestId}",
        "ingests.retrieve",
    );
    assert_route("GET", "/knowledge/v3/api/documents", "documents.list");
    assert_route(
        "GET",
        "/knowledge/v3/api/documents/{documentId}",
        "documents.retrieve",
    );
    assert_route(
        "GET",
        "/knowledge/v3/api/spaces/{spaceId}/browser",
        "spaces.browser.list",
    );
}

#[tokio::test]
async fn open_retrieval_route_calls_injected_service_with_api_key_context() {
    let service = RecordingOpenApi::default();
    let app = build_router_with_open_api(service.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/knowledge/v3/api/retrievals")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    r#"{"tenantId":"20001","actorId":"30001","query":"enterprise renewal support","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    assert_eq!(body["retrievalId"], "701");
    assert_eq!(body["hits"][0]["chunkId"], "11");
    assert_eq!(service.contexts(), vec![("api-key-001".to_string(), 20001)]);
}

#[tokio::test]
async fn open_retrieval_route_rejects_missing_api_key_context() {
    let app = build_router_with_open_api(RecordingOpenApi::default());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/knowledge/v3/api/retrievals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"20001","query":"enterprise renewal support","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/problem+json"
    );
    let problem: ProblemDetails = response_model(response).await;
    assert_eq!(
        problem.code.as_deref(),
        Some("missing_open_api_request_context")
    );
}

#[tokio::test]
async fn open_context_pack_route_calls_injected_service() {
    let app = build_router_with_open_api(RecordingOpenApi::default());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/knowledge/v3/api/context_packs")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    r#"{"tenantId":"20001","actorId":"30001","query":"enterprise renewal support","bindings":[{"spaceId":"7","priority":10}],"contextBudgetTokens":80,"includeCitations":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = response_json(response).await;
    assert_eq!(body["contextPackId"], "801");
    assert_eq!(body["fragments"][0]["chunkId"], "11");
}

#[tokio::test]
async fn open_browser_route_preserves_query_parameters() {
    let service = RecordingOpenApi::default();
    let app = build_router_with_open_api(service.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/knowledge/v3/api/spaces/7/browser?view=wiki&pageSize=25&parentId=node-wiki&cursor=c1")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let page: KnowledgeBrowserPage = response_model(response).await;
    assert_eq!(page.space_id, 7);
    assert_eq!(page.view, KnowledgeBrowserView::Wiki);
    assert_eq!(
        service.last_browser_request().unwrap(),
        ListKnowledgeBrowserRequest {
            space_id: 7,
            parent_id: Some("node-wiki".to_string()),
            view: KnowledgeBrowserView::Wiki,
            cursor: Some("c1".to_string()),
            page_size: Some(25),
        }
    );
}

#[tokio::test]
async fn open_router_exposes_document_and_ingest_read_routes() {
    let app = build_router_with_open_api(RecordingOpenApi::default());

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/knowledge/v3/api/documents")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    assert_eq!(response_json(list_response).await["items"][0]["id"], 901);

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/knowledge/v3/api/documents/901")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    assert_eq!(response_json(retrieve_response).await["id"], 901);

    let ingest_response = app
        .oneshot(
            Request::builder()
                .uri("/knowledge/v3/api/ingests/51")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ingest_response.status(), StatusCode::OK);
    assert_eq!(response_json(ingest_response).await["id"], 51);
}

#[test]
fn open_route_manifest_declares_api_key_auth_for_all_operations() {
    let manifest = open_route_manifest();
    assert_eq!(manifest::ROUTES.len(), 8);
    for entry in manifest::ROUTES {
        let matched = manifest
            .match_route(entry.method, entry.path)
            .unwrap_or_else(|| {
                panic!(
                    "missing http route manifest for {} {}",
                    entry.method, entry.path
                )
            });
        assert_eq!(matched.auth, RouteAuth::ApiKey);
        assert_eq!(matched.operation_id, entry.operation_id);
    }
}

#[tokio::test]
async fn open_router_web_framework_rejects_unauthenticated_requests() {
    let app = wrap_router_with_web_framework_from_env(build_router_with_open_api(
        RecordingOpenApi::default(),
    ))
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/knowledge/v3/api/retrievals")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenantId":"20001","query":"enterprise renewal support","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn open_router_web_framework_accepts_dev_inline_api_key_before_handler() {
    let service = RecordingOpenApi::default();
    let app = wrap_router_with_web_framework(
        IamDatabaseWebRequestContextResolver::new(None),
        build_router_with_open_api(service.clone()),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/knowledge/v3/api/retrievals")
                .header("content-type", "application/json")
                .header(
                    "x-api-key",
                    "api_key_id=api-key-001;tenant_id=20001;user_id=30001;app_id=knowledgebase",
                )
                .body(Body::from(
                    r#"{"tenantId":"20001","actorId":"30001","query":"enterprise renewal support","bindings":[{"spaceId":"7","priority":10}],"methods":["hybrid"],"includeCitations":true,"includeTrace":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(service.contexts(), vec![("api-key-001".to_string(), 20001)]);
}

fn assert_route(method: &str, path: &str, operation_id: &str) {
    assert!(
        manifest::ROUTES.iter().any(|route| {
            route.method == method && route.path == path && route.operation_id == operation_id
        }),
        "missing route {method} {path} {operation_id}"
    );
}

fn open_context() -> KnowledgeOpenApiRequestContext {
    KnowledgeOpenApiRequestContext {
        api_key_id: "api-key-001".to_string(),
        tenant_id: 20001,
        actor_id: Some(30001),
    }
}

#[derive(Clone, Default)]
struct RecordingOpenApi {
    contexts: std::sync::Arc<Mutex<Vec<(String, u64)>>>,
    browser_request: std::sync::Arc<Mutex<Option<ListKnowledgeBrowserRequest>>>,
}

impl RecordingOpenApi {
    fn contexts(&self) -> Vec<(String, u64)> {
        self.contexts.lock().unwrap().clone()
    }

    fn last_browser_request(&self) -> Option<ListKnowledgeBrowserRequest> {
        self.browser_request.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeOpenApi for RecordingOpenApi {
    async fn create_retrieval(
        &self,
        context: KnowledgeOpenApiRequestContext,
        _request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.contexts
            .lock()
            .unwrap()
            .push((context.api_key_id, context.tenant_id));
        Ok(retrieval_result(701))
    }

    async fn retrieve_retrieval(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Ok(retrieval_result(retrieval_id))
    }

    async fn create_context_pack(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        Ok(KnowledgeContextPack {
            context_pack_id: 801,
            retrieval_id: Some(701),
            query: "enterprise renewal support".to_string(),
            fragments: vec![context_fragment()],
            memory_fragments: vec![],
            estimated_tokens: 8,
            citations: vec![],
            truncated: false,
        })
    }

    async fn create_ingest(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        request: KnowledgeIngestRequest,
    ) -> ApiResult<IngestionJob> {
        Ok(IngestionJob {
            id: 51,
            space_id: request.space_id,
            source_type: "api_payload".to_string(),
            idempotency_key: request.idempotency_key,
            state: IngestionJobState::Queued,
            error_message: None,
        })
    }

    async fn retrieve_ingest(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        ingest_id: u64,
    ) -> ApiResult<IngestionJob> {
        Ok(IngestionJob {
            id: ingest_id,
            space_id: 7,
            source_type: "api_payload".to_string(),
            idempotency_key: "api-note".to_string(),
            state: IngestionJobState::Succeeded,
            error_message: None,
        })
    }

    async fn list_documents(
        &self,
        _context: KnowledgeOpenApiRequestContext,
    ) -> ApiResult<KnowledgeDocumentList> {
        Ok(KnowledgeDocumentList {
            items: vec![document(901)],
        })
    }

    async fn retrieve_document(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocument> {
        Ok(document(document_id))
    }

    async fn list_browser(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        *self.browser_request.lock().unwrap() = Some(request.clone());
        Ok(KnowledgeBrowserPage {
            space_id: request.space_id,
            drive_space_id: "drv-kb-001".to_string(),
            parent_id: request.parent_id,
            view: request.view,
            page_size: request.page_size.unwrap_or(50),
            items: vec![KnowledgeBrowserNode {
                id: "node-index".to_string(),
                node_type: KnowledgeBrowserNodeType::WikiPage,
                name: "index.md".to_string(),
                parent_id: Some("node-wiki".to_string()),
                path: "wiki/index.md".to_string(),
                drive_space_id: Some("drv-kb-001".to_string()),
                drive_node_id: Some("node-index".to_string()),
                document_id: None,
                document_version_id: None,
                wiki_page_id: Some(1),
                wiki_revision_id: Some(2),
                mime_type: Some("text/markdown; charset=utf-8".to_string()),
                size_bytes: Some(64),
                ingest_state: None,
                parse_state: None,
                index_state: None,
                wiki_state: Some("published".to_string()),
                children_count: None,
                updated_at: "2026-06-04T12:00:00Z".to_string(),
                permissions: KnowledgeBrowserNodePermissions::read_only(),
            }],
            next_cursor: None,
        })
    }
}

fn retrieval_result(retrieval_id: u64) -> KnowledgeRetrievalResult {
    KnowledgeRetrievalResult {
        retrieval_id,
        trace: Some(KnowledgeRetrievalTrace {
            retrieval_trace_id: retrieval_id,
            status: "succeeded".to_string(),
            latency_ms: Some(9),
            result_count: 1,
        }),
        hits: vec![context_fragment()],
    }
}

fn context_fragment() -> KnowledgeContextFragment {
    KnowledgeContextFragment {
        chunk_id: 11,
        document_id: 101,
        document_version_id: Some(201),
        space_id: 7,
        collection_id: None,
        title: "Support Playbook".to_string(),
        content: "enterprise renewal support answer".to_string(),
        score: Some(0.91),
        rank: 1,
        token_count: Some(8),
        retrieval_method: KnowledgeRetrievalMethod::Hybrid,
        citation: None,
    }
}

fn document(document_id: u64) -> KnowledgeDocument {
    KnowledgeDocument {
        id: document_id,
        space_id: 7,
        collection_id: 0,
        source_id: Some(31),
        original_file_drive_node_id: Some("node-index".to_string()),
        title: "Support Playbook".to_string(),
        mime_type: Some("text/markdown".to_string()),
        language: Some("en-US".to_string()),
        current_version_id: Some(101),
        visibility: KnowledgeDocumentVisibility::Organization,
        content_state: KnowledgeDocumentState::Ready,
        index_state: KnowledgeDocumentVersionState::Succeeded,
    }
}

async fn response_json(response: axum::response::Response) -> Value {
    response_model(response).await
}

async fn response_model<T>(response: axum::response::Response) -> T
where
    T: serde::de::DeserializeOwned,
{
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
