use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sdkwork_agent_kernel::{SecretAccessPurpose, SecretAccessRequest, SecretError, SecretProvider};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::ResolvedKnowledgeEngineProviderCredential;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_credential_resolver::{
    KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialAccessContext,
    KnowledgeEngineProviderCredentialError, KnowledgeEngineProviderCredentialResolver,
};
use sdkwork_utils_rust::is_blank;
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;
use zeroize::Zeroizing;

use crate::config::{
    KnowledgebaseProviderCredentialResolverConfig,
    KnowledgebaseProviderCredentialResolverConfigurationError,
    KNOWLEDGEBASE_PROVIDER_SECRET_ENV_PREFIX,
};

const MANAGED_SECRET_LOCATOR_PREFIX: &str = "secret://knowledgebase/provider/";
const MANAGED_SECRET_REQUESTER: &str = "sdkwork-knowledgebase-provider-binding";
const EXTERNAL_IMPLEMENTATION_PREFIX: &str = "engine.knowledge.external.";

enum ResolverSource {
    Local,
    Managed {
        provider: Arc<dyn SecretProvider>,
        concurrency: Arc<Semaphore>,
    },
}

pub struct KnowledgebaseProviderCredentialResolver {
    config: KnowledgebaseProviderCredentialResolverConfig,
    source: ResolverSource,
}

impl std::fmt::Debug for KnowledgebaseProviderCredentialResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("KnowledgebaseProviderCredentialResolver")
            .field("environment", &self.config.environment)
            .field("local_secret_root", &self.config.local_secret_root)
            .field("max_credential_bytes", &self.config.max_credential_bytes)
            .field(
                "max_managed_resolution_duration",
                &self.config.max_managed_resolution_duration,
            )
            .field(
                "source",
                &match self.source {
                    ResolverSource::Local => "local",
                    ResolverSource::Managed { .. } => "managed",
                },
            )
            .field(
                "max_managed_concurrency",
                &self.config.max_managed_concurrency,
            )
            .finish()
    }
}

impl KnowledgebaseProviderCredentialResolver {
    pub fn local(
        config: KnowledgebaseProviderCredentialResolverConfig,
    ) -> Result<Self, KnowledgebaseProviderCredentialResolverConfigurationError> {
        if !config.environment.allows_local_sources() {
            return Err(
                KnowledgebaseProviderCredentialResolverConfigurationError::ManagedSourceRequired,
            );
        }
        Ok(Self {
            config,
            source: ResolverSource::Local,
        })
    }

    pub fn managed(
        config: KnowledgebaseProviderCredentialResolverConfig,
        provider: Arc<dyn SecretProvider>,
    ) -> Result<Self, KnowledgebaseProviderCredentialResolverConfigurationError> {
        if !config.environment.requires_managed_source() {
            return Err(
                KnowledgebaseProviderCredentialResolverConfigurationError::LocalSourceRequired,
            );
        }
        Ok(Self {
            source: ResolverSource::Managed {
                provider,
                concurrency: Arc::new(Semaphore::new(config.max_managed_concurrency)),
            },
            config,
        })
    }

    fn validate_access_context(
        &self,
        context: &KnowledgeEngineProviderCredentialAccessContext,
        reference: &ResolvedKnowledgeEngineProviderCredential,
    ) -> Result<(), KnowledgeEngineProviderCredentialError> {
        if context.tenant_id == 0
            || context.space_id == 0
            || context.binding_id == 0
            || context.credential_reference_id == 0
            || context.credential_reference_id != reference.credential_reference_id
            || context.credential_reference_version != reference.version
            || context.implementation_id != reference.implementation_id
            || is_blank(Some(context.actor_id.as_str()))
            || is_blank(Some(context.trace_id.as_str()))
        {
            return Err(KnowledgeEngineProviderCredentialError::AccessDenied);
        }
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| KnowledgeEngineProviderCredentialError::Internal)?
            .as_millis();
        if u128::from(context.deadline_unix_ms) <= now_ms {
            return Err(KnowledgeEngineProviderCredentialError::Unavailable);
        }
        Ok(())
    }

    fn validate_local_locator(
        &self,
        implementation_id: &str,
        locator: &str,
    ) -> Result<LocalCredentialLocator, KnowledgeEngineProviderCredentialError> {
        let provider_code = provider_code(implementation_id)?;
        if let Some(variable) = locator.strip_prefix("env://") {
            validate_environment_variable(provider_code, variable)?;
            return Ok(LocalCredentialLocator::Environment(variable.to_string()));
        }
        if locator.starts_with("file://") {
            let root = self
                .config
                .local_secret_root
                .as_ref()
                .ok_or(KnowledgeEngineProviderCredentialError::InvalidReference)?;
            let path = parse_file_locator(locator)?;
            let approved_root = root.join(provider_code);
            validate_lexical_containment(&approved_root, &path)?;
            return Ok(LocalCredentialLocator::File {
                approved_root,
                path,
            });
        }
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    }

    async fn resolve_local(
        &self,
        locator: LocalCredentialLocator,
    ) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
        match locator {
            LocalCredentialLocator::Environment(variable) => {
                let value = std::env::var(variable)
                    .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
                credential_from_bounded_string(value, self.config.max_credential_bytes)
            }
            LocalCredentialLocator::File {
                approved_root,
                path,
            } => {
                resolve_bounded_file(&approved_root, &path, self.config.max_credential_bytes).await
            }
        }
    }

    async fn resolve_managed(
        &self,
        provider: Arc<dyn SecretProvider>,
        concurrency: Arc<Semaphore>,
        context: &KnowledgeEngineProviderCredentialAccessContext,
        secret_id: String,
    ) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
        let request = managed_access_request(context, secret_id);
        let resolution_started = Instant::now();
        let resolution_bound = remaining_deadline(context.deadline_unix_ms)?
            .min(self.config.max_managed_resolution_duration);
        let permit = tokio::time::timeout(resolution_bound, concurrency.acquire_owned())
            .await
            .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?
            .map_err(|_| KnowledgeEngineProviderCredentialError::Internal)?;
        let remaining = resolution_bound.saturating_sub(resolution_started.elapsed());
        if remaining.is_zero() {
            return Err(KnowledgeEngineProviderCredentialError::Unavailable);
        }
        let result = tokio::time::timeout(
            remaining,
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                provider.access_secret(request)
            }),
        )
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?
        .map_err(|_| KnowledgeEngineProviderCredentialError::Internal)?
        .map_err(map_secret_error)?;

        if !result.granted {
            return Err(KnowledgeEngineProviderCredentialError::AccessDenied);
        }
        if is_blank(Some(result.audit_record_id.as_str())) {
            return Err(KnowledgeEngineProviderCredentialError::Internal);
        }
        let value = result
            .value
            .ok_or(KnowledgeEngineProviderCredentialError::Internal)?;
        credential_from_bounded_string(value, self.config.max_credential_bytes)
    }

    fn record_resolution(
        &self,
        context: &KnowledgeEngineProviderCredentialAccessContext,
        outcome: &str,
    ) {
        tracing::info!(
            security_event = "knowledge.provider_credential.access",
            environment = self.config.environment.as_str(),
            tenant_id = context.tenant_id,
            organization_id = context.organization_id,
            space_id = context.space_id,
            binding_id = context.binding_id,
            credential_reference_version = context.credential_reference_version,
            implementation_id = %context.implementation_id,
            actor_id = %context.actor_id,
            operation = %context.operation,
            trace_id = %context.trace_id,
            outcome,
            "Knowledgebase Provider credential access completed"
        );
    }
}

#[async_trait]
impl KnowledgeEngineProviderCredentialResolver for KnowledgebaseProviderCredentialResolver {
    fn validate_reference_locator(
        &self,
        implementation_id: &str,
        reference_locator: &str,
    ) -> Result<(), KnowledgeEngineProviderCredentialError> {
        match &self.source {
            ResolverSource::Local => self
                .validate_local_locator(implementation_id, reference_locator)
                .map(|_| ()),
            ResolverSource::Managed { .. } => {
                validate_managed_secret_locator(implementation_id, reference_locator).map(|_| ())
            }
        }
    }

    async fn resolve(
        &self,
        context: &KnowledgeEngineProviderCredentialAccessContext,
        reference: &ResolvedKnowledgeEngineProviderCredential,
    ) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
        if let Err(error) = self.validate_access_context(context, reference) {
            self.record_resolution(context, resolution_outcome(&Err(error.clone())));
            return Err(error);
        }
        let result = match &self.source {
            ResolverSource::Local => {
                match self.validate_local_locator(
                    &context.implementation_id,
                    &reference.reference_locator,
                ) {
                    Ok(locator) => self.resolve_local(locator).await,
                    Err(error) => Err(error),
                }
            }
            ResolverSource::Managed {
                provider,
                concurrency,
            } => {
                match validate_managed_secret_locator(
                    &context.implementation_id,
                    &reference.reference_locator,
                ) {
                    Ok(secret_id) => {
                        self.resolve_managed(
                            provider.clone(),
                            concurrency.clone(),
                            context,
                            secret_id,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                }
            }
        };
        self.record_resolution(context, resolution_outcome(&result));
        result
    }
}

enum LocalCredentialLocator {
    Environment(String),
    File {
        approved_root: PathBuf,
        path: PathBuf,
    },
}

fn validate_environment_variable(
    provider_code: &str,
    variable: &str,
) -> Result<(), KnowledgeEngineProviderCredentialError> {
    let provider_prefix = format!(
        "{KNOWLEDGEBASE_PROVIDER_SECRET_ENV_PREFIX}{}_",
        provider_code.replace('-', "_").to_ascii_uppercase()
    );
    let suffix = variable.strip_prefix(&provider_prefix);
    if variable.len() > 128
        || !variable
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        || suffix.is_none_or(|suffix| {
            suffix.is_empty() || !suffix.bytes().any(|byte| byte.is_ascii_alphanumeric())
        })
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    Ok(())
}

fn parse_file_locator(locator: &str) -> Result<PathBuf, KnowledgeEngineProviderCredentialError> {
    let parsed = url::Url::parse(locator)
        .map_err(|_| KnowledgeEngineProviderCredentialError::InvalidReference)?;
    if parsed.scheme() != "file"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed
            .host_str()
            .is_some_and(|host| !host.eq_ignore_ascii_case("localhost"))
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    parsed
        .to_file_path()
        .map_err(|_| KnowledgeEngineProviderCredentialError::InvalidReference)
}

fn validate_lexical_containment(
    root: &Path,
    path: &Path,
) -> Result<(), KnowledgeEngineProviderCredentialError> {
    if !root.is_absolute() || !path.is_absolute() {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        || !path.starts_with(root)
    {
        return Err(KnowledgeEngineProviderCredentialError::AccessDenied);
    }
    Ok(())
}

async fn resolve_bounded_file(
    root: &Path,
    path: &Path,
    max_bytes: usize,
) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
    let canonical_root = tokio::fs::canonicalize(root)
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
    let canonical_path = tokio::fs::canonicalize(path)
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(KnowledgeEngineProviderCredentialError::AccessDenied);
    }

    let file = tokio::fs::File::open(canonical_path)
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
    let metadata = file
        .metadata()
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    if metadata.len() > max_bytes as u64 {
        return Err(KnowledgeEngineProviderCredentialError::ResponseTooLarge);
    }

    let mut bytes = Zeroizing::new(Vec::with_capacity(max_bytes.min(metadata.len() as usize)));
    file.take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| KnowledgeEngineProviderCredentialError::Unavailable)?;
    if bytes.len() > max_bytes {
        return Err(KnowledgeEngineProviderCredentialError::ResponseTooLarge);
    }
    let value = std::str::from_utf8(&bytes)
        .map_err(|_| KnowledgeEngineProviderCredentialError::InvalidReference)?
        .to_string();
    credential_from_bounded_string(value, max_bytes)
}

fn validate_managed_secret_locator(
    implementation_id: &str,
    locator: &str,
) -> Result<String, KnowledgeEngineProviderCredentialError> {
    let expected_provider_code = provider_code(implementation_id)?;
    let path = locator
        .strip_prefix(MANAGED_SECRET_LOCATOR_PREFIX)
        .ok_or(KnowledgeEngineProviderCredentialError::InvalidReference)?;
    if path.is_empty()
        || path.len() > 256
        || path.starts_with('/')
        || path.ends_with('/')
        || path.split('/').any(|segment| {
            segment.is_empty()
                || segment == "."
                || segment == ".."
                || !segment.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.'
                })
        })
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    if path.split('/').next() != Some(expected_provider_code) {
        return Err(KnowledgeEngineProviderCredentialError::AccessDenied);
    }
    Ok(format!("knowledgebase/provider/{path}"))
}

fn provider_code(implementation_id: &str) -> Result<&str, KnowledgeEngineProviderCredentialError> {
    let code = implementation_id
        .strip_prefix(EXTERNAL_IMPLEMENTATION_PREFIX)
        .ok_or(KnowledgeEngineProviderCredentialError::InvalidReference)?;
    if code.is_empty()
        || code.len() > 64
        || code.starts_with('-')
        || code.ends_with('-')
        || !code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference);
    }
    Ok(code)
}

fn managed_access_request(
    context: &KnowledgeEngineProviderCredentialAccessContext,
    secret_id: String,
) -> SecretAccessRequest {
    SecretAccessRequest::new(secret_id, MANAGED_SECRET_REQUESTER)
        .with_purpose(SecretAccessPurpose::Read)
        .with_context("tenant_id", context.tenant_id.to_string())
        .with_context("organization_id", context.organization_id.to_string())
        .with_context("space_id", context.space_id.to_string())
        .with_context("binding_id", context.binding_id.to_string())
        .with_context(
            "credential_reference_id",
            context.credential_reference_id.to_string(),
        )
        .with_context(
            "credential_reference_version",
            context.credential_reference_version.to_string(),
        )
        .with_context("implementation_id", context.implementation_id.clone())
        .with_context("actor_id", context.actor_id.clone())
        .with_context("operation", context.operation.to_string())
        .with_context("trace_id", context.trace_id.clone())
        .with_context("deadline_unix_ms", context.deadline_unix_ms.to_string())
}

fn remaining_deadline(
    deadline_unix_ms: u64,
) -> Result<Duration, KnowledgeEngineProviderCredentialError> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| KnowledgeEngineProviderCredentialError::Internal)?
        .as_millis();
    let remaining_ms = u128::from(deadline_unix_ms)
        .checked_sub(now_ms)
        .filter(|value| *value > 0)
        .ok_or(KnowledgeEngineProviderCredentialError::Unavailable)?;
    Ok(Duration::from_millis(
        u64::try_from(remaining_ms).unwrap_or(u64::MAX),
    ))
}

fn credential_from_bounded_string(
    value: String,
    max_bytes: usize,
) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
    let value = Zeroizing::new(value);
    if value.len() > max_bytes {
        return Err(KnowledgeEngineProviderCredentialError::ResponseTooLarge);
    }
    KnowledgeEngineProviderCredential::new(value.trim().to_string())
}

fn map_secret_error(error: SecretError) -> KnowledgeEngineProviderCredentialError {
    match error {
        SecretError::AccessDenied(_) => KnowledgeEngineProviderCredentialError::AccessDenied,
        SecretError::NotFound(_) | SecretError::Expired(_) | SecretError::ProviderUnavailable => {
            KnowledgeEngineProviderCredentialError::Unavailable
        }
        SecretError::EncryptionFailed(_)
        | SecretError::DecryptionFailed(_)
        | SecretError::InvalidRequest(_)
        | SecretError::StorageError(_) => KnowledgeEngineProviderCredentialError::Internal,
    }
}

fn resolution_outcome(
    result: &Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError>,
) -> &'static str {
    match result {
        Ok(_) => "granted",
        Err(KnowledgeEngineProviderCredentialError::InvalidReference) => "invalid_reference",
        Err(KnowledgeEngineProviderCredentialError::AccessDenied) => "access_denied",
        Err(KnowledgeEngineProviderCredentialError::Unavailable) => "unavailable",
        Err(KnowledgeEngineProviderCredentialError::ResponseTooLarge) => "response_too_large",
        Err(KnowledgeEngineProviderCredentialError::Internal) => "internal",
    }
}
