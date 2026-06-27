use axum::Router;
use sdkwork_knowledgebase_observability::{tracing_support, wrap_router_with_metrics};

pub fn init_tracing(service_name: &str) {
    tracing_support::init_tracing_from_env(service_name);
}

pub async fn serve_router(listen_addr: &str, service_name: &str, router: Router) {
    init_tracing(service_name);
    let router = wrap_router_with_metrics(router);

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("bind knowledgebase api listener");
    tracing::info!(%listen_addr, service = service_name, "listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve knowledgebase api");
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
