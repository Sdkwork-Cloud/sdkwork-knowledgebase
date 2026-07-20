use axum::Router;
use sdkwork_api_knowledgebase_standalone_gateway::{
    serve_router_with_runtime_shutdown, shutdown_runtime_services, GatewayRuntimeError,
};
use sdkwork_knowledgebase_agent_provider::async_bridge::{block_on_async, AsyncBridgeError};

#[tokio::test]
async fn bind_failure_still_shuts_down_runtime_services() {
    let previous_environment = std::env::var_os("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
    std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "development");
    assert_eq!(block_on_async(async { 1_u8 }), Ok(1_u8));

    let error = serve_router_with_runtime_shutdown(
        "not-a-valid-socket-address",
        "gateway-shutdown-test",
        Router::new(),
    )
    .await
    .expect_err("invalid bind must fail");

    assert!(matches!(error, GatewayRuntimeError::Serve(_)));
    assert_eq!(
        block_on_async(async { 2_u8 }),
        Err(AsyncBridgeError::ShuttingDown)
    );
    assert_eq!(shutdown_runtime_services(), Ok(()));
    match previous_environment {
        Some(value) => std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", value),
        None => std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT"),
    }
}
