use crate::config::{GatewayConfigError, GatewayServerConfig};
#[cfg(test)]
use crate::config::{DEFAULT_HEADER_READ_TIMEOUT, DEFAULT_MAX_CONNECTIONS};
use crate::error::{
    merge_gateway_results, GatewayRuntimeError, GatewayServeError, GatewaySignalError,
};
use axum::{body::Body, http::Request, Router};
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::{TokioIo, TokioTimer};
use sdkwork_knowledgebase_agent_provider::async_bridge::AsyncBridgeError;
use sdkwork_knowledgebase_observability::wrap_router_with_metrics;
use std::future::Future;
use std::io;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::{JoinError, JoinSet};
use tokio::time::Instant;
use tower::ServiceExt;

const HTTP1_MAX_BUFFER_SIZE_BYTES: usize = 64 * 1_024;

pub(crate) async fn serve_router_with_config<ShutdownSignal>(
    listen_addr: &str,
    service_name: &str,
    router: Router,
    config: Result<GatewayServerConfig, GatewayConfigError>,
    shutdown_signal: ShutdownSignal,
) -> Result<(), GatewayServeError>
where
    ShutdownSignal: Future<Output = Result<(), GatewaySignalError>> + Send + 'static,
{
    let config = config.map_err(GatewayServeError::Config)?;
    serve_router(listen_addr, service_name, router, shutdown_signal, config).await
}

async fn serve_router<ShutdownSignal>(
    listen_addr: &str,
    service_name: &str,
    router: Router,
    shutdown_signal: ShutdownSignal,
    config: GatewayServerConfig,
) -> Result<(), GatewayServeError>
where
    ShutdownSignal: Future<Output = Result<(), GatewaySignalError>> + Send + 'static,
{
    crate::init_tracing(service_name);
    let router = wrap_router_with_metrics(router);
    let listener = TcpListener::bind(listen_addr)
        .await
        .map_err(GatewayServeError::Io)?;
    tracing::info!(%listen_addr, service = service_name, "listening");

    serve_listener_with_limits(
        listener,
        router,
        shutdown_signal,
        config.drain_timeout,
        config.max_connections,
        config.header_read_timeout,
    )
    .await
}

#[cfg(test)]
pub(crate) async fn serve_listener_with_drain_timeout<ShutdownSignal>(
    listener: TcpListener,
    router: Router,
    shutdown_signal: ShutdownSignal,
    drain_timeout: Duration,
) -> Result<(), GatewayServeError>
where
    ShutdownSignal: Future<Output = Result<(), GatewaySignalError>> + Send + 'static,
{
    serve_listener_with_limits(
        listener,
        router,
        shutdown_signal,
        drain_timeout,
        DEFAULT_MAX_CONNECTIONS,
        DEFAULT_HEADER_READ_TIMEOUT,
    )
    .await
}

pub(crate) async fn serve_listener_with_limits<ShutdownSignal>(
    listener: TcpListener,
    router: Router,
    shutdown_signal: ShutdownSignal,
    drain_timeout: Duration,
    max_connections: usize,
    header_read_timeout: Duration,
) -> Result<(), GatewayServeError>
where
    ShutdownSignal: Future<Output = Result<(), GatewaySignalError>> + Send + 'static,
{
    debug_assert!(max_connections > 0);
    let (connection_shutdown_tx, connection_shutdown_rx) = watch::channel(false);
    let mut connections = JoinSet::new();
    tokio::pin!(shutdown_signal);

    let signal_error = loop {
        if connections.len() >= max_connections {
            tokio::select! {
                biased;
                signal_result = &mut shutdown_signal => {
                    break signal_result.err();
                },
                joined = connections.join_next() => observe_connection_completion(joined),
            }
        } else {
            tokio::select! {
                biased;
                signal_result = &mut shutdown_signal => {
                    break signal_result.err();
                },
                joined = connections.join_next(), if !connections.is_empty() => {
                    observe_connection_completion(joined);
                }
                accepted = accept_with_retry(&listener) => {
                    let (stream, remote_addr) = accepted;
                    let connection_router = router.clone();
                    let connection_shutdown = connection_shutdown_rx.clone();
                    connections.spawn(async move {
                        serve_connection(
                            stream,
                            remote_addr,
                            connection_router,
                            connection_shutdown,
                            header_read_timeout,
                        )
                        .await;
                    });
                }
            }
        }
    };

    connection_shutdown_tx.send_replace(true);
    drop(connection_shutdown_rx);
    drop(listener);
    if let Some(error) = signal_error {
        abort_and_reap_connections(&mut connections).await;
        return Err(GatewayServeError::Signal(error));
    }

    let drain_deadline = Instant::now() + drain_timeout;

    while !connections.is_empty() {
        tokio::select! {
            biased;
            () = tokio::time::sleep_until(drain_deadline) => {
                abort_and_reap_connections(&mut connections).await;
                return Err(GatewayServeError::DrainTimedOut {
                    timeout: drain_timeout,
                });
            }
            joined = connections.join_next() => observe_connection_completion(joined),
        }
    }

    Ok(())
}

async fn abort_and_reap_connections(connections: &mut JoinSet<()>) {
    connections.abort_all();
    while let Some(joined) = connections.join_next().await {
        if let Err(error) = joined {
            if !error.is_cancelled() {
                log_connection_join_error(error);
            }
        }
    }
}

async fn accept_with_retry(listener: &TcpListener) -> (TcpStream, std::net::SocketAddr) {
    loop {
        match listener.accept().await {
            Ok(connection) => return connection,
            Err(error) if is_connection_error(&error) => continue,
            Err(error) => {
                tracing::error!(%error, "gateway TCP accept failed; retrying");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn is_connection_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    )
}

async fn serve_connection(
    stream: TcpStream,
    remote_addr: std::net::SocketAddr,
    router: Router,
    mut shutdown: watch::Receiver<bool>,
    header_read_timeout: Duration,
) {
    let service = service_fn(move |request: Request<Incoming>| {
        let router = router.clone();
        async move { router.oneshot(request.map(Body::new)).await }
    });
    let mut builder = http1::Builder::new();
    builder
        .timer(TokioTimer::new())
        .header_read_timeout(header_read_timeout)
        .max_buf_size(HTTP1_MAX_BUFFER_SIZE_BYTES);
    let connection = builder.serve_connection(TokioIo::new(stream), service);
    tokio::pin!(connection);

    let shutdown_started = *shutdown.borrow();
    if shutdown_started {
        connection.as_mut().graceful_shutdown();
    }

    let result = if shutdown_started {
        connection.await
    } else {
        tokio::select! {
            result = &mut connection => result,
            _ = shutdown.changed() => {
                connection.as_mut().graceful_shutdown();
                connection.await
            }
        }
    };
    if let Err(error) = result {
        tracing::debug!(%error, %remote_addr, "gateway connection ended with an error");
    }
}

fn observe_connection_completion(joined: Option<Result<(), JoinError>>) {
    if let Some(Err(error)) = joined {
        log_connection_join_error(error);
    }
}

fn log_connection_join_error(error: JoinError) {
    if error.is_panic() {
        tracing::error!(%error, "gateway connection task panicked");
    } else if !error.is_cancelled() {
        tracing::warn!(%error, "gateway connection task stopped unexpectedly");
    }
}

pub(crate) async fn run_with_runtime_cleanup<ServeFuture, ShutdownRuntime>(
    serve: ServeFuture,
    shutdown_runtime: ShutdownRuntime,
) -> Result<(), GatewayRuntimeError>
where
    ServeFuture: Future<Output = Result<(), GatewayServeError>>,
    ShutdownRuntime: FnOnce() -> Result<(), AsyncBridgeError> + Send + 'static,
{
    let serve_result = serve.await;
    let shutdown_result = tokio::task::spawn_blocking(shutdown_runtime)
        .await
        .unwrap_or(Err(AsyncBridgeError::ShutdownFailed));
    merge_gateway_results(serve_result, shutdown_result)
}
