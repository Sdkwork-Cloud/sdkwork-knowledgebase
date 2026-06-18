use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_router_knowledgebase_app_api::{
    build_router_with_shared_app_api_and_readiness, dev_auth, paths, KnowledgeAppApi,
};
use serde_json::Value;
use std::sync::Arc;
use tower::util::ServiceExt;

struct UnimplementedAppApi;

#[async_trait::async_trait]
impl KnowledgeAppApi for UnimplementedAppApi {}

#[tokio::test]
async fn contract_health_route_matches_openapi_path() {
    let app = dev_auth::with_dev_app_auth(
        build_router_with_shared_app_api_and_readiness(Arc::new(UnimplementedAppApi), None),
        1,
        Some(1),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(paths::HEALTHZ)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn contract_context_binding_operations_are_declared() {
    let spec: Value = serde_json::from_str(include_str!(
        "../../../sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json"
    ))
    .unwrap();

    for operation_id in [
        "spaces.contextBindings.list",
        "spaces.contextBindings.create",
        "contextBindings.retrieve",
        "contextBindings.update",
        "contextBindings.delete",
        "uploadSessions.create",
        "uploadSessions.complete",
    ] {
        assert!(
            spec["paths"].as_object().unwrap().values().any(|methods| {
                methods
                    .as_object()
                    .unwrap()
                    .values()
                    .any(|operation| operation["operationId"] == operation_id)
            }),
            "missing operationId in authority OpenAPI: {operation_id}"
        );
    }
}
