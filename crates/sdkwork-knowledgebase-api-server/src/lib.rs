use axum::Router;
use sdkwork_knowledgebase_observability::wrap_router_with_metrics;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sdkwork_knowledgebase_api_server=debug".into()),
        )
        .init();
}

pub async fn serve_router(listen_addr: &str, service_name: &str, router: Router) {
    let router = wrap_router_with_metrics(router);
    init_tracing();

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .expect("bind knowledgebase api listener");
    tracing::info!(%listen_addr, service = service_name, "listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve knowledgebase api");
}

async fn shutdown_signal() {
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
