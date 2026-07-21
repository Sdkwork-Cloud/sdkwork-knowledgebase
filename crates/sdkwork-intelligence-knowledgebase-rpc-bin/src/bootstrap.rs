use std::sync::Arc;

use sdkwork_intelligence_knowledgebase_rpc::GroupKnowledgeSpaceLifecycleRpcService;
use sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::group_knowledge_space_lifecycle_service_server::GroupKnowledgeSpaceLifecycleServiceServer;
use sdkwork_rpc_server::{apply_server_tls, serve_with_graceful_shutdown, wait_for_ctrl_c};
use thiserror::Error;
use tonic::{service::interceptor::InterceptedService, transport::Server};

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
        config.drive_storage_root.clone(),
        config.operator_id.clone(),
        config.system_actor_id,
    )
    .await?;
    runtime.readiness_check().await?;

    let lifecycle_service = GroupKnowledgeSpaceLifecycleRpcService::new(Arc::new(runtime));
    let lifecycle_server = GroupKnowledgeSpaceLifecycleServiceServer::with_interceptor(
        lifecycle_service,
        security.interceptor(),
    );

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GroupKnowledgeSpaceLifecycleServiceServer<
            GroupKnowledgeSpaceLifecycleRpcService,
        >>()
        .await;
    let secured_health_service = InterceptedService::new(health_service, security.interceptor());

    let mut server = apply_server_tls(Server::builder(), &config.tls)?;
    let router = server
        .add_service(lifecycle_server)
        .add_service(secured_health_service);
    let bind_addr = config.bind_addr.to_string();
    serve_with_graceful_shutdown(router, bind_addr.as_str(), wait_for_ctrl_c()).await?;
    Ok(())
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
