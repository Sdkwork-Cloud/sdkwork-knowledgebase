use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use sdkwork_rpc_framework_core::{
    RpcCallerContextSigningKey, RpcCallerContextVerifier, RpcFrameworkError,
    RpcServiceIdentityPolicy,
};
use sdkwork_rpc_server::{RpcInternalServiceSecurity, RpcServerTlsConfig};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

const ENVIRONMENT_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ENVIRONMENT";
const RPC_ENABLED_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_ENABLED";
const RPC_BIND_ADDR_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_BIND_ADDR";
const RPC_TLS_ENABLED_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_TLS_ENABLED";
const RPC_MTLS_ENABLED_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_MTLS_ENABLED";
const RPC_HEALTH_ENABLED_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_HEALTH_ENABLED";
const RPC_SERVER_CERT_PATH_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_SERVER_CERT_PATH";
const RPC_SERVER_KEY_PATH_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_SERVER_KEY_PATH";
const RPC_CLIENT_CA_CERTIFICATE_PATH_ENV: &str =
    "SDKWORK_KNOWLEDGEBASE_RPC_CLIENT_CA_CERTIFICATE_PATH";
const RPC_SPIFFE_TRUST_DOMAIN_ENV: &str = "SDKWORK_KNOWLEDGEBASE_RPC_SPIFFE_TRUST_DOMAIN";
const RPC_IM_CALLER_CONTEXT_SIGNING_KEY_ENV: &str =
    "SDKWORK_KNOWLEDGEBASE_RPC_IM_CALLER_CONTEXT_SIGNING_KEY";
const RPC_IM_CALLER_CONTEXT_SIGNING_KEY_FILE_ENV: &str =
    "SDKWORK_KNOWLEDGEBASE_RPC_IM_CALLER_CONTEXT_SIGNING_KEY_FILE";
const DATABASE_URL_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DATABASE_URL";
const DRIVE_STORAGE_ROOT_ENV: &str = "SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT";
const OPERATOR_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OPERATOR_ID";
const ACTOR_ID_ENV: &str = "SDKWORK_KNOWLEDGEBASE_ACTOR_ID";

const IM_SERVICE_ID: &str = "sdkwork-im";
const KNOWLEDGEBASE_SERVICE_ID: &str = "sdkwork-knowledgebase";
const DEFAULT_DRIVE_STORAGE_ROOT: &str = "data/drive-objects";

/// Bootstrap-owned private configuration for the IM-facing Knowledgebase RPC listener.
///
/// It deliberately does not support a plaintext or unsigned-local fallback. The lifecycle
/// surface changes Drive ACLs and is always authenticated as `sdkwork-im` over strict mTLS.
#[derive(Clone, Debug)]
pub struct GroupKnowledgeSpaceLifecycleRpcHostConfig {
    pub environment: RpcHostEnvironment,
    pub bind_addr: SocketAddr,
    pub tls: RpcServerTlsConfig,
    pub database_url: String,
    pub drive_storage_root: PathBuf,
    pub operator_id: String,
    pub system_actor_id: u64,
    caller_context_signing_key: RpcCallerContextSigningKey,
    spiffe_trust_domain: String,
}

impl GroupKnowledgeSpaceLifecycleRpcHostConfig {
    pub fn from_env() -> Result<Self, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
        let environment = RpcHostEnvironment::parse(&required_env(ENVIRONMENT_ENV)?)?;
        require_enabled(RPC_ENABLED_ENV)?;
        require_enabled(RPC_TLS_ENABLED_ENV)?;
        require_enabled(RPC_MTLS_ENABLED_ENV)?;
        require_enabled(RPC_HEALTH_ENABLED_ENV)?;

        let bind_addr = SocketAddr::from_str(&required_env(RPC_BIND_ADDR_ENV)?).map_err(|_| {
            GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                key: RPC_BIND_ADDR_ENV,
            }
        })?;
        let tls = RpcServerTlsConfig {
            server_cert_path: PathBuf::from(required_env(RPC_SERVER_CERT_PATH_ENV)?),
            server_key_path: PathBuf::from(required_env(RPC_SERVER_KEY_PATH_ENV)?),
            client_ca_certificate_path: Some(PathBuf::from(required_env(
                RPC_CLIENT_CA_CERTIFICATE_PATH_ENV,
            )?)),
            client_auth_optional: false,
        };
        let database_url = required_env(DATABASE_URL_ENV)?;
        let drive_storage_root = resolve_drive_storage_root_from_env(environment)?;
        let operator_id =
            configured_nonblank_text_or_default(OPERATOR_ID_ENV, KNOWLEDGEBASE_SERVICE_ID)?;
        if operator_id.len() > 256 {
            return Err(
                GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                    key: OPERATOR_ID_ENV,
                },
            );
        }
        let system_actor_id = sdkwork_knowledgebase_contract::parse_canonical_positive_signed_i64(
            &required_env(ACTOR_ID_ENV)?,
        )
        .map_err(|_| GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
            key: ACTOR_ID_ENV,
        })?;
        let spiffe_trust_domain = required_env(RPC_SPIFFE_TRUST_DOMAIN_ENV)?;
        let caller_context_signing_key = RpcCallerContextSigningKey::from_base64url(
            read_secret_env_or_file(
                RPC_IM_CALLER_CONTEXT_SIGNING_KEY_ENV,
                RPC_IM_CALLER_CONTEXT_SIGNING_KEY_FILE_ENV,
            )?
            .as_str(),
        )
        .map_err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::RpcFramework)?;
        let config = Self {
            environment,
            bind_addr,
            tls,
            database_url,
            drive_storage_root,
            operator_id,
            system_actor_id,
            caller_context_signing_key,
            spiffe_trust_domain,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn internal_service_security(
        &self,
    ) -> Result<RpcInternalServiceSecurity, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
        let identity_policy =
            RpcServiceIdentityPolicy::new(self.spiffe_trust_domain.as_str(), [IM_SERVICE_ID])
                .map_err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::RpcFramework)?;
        let caller_context_verifier = RpcCallerContextVerifier::new(
            KNOWLEDGEBASE_SERVICE_ID,
            [(IM_SERVICE_ID, self.caller_context_signing_key.clone())],
        )
        .map_err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::RpcFramework)?;
        Ok(RpcInternalServiceSecurity::new(
            identity_policy,
            Some(caller_context_verifier),
        ))
    }

    fn validate(&self) -> Result<(), GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
        if is_blank(Some(self.database_url.as_str()))
            || contains_control_character(&self.database_url)
        {
            return Err(
                GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                    key: DATABASE_URL_ENV,
                },
            );
        }
        let security = self.internal_service_security()?;
        security
            .validate_mtls_listener(&self.tls)
            .map_err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::RpcServer)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RpcHostEnvironment {
    Development,
    Test,
    Staging,
    Production,
}

impl RpcHostEnvironment {
    fn parse(value: &str) -> Result<Self, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
        match value {
            "development" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(
                GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                    key: ENVIRONMENT_ENV,
                },
            ),
        }
    }

    fn requires_explicit_persistent_storage(self) -> bool {
        matches!(self, Self::Staging | Self::Production)
    }
}

fn required_env(
    key: &'static str,
) -> Result<String, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    let value = std::env::var(key)
        .map_err(|_| GroupKnowledgeSpaceLifecycleRpcHostConfigError::Missing { key })?;
    if is_blank(Some(value.as_str())) || contains_control_character(&value) {
        return Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key });
    }
    Ok(value)
}

fn configured_nonblank_text_or_default(
    key: &'static str,
    default_value: &'static str,
) -> Result<String, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    let value = optional_env_text(key)?.unwrap_or_else(|| default_value.to_string());
    validate_configured_value(key, value)
}

fn resolve_drive_storage_root_from_env(
    environment: RpcHostEnvironment,
) -> Result<PathBuf, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    resolve_drive_storage_root(environment, optional_env_text(DRIVE_STORAGE_ROOT_ENV)?)
}

fn resolve_drive_storage_root(
    environment: RpcHostEnvironment,
    configured_value: Option<String>,
) -> Result<PathBuf, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    let value = match configured_value {
        Some(value) => validate_configured_value(DRIVE_STORAGE_ROOT_ENV, value)?,
        None if !environment.requires_explicit_persistent_storage() => {
            DEFAULT_DRIVE_STORAGE_ROOT.to_string()
        }
        None => {
            return Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::Missing {
                key: DRIVE_STORAGE_ROOT_ENV,
            });
        }
    };
    let path = PathBuf::from(value);
    if environment.requires_explicit_persistent_storage() && !path.is_absolute() {
        return Err(
            GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                key: DRIVE_STORAGE_ROOT_ENV,
            },
        );
    }
    Ok(path)
}

fn optional_env_text(
    key: &'static str,
) -> Result<Option<String>, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    std::env::var_os(key)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key })
        })
        .transpose()
}

fn validate_configured_value(
    key: &'static str,
    value: String,
) -> Result<String, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    if is_blank(Some(value.as_str())) || contains_control_character(&value) {
        return Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key });
    }
    Ok(value.trim().to_string())
}

fn require_enabled(
    key: &'static str,
) -> Result<(), GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    let value = required_env(key)?;
    match value.as_str() {
        "true" | "1" => Ok(()),
        "false" | "0" => Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::Disabled { key }),
        _ => Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key }),
    }
}

fn read_secret_env_or_file(
    value_key: &'static str,
    file_key: &'static str,
) -> Result<String, GroupKnowledgeSpaceLifecycleRpcHostConfigError> {
    let direct = std::env::var(value_key)
        .ok()
        .filter(|value| !is_blank(Some(value)));
    let file = std::env::var(file_key)
        .ok()
        .filter(|value| !is_blank(Some(value)));
    match (direct, file) {
        (Some(_), Some(_)) => Err(
            GroupKnowledgeSpaceLifecycleRpcHostConfigError::ConflictingSecretSources {
                value_key,
                file_key,
            },
        ),
        (Some(value), None) => Ok(value),
        (None, Some(path)) => std::fs::read_to_string(path)
            .map(|value| value.trim_end_matches(['\r', '\n']).to_string())
            .map_err(
                |_| GroupKnowledgeSpaceLifecycleRpcHostConfigError::UnreadableSecretFile {
                    key: file_key,
                },
            ),
        (None, None) => {
            Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::Missing { key: value_key })
        }
    }
}

fn contains_control_character(value: &str) -> bool {
    value.chars().any(char::is_control)
}

#[derive(Debug, Error)]
pub enum GroupKnowledgeSpaceLifecycleRpcHostConfigError {
    #[error("required private RPC configuration is missing: {key}")]
    Missing { key: &'static str },
    #[error("private RPC configuration has an invalid value: {key}")]
    InvalidValue { key: &'static str },
    #[error("the internal RPC listener requires enabled configuration: {key}")]
    Disabled { key: &'static str },
    #[error("private RPC signing key has conflicting environment and file sources")]
    ConflictingSecretSources {
        value_key: &'static str,
        file_key: &'static str,
    },
    #[error("private RPC signing-key file cannot be read: {key}")]
    UnreadableSecretFile { key: &'static str },
    #[error(transparent)]
    RpcFramework(#[from] RpcFrameworkError),
    #[error("internal RPC TLS configuration is invalid: {0}")]
    RpcServer(#[from] sdkwork_rpc_server::ServeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_disabled_and_invalid_server_values_before_bind() {
        assert!(matches!(
            validate_configured_value("TEST_ROOT", "default".to_string()),
            Ok(value) if value == "default"
        ));
        assert!(matches!(
            validate_configured_value("TEST_ROOT", "\n".to_string()),
            Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key: "TEST_ROOT" })
        ));
        assert!(matches!(
            validate_configured_value("TEST_ROOT", "  ".to_string()),
            Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue { key: "TEST_ROOT" })
        ));
        assert!(matches!(
            RpcHostEnvironment::parse("localhost"),
            Err(
                GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                    key: ENVIRONMENT_ENV
                }
            )
        ));
    }

    #[test]
    fn production_and_staging_require_an_explicit_absolute_drive_storage_root() {
        assert_eq!(
            resolve_drive_storage_root(RpcHostEnvironment::Development, None)
                .expect("development may use the local default"),
            PathBuf::from(DEFAULT_DRIVE_STORAGE_ROOT),
        );
        assert!(matches!(
            resolve_drive_storage_root(RpcHostEnvironment::Production, None),
            Err(GroupKnowledgeSpaceLifecycleRpcHostConfigError::Missing {
                key: DRIVE_STORAGE_ROOT_ENV
            })
        ));
        assert!(matches!(
            resolve_drive_storage_root(
                RpcHostEnvironment::Staging,
                Some("relative/drive-storage".to_string()),
            ),
            Err(
                GroupKnowledgeSpaceLifecycleRpcHostConfigError::InvalidValue {
                    key: DRIVE_STORAGE_ROOT_ENV
                }
            )
        ));
        let production_mount = std::env::temp_dir().join("sdkwork-knowledgebase-rpc-test");
        assert_eq!(
            resolve_drive_storage_root(
                RpcHostEnvironment::Production,
                Some(production_mount.to_string_lossy().into_owned()),
            )
            .expect("production accepts an explicitly configured persistent mount"),
            production_mount,
        );
    }
}
