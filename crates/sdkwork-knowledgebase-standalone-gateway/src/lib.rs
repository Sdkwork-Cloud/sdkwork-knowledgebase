mod config;
mod error;
mod server;

use axum::Router;
use sdkwork_knowledgebase_agent_provider::async_bridge::{shutdown_async_bridge, AsyncBridgeError};
use sdkwork_knowledgebase_observability::tracing_support;

pub use config::GatewayConfigError;
pub use error::{GatewayRuntimeError, GatewayServeError, GatewaySignalError};

use config::GatewayServerConfig;
use server::{run_with_runtime_cleanup, serve_router_with_config};

#[cfg(test)]
mod tests;

pub fn init_tracing(service_name: &str) {
    tracing_support::init_tracing_from_env(service_name);
}

pub fn shutdown_runtime_services() -> Result<(), AsyncBridgeError> {
    shutdown_async_bridge()
}

pub async fn serve_router_with_runtime_shutdown(
    listen_addr: &str,
    service_name: &str,
    router: Router,
) -> Result<(), GatewayRuntimeError> {
    let config = GatewayServerConfig::from_env();
    run_with_runtime_cleanup(
        serve_router_with_config(
            listen_addr,
            service_name,
            router,
            config,
            shutdown_signal_result(),
        ),
        shutdown_runtime_services,
    )
    .await
}

pub async fn shutdown_signal() {
    if let Err(error) = shutdown_signal_result().await {
        tracing::error!(%error, "all gateway shutdown signal handlers failed");
        std::future::pending::<()>().await;
    }
}

#[derive(Debug)]
enum SignalWaitOutcome {
    Received,
    Failed(String),
}

async fn wait_for_ctrl_c() -> SignalWaitOutcome {
    match tokio::signal::ctrl_c().await {
        Ok(()) => SignalWaitOutcome::Received,
        Err(error) => SignalWaitOutcome::Failed(error.to_string()),
    }
}

#[cfg(unix)]
async fn wait_for_sigterm() -> SignalWaitOutcome {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut signal) => match signal.recv().await {
            Some(()) => SignalWaitOutcome::Received,
            None => SignalWaitOutcome::Failed(
                "SIGTERM signal stream closed before receiving a signal".to_string(),
            ),
        },
        Err(error) => SignalWaitOutcome::Failed(error.to_string()),
    }
}

#[cfg(any(unix, test))]
async fn wait_for_two_signal_sources<First, Second>(
    first_name: &'static str,
    first: First,
    second_name: &'static str,
    second: Second,
) -> Result<(), GatewaySignalError>
where
    First: std::future::Future<Output = SignalWaitOutcome>,
    Second: std::future::Future<Output = SignalWaitOutcome>,
{
    tokio::pin!(first);
    tokio::pin!(second);
    tokio::select! {
        outcome = &mut first => match outcome {
            SignalWaitOutcome::Received => Ok(()),
            SignalWaitOutcome::Failed(first_error) => {
                tracing::error!(source = first_name, error = %first_error, "gateway shutdown signal source failed");
                match second.await {
                    SignalWaitOutcome::Received => Ok(()),
                    SignalWaitOutcome::Failed(second_error) => Err(
                        GatewaySignalError::all_handlers_unavailable(format!(
                            "{first_name}: {first_error}; {second_name}: {second_error}"
                        )),
                    ),
                }
            }
        },
        outcome = &mut second => match outcome {
            SignalWaitOutcome::Received => Ok(()),
            SignalWaitOutcome::Failed(second_error) => {
                tracing::error!(source = second_name, error = %second_error, "gateway shutdown signal source failed");
                match first.await {
                    SignalWaitOutcome::Received => Ok(()),
                    SignalWaitOutcome::Failed(first_error) => Err(
                        GatewaySignalError::all_handlers_unavailable(format!(
                            "{second_name}: {second_error}; {first_name}: {first_error}"
                        )),
                    ),
                }
            }
        }
    }
}

async fn shutdown_signal_result() -> Result<(), GatewaySignalError> {
    #[cfg(unix)]
    let result =
        wait_for_two_signal_sources("Ctrl+C", wait_for_ctrl_c(), "SIGTERM", wait_for_sigterm())
            .await;

    #[cfg(not(unix))]
    let result = match wait_for_ctrl_c().await {
        SignalWaitOutcome::Received => Ok(()),
        SignalWaitOutcome::Failed(error) => Err(GatewaySignalError::all_handlers_unavailable(
            format!("Ctrl+C: {error}"),
        )),
    };

    if result.is_ok() {
        tracing::info!("shutdown signal received");
    }
    result
}
