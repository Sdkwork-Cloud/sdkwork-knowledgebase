use super::config::{
    resolve_gateway_drain_timeout, resolve_gateway_header_read_timeout,
    resolve_gateway_max_connections,
};
use super::error::merge_gateway_results;
use super::server::{
    run_with_runtime_cleanup, serve_listener_with_drain_timeout, serve_listener_with_limits,
    serve_router_with_config,
};
use super::{
    wait_for_two_signal_sources, GatewayConfigError, GatewayRuntimeError, GatewayServeError,
    GatewaySignalError, SignalWaitOutcome,
};
use axum::{extract::State, response::Response, routing::get, Router};
use sdkwork_knowledgebase_agent_provider::async_bridge::AsyncBridgeError;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Notify};

#[derive(Clone)]
struct PendingHandlerState {
    entered: Arc<Notify>,
    dropped: Arc<AtomicBool>,
}

struct PendingHandlerDropGuard(Arc<AtomicBool>);

impl Drop for PendingHandlerDropGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

async fn never_finishes(State(state): State<PendingHandlerState>) -> Response {
    let _drop_guard = PendingHandlerDropGuard(state.dropped.clone());
    state.entered.notify_one();
    std::future::pending::<Response>().await
}

#[test]
fn drain_timeout_defaults_to_thirty_seconds() {
    assert_eq!(
        resolve_gateway_drain_timeout(None, Some("development")),
        Ok(Duration::from_secs(30))
    );
    assert_eq!(
        resolve_gateway_drain_timeout(None, None),
        Err(GatewayConfigError::MissingEnvironment)
    );
}

#[test]
fn production_drain_timeout_accepts_only_five_through_three_hundred_seconds() {
    assert_eq!(
        resolve_gateway_drain_timeout(Some("5"), Some("production")),
        Ok(Duration::from_secs(5))
    );
    assert_eq!(
        resolve_gateway_drain_timeout(Some("300"), Some("production")),
        Ok(Duration::from_secs(300))
    );

    assert!(matches!(
        resolve_gateway_drain_timeout(Some("4"), Some("production")),
        Err(GatewayConfigError::ProductionDrainTimeoutOutOfRange {
            seconds: 4,
            minimum_seconds: 5,
            maximum_seconds: 300,
        })
    ));
    assert!(matches!(
        resolve_gateway_drain_timeout(Some("301"), Some("production")),
        Err(GatewayConfigError::ProductionDrainTimeoutOutOfRange {
            seconds: 301,
            minimum_seconds: 5,
            maximum_seconds: 300,
        })
    ));
}

#[test]
fn malformed_drain_timeout_is_rejected() {
    assert!(matches!(
        resolve_gateway_drain_timeout(Some("thirty"), Some("development")),
        Err(GatewayConfigError::InvalidDrainTimeoutSeconds { value })
            if value == "thirty"
    ));
    assert!(matches!(
        resolve_gateway_drain_timeout(Some("18446744073709551615"), Some("development")),
        Err(GatewayConfigError::DrainTimeoutExceedsMaximum {
            seconds: u64::MAX,
            maximum_seconds: 300,
        })
    ));
}

#[test]
fn lifecycle_environment_must_use_a_canonical_value() {
    for value in ["prod", "Production", "production ", ""] {
        assert!(matches!(
            resolve_gateway_drain_timeout(Some("30"), Some(value)),
            Err(GatewayConfigError::InvalidEnvironment { value: actual })
                if actual == value
        ));
    }
}

#[test]
fn active_connection_limit_is_bounded_and_strictly_validated() {
    assert_eq!(resolve_gateway_max_connections(None), Ok(4_096));
    assert_eq!(resolve_gateway_max_connections(Some("1")), Ok(1));
    assert_eq!(resolve_gateway_max_connections(Some("16384")), Ok(16_384));
    assert!(matches!(
        resolve_gateway_max_connections(Some("many")),
        Err(GatewayConfigError::InvalidMaxConnections { value }) if value == "many"
    ));
    for value in ["0", "16385"] {
        assert!(matches!(
            resolve_gateway_max_connections(Some(value)),
            Err(GatewayConfigError::MaxConnectionsOutOfRange { .. })
        ));
    }
}

#[test]
fn header_read_timeout_is_bounded_and_strictly_validated() {
    assert_eq!(
        resolve_gateway_header_read_timeout(None),
        Ok(Duration::from_secs(10))
    );
    assert_eq!(
        resolve_gateway_header_read_timeout(Some("1")),
        Ok(Duration::from_secs(1))
    );
    assert_eq!(
        resolve_gateway_header_read_timeout(Some("30")),
        Ok(Duration::from_secs(30))
    );
    assert!(matches!(
        resolve_gateway_header_read_timeout(Some("slow")),
        Err(GatewayConfigError::InvalidHeaderReadTimeoutSeconds { value })
            if value == "slow"
    ));
    for value in ["0", "31"] {
        assert!(matches!(
            resolve_gateway_header_read_timeout(Some(value)),
            Err(GatewayConfigError::HeaderReadTimeoutOutOfRange { .. })
        ));
    }
}

#[test]
fn merge_preserves_serve_only_failure() {
    let serve = GatewayServeError::Io(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        "bind failed",
    ));

    let error = merge_gateway_results(Err(serve), Ok(())).expect_err("serve must fail");

    assert!(matches!(
        error,
        GatewayRuntimeError::Serve(GatewayServeError::Io(error))
            if error.kind() == std::io::ErrorKind::AddrNotAvailable
    ));
}

#[test]
fn merge_preserves_shutdown_only_failure() {
    let error = merge_gateway_results(Ok(()), Err(AsyncBridgeError::ShutdownFailed))
        .expect_err("shutdown must fail");

    assert!(matches!(
        error,
        GatewayRuntimeError::Shutdown(AsyncBridgeError::ShutdownFailed)
    ));
}

#[test]
fn merge_preserves_serve_and_shutdown_failures() {
    let serve = GatewayServeError::DrainTimedOut {
        timeout: Duration::from_millis(25),
    };

    let error = merge_gateway_results(Err(serve), Err(AsyncBridgeError::ShutdownFailed))
        .expect_err("both phases must fail");

    assert!(matches!(
        error,
        GatewayRuntimeError::ServeAndShutdown {
            serve: GatewayServeError::DrainTimedOut { timeout },
            shutdown: AsyncBridgeError::ShutdownFailed,
        } if timeout == Duration::from_millis(25)
    ));
}

#[tokio::test]
async fn one_failed_signal_source_still_waits_for_the_other_source() {
    let result = wait_for_two_signal_sources(
        "first",
        std::future::ready(SignalWaitOutcome::Failed("unavailable".to_string())),
        "second",
        std::future::ready(SignalWaitOutcome::Received),
    )
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn all_failed_signal_sources_return_a_typed_error() {
    let error = wait_for_two_signal_sources(
        "first",
        std::future::ready(SignalWaitOutcome::Failed("first failed".to_string())),
        "second",
        std::future::ready(SignalWaitOutcome::Failed("second failed".to_string())),
    )
    .await
    .expect_err("all signal sources must fail");

    let details = error.to_string();
    assert!(details.contains("first failed"));
    assert!(details.contains("second failed"));
}

#[tokio::test]
async fn invalid_config_fails_before_bind_and_still_runs_cleanup() {
    let cleanup_called = Arc::new(AtomicBool::new(false));
    let cleanup_marker = cleanup_called.clone();
    let config_error = GatewayConfigError::InvalidDrainTimeoutSeconds {
        value: "invalid".to_string(),
    };

    let result = tokio::time::timeout(
        Duration::from_secs(1),
        run_with_runtime_cleanup(
            serve_router_with_config(
                "not-a-valid-socket-address",
                "gateway-invalid-config-test",
                Router::new(),
                Err(config_error),
                std::future::pending::<Result<(), GatewaySignalError>>(),
            ),
            move || {
                cleanup_marker.store(true, Ordering::SeqCst);
                Ok(())
            },
        ),
    )
    .await
    .expect("invalid config must fail without waiting for a signal");

    assert!(matches!(
        result,
        Err(GatewayRuntimeError::Serve(GatewayServeError::Config(
            GatewayConfigError::InvalidDrainTimeoutSeconds { value }
        ))) if value == "invalid"
    ));
    assert!(cleanup_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn shutdown_without_active_connections_drains_cleanly() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");

    let result = tokio::time::timeout(
        Duration::from_secs(1),
        serve_listener_with_drain_timeout(
            listener,
            Router::new(),
            std::future::ready(Ok(())),
            Duration::from_millis(50),
        ),
    )
    .await
    .expect("empty gateway drain must complete");

    assert!(result.is_ok());
}

#[tokio::test]
async fn connection_admission_never_exceeds_the_configured_limit() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let listen_addr = listener.local_addr().expect("read listener address");
    let entered = Arc::new(Notify::new());
    let handler_dropped = Arc::new(AtomicBool::new(false));
    let router = Router::new()
        .route("/pending", get(never_finishes))
        .with_state(PendingHandlerState {
            entered: entered.clone(),
            dropped: handler_dropped,
        });
    let (signal_tx, signal_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(serve_listener_with_limits(
        listener,
        router,
        async move {
            signal_rx.await.expect("test signal sender must stay alive");
            Ok(())
        },
        Duration::from_millis(25),
        1,
        Duration::from_secs(10),
    ));

    let first_client = open_pending_request(listen_addr).await;
    tokio::time::timeout(Duration::from_secs(1), entered.notified())
        .await
        .expect("first handler must start");
    let second_client = open_pending_request(listen_addr).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(50), entered.notified())
            .await
            .is_err(),
        "a second handler started above the configured connection limit"
    );

    signal_tx.send(()).expect("send shutdown signal");
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(1), server)
            .await
            .expect("bounded server shutdown")
            .expect("join server task"),
        Err(GatewayServeError::DrainTimedOut { timeout })
            if timeout == Duration::from_millis(25)
    ));

    drop(first_client);
    drop(second_client);
}

#[tokio::test]
async fn partial_request_headers_release_the_connection_slot_after_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let listen_addr = listener.local_addr().expect("read listener address");
    let handler_invoked = Arc::new(AtomicBool::new(false));
    let route_invoked = handler_invoked.clone();
    let router = Router::new().route(
        "/",
        get(move || {
            let route_invoked = route_invoked.clone();
            async move {
                route_invoked.store(true, Ordering::Release);
                "unexpected"
            }
        }),
    );
    let (signal_tx, signal_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(serve_listener_with_limits(
        listener,
        router,
        async move {
            signal_rx.await.expect("test signal sender must stay alive");
            Ok(())
        },
        Duration::from_millis(50),
        1,
        Duration::from_millis(25),
    ));

    let started = Instant::now();
    tokio::task::spawn_blocking(move || {
        let mut stream = std::net::TcpStream::connect(listen_addr).expect("connect test client");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("set client read timeout");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost:")
            .expect("write partial request headers");
        let mut buffer = [0_u8; 256];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::UnexpectedEof
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("partial header connection did not close: {error}"),
            }
        }
    })
    .await
    .expect("join partial header client");

    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(!handler_invoked.load(Ordering::Acquire));
    signal_tx.send(()).expect("send shutdown signal");
    tokio::time::timeout(Duration::from_secs(1), server)
        .await
        .expect("bounded server shutdown")
        .expect("join server task")
        .expect("partial header connection must be reaped cleanly");
}

#[tokio::test]
async fn signal_failure_reaps_pending_handlers_before_runtime_cleanup() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let listen_addr = listener.local_addr().expect("read listener address");
    let entered = Arc::new(Notify::new());
    let handler_dropped = Arc::new(AtomicBool::new(false));
    let router = Router::new()
        .route("/pending", get(never_finishes))
        .with_state(PendingHandlerState {
            entered: entered.clone(),
            dropped: handler_dropped.clone(),
        });
    let (signal_tx, signal_rx) = oneshot::channel::<Result<(), GatewaySignalError>>();
    let cleanup_saw_handler_dropped = Arc::new(AtomicBool::new(false));
    let cleanup_marker = cleanup_saw_handler_dropped.clone();
    let cleanup_handler_state = handler_dropped.clone();
    let server = tokio::spawn(run_with_runtime_cleanup(
        serve_listener_with_drain_timeout(
            listener,
            router,
            async move {
                signal_rx
                    .await
                    .expect("signal result sender must stay alive")
            },
            Duration::from_millis(50),
        ),
        move || {
            cleanup_marker.store(
                cleanup_handler_state.load(Ordering::Acquire),
                Ordering::Release,
            );
            Ok(())
        },
    ));

    let client = open_pending_request(listen_addr).await;
    tokio::time::timeout(Duration::from_secs(1), entered.notified())
        .await
        .expect("pending handler must start");
    signal_tx
        .send(Err(GatewaySignalError::all_handlers_unavailable(
            "test signal failure".to_string(),
        )))
        .expect("send signal failure");

    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(1), server)
            .await
            .expect("signal failure cleanup must be bounded")
            .expect("join gateway task"),
        Err(GatewayRuntimeError::Serve(GatewayServeError::Signal(_)))
    ));
    assert!(handler_dropped.load(Ordering::Acquire));
    assert!(cleanup_saw_handler_dropped.load(Ordering::Acquire));
    drop(client);
}

async fn open_pending_request(listen_addr: std::net::SocketAddr) -> std::net::TcpStream {
    tokio::task::spawn_blocking(move || {
        let mut stream = std::net::TcpStream::connect(listen_addr).expect("connect test client");
        stream
            .write_all(b"GET /pending HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .expect("write test request");
        stream
    })
    .await
    .expect("join test client")
}

#[tokio::test]
async fn drain_timeout_starts_at_signal_and_cleanup_still_runs() {
    let drain_timeout = Duration::from_millis(50);
    let wall_clock_started = Instant::now();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let listen_addr = listener.local_addr().expect("read test listener address");
    let entered = Arc::new(Notify::new());
    let handler_dropped = Arc::new(AtomicBool::new(false));
    let router = Router::new()
        .route("/pending", get(never_finishes))
        .with_state(PendingHandlerState {
            entered: entered.clone(),
            dropped: handler_dropped.clone(),
        });
    let (signal_tx, signal_rx) = oneshot::channel::<()>();
    let cleanup_called = Arc::new(AtomicBool::new(false));
    let cleanup_marker = cleanup_called.clone();
    let cleanup_saw_handler_dropped = Arc::new(AtomicBool::new(false));
    let cleanup_handler_marker = cleanup_saw_handler_dropped.clone();
    let cleanup_handler_state = handler_dropped.clone();

    let server = tokio::spawn(run_with_runtime_cleanup(
        serve_listener_with_drain_timeout(
            listener,
            router,
            async move {
                signal_rx.await.expect("test signal sender must stay alive");
                Ok(())
            },
            drain_timeout,
        ),
        move || {
            cleanup_handler_marker.store(
                cleanup_handler_state.load(Ordering::Acquire),
                Ordering::Release,
            );
            cleanup_marker.store(true, Ordering::SeqCst);
            Ok(())
        },
    ));

    let client = open_pending_request(listen_addr).await;

    tokio::time::timeout(Duration::from_secs(1), entered.notified())
        .await
        .expect("pending handler must start");
    tokio::time::sleep(drain_timeout + Duration::from_millis(25)).await;
    assert!(
        !server.is_finished(),
        "drain timeout must not start before the shutdown signal"
    );

    let signal_started = Instant::now();
    signal_tx.send(()).expect("send test shutdown signal");
    let result = tokio::time::timeout(Duration::from_secs(1), server)
        .await
        .expect("gateway drain must have a hard wall-clock bound")
        .expect("join gateway task");
    let elapsed_after_signal = signal_started.elapsed();

    assert!(matches!(
        result,
        Err(GatewayRuntimeError::Serve(
            GatewayServeError::DrainTimedOut { timeout }
        )) if timeout == drain_timeout
    ));
    assert!(cleanup_called.load(Ordering::SeqCst));
    assert!(
        handler_dropped.load(Ordering::Acquire),
        "drain timeout must abort and reap the pending handler"
    );
    assert!(
        cleanup_saw_handler_dropped.load(Ordering::Acquire),
        "runtime cleanup must run only after pending handlers are dropped"
    );
    assert!(elapsed_after_signal >= drain_timeout);
    assert!(elapsed_after_signal < Duration::from_millis(500));
    assert!(wall_clock_started.elapsed() < Duration::from_secs(2));

    drop(client);
}
