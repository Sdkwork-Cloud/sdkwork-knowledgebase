use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use sdkwork_intelligence_knowledgebase_rpc::GroupKnowledgeSpaceLifecycleRpcService;
use sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::group_knowledge_space_lifecycle_service_server::GroupKnowledgeSpaceLifecycleServiceServer;
use sdkwork_rpc_server::{apply_server_tls, serve_with_graceful_shutdown, wait_for_ctrl_c};
use thiserror::Error;
use tonic::{
    service::interceptor::InterceptedService,
    transport::Server,
};
use tonic_health::{server::HealthReporter, ServingStatus};

use crate::{
    config::{
        GroupKnowledgeSpaceLifecycleRpcHostConfig, GroupKnowledgeSpaceLifecycleRpcHostConfigError,
    },
    runtime::{
        KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime,
        KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError,
    },
};

/// Validates all private configuration and runtime dependencies before binding the internal RPC
/// port. A malformed certificate path, signing key, database, or Drive dependency cannot leave a
/// partially initialized listener accepting traffic.
pub async fn run_group_knowledge_space_lifecycle_rpc_from_env(
) -> Result<(), GroupKnowledgeSpaceLifecycleRpcHostError> {
    let config = GroupKnowledgeSpaceLifecycleRpcHostConfig::from_env()?;
    let security = config.internal_service_security()?;
    security.validate_mtls_listener(&config.tls)?;

    let runtime = KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime::connect(
        config.database_url.as_str(),
        config.drive_storage.clone(),
        config.operator_id.clone(),
        config.system_actor_id,
    )
    .await?;
    runtime.readiness_check().await?;

    let health_runtime = runtime.clone();
    let lifecycle_service = GroupKnowledgeSpaceLifecycleRpcService::new(Arc::new(runtime));
    let lifecycle_server = GroupKnowledgeSpaceLifecycleServiceServer::new(lifecycle_service)
        .max_decoding_message_size(config.transport.max_decoding_message_bytes)
        .max_encoding_message_size(config.transport.max_encoding_message_bytes);
    let lifecycle_server = InterceptedService::new(lifecycle_server, security.interceptor());

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GroupKnowledgeSpaceLifecycleServiceServer<
            GroupKnowledgeSpaceLifecycleRpcService,
        >>()
        .await;
    health_reporter
        .set_service_status("", ServingStatus::Serving)
        .await;
    let secured_health_service = InterceptedService::new(health_service, security.interceptor());

    let server = Server::builder()
        .concurrency_limit_per_connection(config.transport.max_concurrent_requests_per_connection)
        .timeout(config.transport.request_timeout)
        .http2_keepalive_interval(Some(config.transport.http2_keepalive_interval))
        .http2_keepalive_timeout(Some(config.transport.http2_keepalive_timeout))
        .max_connection_age(config.transport.max_connection_age)
        .max_connection_age_grace(config.transport.max_connection_age_grace)
        .tcp_keepalive(Some(config.transport.tcp_keepalive));
    let mut server = apply_server_tls(server, &config.tls)?;
    let router = server
        .add_service(lifecycle_server)
        .add_service(secured_health_service);
    let bind_addr = config.bind_addr.to_string();
    let shutting_down = Arc::new(AtomicBool::new(false));
    let readiness_task = tokio::spawn(monitor_runtime_readiness(
        health_runtime,
        health_reporter.clone(),
        shutting_down.clone(),
    ));
    let result = serve_with_bounded_graceful_shutdown(
        router,
        bind_addr.as_str(),
        health_reporter,
        shutting_down,
        config.transport.drain_timeout,
    )
    .await;
    readiness_task.abort();
    let _ = readiness_task.await;
    result?;
    Ok(())
}

async fn monitor_runtime_readiness(
    runtime: KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime,
    health_reporter: HealthReporter,
    shutting_down: Arc<AtomicBool>,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_ready = true;
    loop {
        interval.tick().await;
        if shutting_down.load(Ordering::Acquire) {
            return;
        }
        let ready = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            runtime.readiness_check(),
        )
        .await
        .is_ok_and(|result| result.is_ok());
        if ready == last_ready {
            continue;
        }
        last_ready = ready;
        if ready {
            health_reporter
                .set_serving::<GroupKnowledgeSpaceLifecycleServiceServer<
                    GroupKnowledgeSpaceLifecycleRpcService,
                >>()
                .await;
            health_reporter
                .set_service_status("", ServingStatus::Serving)
                .await;
            tracing::info!("knowledgebase RPC readiness dependencies recovered");
        } else {
            health_reporter
                .set_not_serving::<GroupKnowledgeSpaceLifecycleServiceServer<
                    GroupKnowledgeSpaceLifecycleRpcService,
                >>()
                .await;
            health_reporter
                .set_service_status("", ServingStatus::NotServing)
                .await;
            tracing::warn!("knowledgebase RPC readiness dependency is unavailable");
        }
    }
}

async fn serve_with_bounded_graceful_shutdown(
    router: tonic::transport::server::Router,
    bind_addr: &str,
    health_reporter: HealthReporter,
    shutting_down: Arc<AtomicBool>,
    drain_timeout: std::time::Duration,
) -> Result<(), sdkwork_rpc_server::ServeError> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let serve = serve_with_graceful_shutdown(router, bind_addr, async move {
        let _ = shutdown_rx.await;
    });
    tokio::pin!(serve);

    tokio::select! {
        result = &mut serve => result,
        _ = wait_for_ctrl_c() => {
            shutting_down.store(true, Ordering::Release);
            health_reporter
                .set_not_serving::<GroupKnowledgeSpaceLifecycleServiceServer<
                    GroupKnowledgeSpaceLifecycleRpcService,
                >>()
                .await;
            health_reporter
                .set_service_status("", ServingStatus::NotServing)
                .await;
            let _ = shutdown_tx.send(());
            match tokio::time::timeout(drain_timeout, &mut serve).await {
                Ok(result) => result,
                Err(_) => {
                    tracing::warn!(
                        drain_timeout_ms = u64::try_from(drain_timeout.as_millis()).unwrap_or(u64::MAX),
                        "knowledgebase RPC drain exceeded its deadline; cancelling remaining calls"
                    );
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum GroupKnowledgeSpaceLifecycleRpcHostError {
    #[error(transparent)]
    Config(#[from] GroupKnowledgeSpaceLifecycleRpcHostConfigError),
    #[error(transparent)]
    Runtime(#[from] KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError),
    #[error(transparent)]
    Server(#[from] sdkwork_rpc_server::ServeError),
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, net::TcpListener, sync::OnceLock};
    use tokio::sync::Mutex;

    use super::*;

    const ENVIRONMENT_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ENVIRONMENT";
    const BIND_ADDR_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_BIND_ADDR";

    fn environment_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvironmentVariableGuard {
        key: &'static str,
        original_value: Option<OsString>,
    }

    impl EnvironmentVariableGuard {
        fn remove(key: &'static str) -> Self {
            let original_value = std::env::var_os(key);
            std::env::remove_var(key);
            Self {
                key,
                original_value,
            }
        }

        fn set(key: &'static str, value: impl Into<OsString>) -> Self {
            let original_value = std::env::var_os(key);
            std::env::set_var(key, value.into());
            Self {
                key,
                original_value,
            }
        }
    }

    impl Drop for EnvironmentVariableGuard {
        fn drop(&mut self) {
            if let Some(value) = self.original_value.as_ref() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[tokio::test]
    async fn invalid_preflight_never_claims_the_requested_listener_port() {
        let _lock = environment_lock().lock().await;
        let reserved = TcpListener::bind("127.0.0.1:0").expect("temporary listener");
        let address = reserved.local_addr().expect("temporary listener address");
        drop(reserved);

        let _bind_addr = EnvironmentVariableGuard::set(BIND_ADDR_ENV, address.to_string());
        let _environment = EnvironmentVariableGuard::remove(ENVIRONMENT_ENV);
        assert!(run_group_knowledge_space_lifecycle_rpc_from_env()
            .await
            .is_err());

        TcpListener::bind(address).expect("invalid preflight must leave the listener unbound");
    }
}
