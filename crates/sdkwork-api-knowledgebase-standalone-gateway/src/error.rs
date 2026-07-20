use crate::config::GatewayConfigError;
use sdkwork_knowledgebase_agent_provider::async_bridge::AsyncBridgeError;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewaySignalError {
    details: String,
}

impl GatewaySignalError {
    pub(crate) fn all_handlers_unavailable(details: String) -> Self {
        Self { details }
    }
}

impl std::fmt::Display for GatewaySignalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "gateway shutdown signal handlers are unavailable: {}",
            self.details
        )
    }
}

impl std::error::Error for GatewaySignalError {}

#[derive(Debug)]
pub enum GatewayServeError {
    Config(GatewayConfigError),
    Io(std::io::Error),
    Signal(GatewaySignalError),
    DrainTimedOut { timeout: Duration },
}

impl std::fmt::Display for GatewayServeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "gateway configuration is invalid: {error}"),
            Self::Io(error) => write!(formatter, "gateway I/O failed: {error}"),
            Self::Signal(error) => write!(formatter, "gateway signal handling failed: {error}"),
            Self::DrainTimedOut { timeout } => {
                write!(formatter, "gateway HTTP drain timed out after {timeout:?}")
            }
        }
    }
}

impl std::error::Error for GatewayServeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Signal(error) => Some(error),
            Self::DrainTimedOut { .. } => None,
        }
    }
}

#[derive(Debug)]
pub enum GatewayRuntimeError {
    Serve(GatewayServeError),
    Shutdown(AsyncBridgeError),
    ServeAndShutdown {
        serve: GatewayServeError,
        shutdown: AsyncBridgeError,
    },
}

impl std::fmt::Display for GatewayRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serve(error) => write!(formatter, "gateway serve failed: {error}"),
            Self::Shutdown(error) => write!(formatter, "gateway runtime shutdown failed: {error}"),
            Self::ServeAndShutdown { serve, shutdown } => write!(
                formatter,
                "gateway serve failed: {serve}; runtime shutdown also failed: {shutdown}"
            ),
        }
    }
}

impl std::error::Error for GatewayRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serve(error) => Some(error),
            Self::Shutdown(error) => Some(error),
            Self::ServeAndShutdown { serve, .. } => Some(serve),
        }
    }
}

pub(crate) fn merge_gateway_results(
    serve_result: Result<(), GatewayServeError>,
    shutdown_result: Result<(), AsyncBridgeError>,
) -> Result<(), GatewayRuntimeError> {
    match (serve_result, shutdown_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(serve), Ok(())) => Err(GatewayRuntimeError::Serve(serve)),
        (Ok(()), Err(shutdown)) => Err(GatewayRuntimeError::Shutdown(shutdown)),
        (Err(serve), Err(shutdown)) => {
            Err(GatewayRuntimeError::ServeAndShutdown { serve, shutdown })
        }
    }
}
