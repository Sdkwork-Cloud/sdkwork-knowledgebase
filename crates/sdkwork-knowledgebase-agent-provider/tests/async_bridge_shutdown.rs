use sdkwork_knowledgebase_agent_provider::async_bridge::{
    block_on_async, shutdown_async_bridge, AsyncBridgeError,
};

#[test]
fn shutdown_before_initialization_is_sticky() {
    assert_eq!(shutdown_async_bridge(), Ok(()));
    assert_eq!(
        block_on_async(async { 42_u8 }),
        Err(AsyncBridgeError::ShuttingDown)
    );
    assert_eq!(shutdown_async_bridge(), Ok(()));
}
