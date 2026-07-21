use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_agent_kernel::{
    EncryptionAlgorithm, SecretAccessRequest, SecretAccessResult, SecretCreateRequest, SecretError,
    SecretMetadata, SecretProvider, SecretProviderHealth, SecretProviderManifest,
    SecretProviderStatus, SecretRotateRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::ResolvedKnowledgeEngineProviderCredential;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_credential_resolver::{
    KnowledgeEngineProviderCredentialAccessContext, KnowledgeEngineProviderCredentialError,
    KnowledgeEngineProviderCredentialResolver,
};
use sdkwork_knowledgebase_provider_secret_adapter::{
    KnowledgebaseProviderCredentialEnvironment, KnowledgebaseProviderCredentialResolver,
    KnowledgebaseProviderCredentialResolverConfig,
    KnowledgebaseProviderCredentialResolverConfigurationError,
    KNOWLEDGEBASE_PROVIDER_SECRET_ENV_PREFIX,
};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, Clone)]
enum SecretOutcome {
    Granted(String),
    Denied(String),
    NotFound(String),
}

#[derive(Debug)]
struct RecordingSecretProvider {
    outcome: Mutex<SecretOutcome>,
    requests: Mutex<Vec<SecretAccessRequest>>,
    delay: Mutex<std::time::Duration>,
}

impl RecordingSecretProvider {
    fn new(outcome: SecretOutcome) -> Self {
        Self {
            outcome: Mutex::new(outcome),
            requests: Mutex::new(Vec::new()),
            delay: Mutex::new(std::time::Duration::ZERO),
        }
    }

    fn set_outcome(&self, outcome: SecretOutcome) {
        *self.outcome.lock().expect("secret outcome lock") = outcome;
    }

    fn requests(&self) -> Vec<SecretAccessRequest> {
        self.requests.lock().expect("secret request lock").clone()
    }

    fn set_delay(&self, delay: std::time::Duration) {
        *self.delay.lock().expect("secret delay lock") = delay;
    }
}

impl SecretProvider for RecordingSecretProvider {
    fn create_secret(
        &mut self,
        _request: SecretCreateRequest,
    ) -> Result<SecretMetadata, SecretError> {
        Err(SecretError::InvalidRequest(
            "read-only test provider".to_string(),
        ))
    }

    fn access_secret(
        &self,
        request: SecretAccessRequest,
    ) -> Result<SecretAccessResult, SecretError> {
        std::thread::sleep(*self.delay.lock().expect("secret delay lock"));
        self.requests
            .lock()
            .expect("secret request lock")
            .push(request);
        match self.outcome.lock().expect("secret outcome lock").clone() {
            SecretOutcome::Granted(value) => {
                Ok(SecretAccessResult::granted(value, "managed-audit-record"))
            }
            SecretOutcome::Denied(reason) => {
                Ok(SecretAccessResult::denied(reason, "managed-audit-record"))
            }
            SecretOutcome::NotFound(secret_id) => Err(SecretError::NotFound(secret_id)),
        }
    }

    fn rotate_secret(
        &mut self,
        _request: SecretRotateRequest,
    ) -> Result<SecretMetadata, SecretError> {
        Err(SecretError::InvalidRequest(
            "read-only test provider".to_string(),
        ))
    }

    fn delete_secret(&mut self, _secret_id: &str) -> Result<(), SecretError> {
        Err(SecretError::InvalidRequest(
            "read-only test provider".to_string(),
        ))
    }

    fn list_secrets(&self) -> Result<Vec<SecretMetadata>, SecretError> {
        Ok(Vec::new())
    }

    fn get_metadata(&self, secret_id: &str) -> Result<SecretMetadata, SecretError> {
        Err(SecretError::NotFound(secret_id.to_string()))
    }

    fn health_check(&self) -> Result<SecretProviderHealth, SecretError> {
        Ok(SecretProviderHealth {
            status: SecretProviderStatus::Healthy,
            secrets_count: 1,
            expired_count: 0,
            last_check_time: 1,
        })
    }

    fn provider_manifest(&self) -> SecretProviderManifest {
        SecretProviderManifest {
            provider_id: "test.managed-secret".to_string(),
            name: "Managed Secret Test Provider".to_string(),
            version: "1.0.0".to_string(),
            supported_algorithms: vec![EncryptionAlgorithm::Aes256Gcm],
            max_secrets: 10,
            supports_rotation: true,
            supports_expiration: true,
        }
    }
}

#[tokio::test]
async fn local_environment_resolution_is_namespaced() {
    let _guard = env_guard().await;
    let variable = format!(
        "{KNOWLEDGEBASE_PROVIDER_SECRET_ENV_PREFIX}DIFY_{}",
        sdkwork_utils_rust::uuid()
            .replace('-', "_")
            .to_ascii_uppercase()
    );
    std::env::set_var(&variable, "local-provider-secret");
    std::env::set_var("SDKWORK_DATABASE_PASSWORD", "unrelated-process-secret");
    let resolver = local_resolver(None);

    let resolved = resolver
        .resolve(&access_context(), &reference(&format!("env://{variable}")))
        .await
        .expect("namespaced local Provider secret");
    let unrelated = resolver.validate_reference_locator(
        "engine.knowledge.external.dify",
        "env://SDKWORK_DATABASE_PASSWORD",
    );

    std::env::remove_var(&variable);
    std::env::remove_var("SDKWORK_DATABASE_PASSWORD");
    assert_eq!(resolved.expose_secret(), "local-provider-secret");
    assert_eq!(
        unrelated,
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    );
}

#[tokio::test]
async fn local_file_resolution_is_bounded_to_the_approved_real_root() {
    let root = temporary_directory("approved-root");
    let outside = temporary_directory("outside-root");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("create approved root");
    tokio::fs::create_dir_all(&outside)
        .await
        .expect("create outside root");
    let provider_root = root.join("dify");
    tokio::fs::create_dir_all(&provider_root)
        .await
        .expect("create Provider root");
    let approved_file = provider_root.join("provider-secret.txt");
    let outside_file = outside.join("host-secret.txt");
    tokio::fs::write(&approved_file, "approved-secret\n")
        .await
        .expect("write approved secret");
    tokio::fs::write(&outside_file, "outside-secret")
        .await
        .expect("write outside secret");
    let resolver = local_resolver(Some(root.clone()));

    let approved = resolver
        .resolve(&access_context(), &reference(&file_url(&approved_file)))
        .await
        .expect("approved file secret");
    let outside_result = resolver
        .validate_reference_locator("engine.knowledge.external.dify", &file_url(&outside_file));

    tokio::fs::remove_dir_all(&root)
        .await
        .expect("remove approved root");
    tokio::fs::remove_dir_all(&outside)
        .await
        .expect("remove outside root");
    assert_eq!(approved.expose_secret(), "approved-secret");
    assert_eq!(
        outside_result,
        Err(KnowledgeEngineProviderCredentialError::AccessDenied)
    );
}

#[tokio::test]
async fn local_file_resolution_rejects_symlink_escape() {
    let root = temporary_directory("symlink-root");
    let outside = temporary_directory("symlink-outside");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("create symlink root");
    tokio::fs::create_dir_all(&outside)
        .await
        .expect("create symlink outside");
    let outside_file = outside.join("host-secret.txt");
    let provider_root = root.join("dify");
    tokio::fs::create_dir_all(&provider_root)
        .await
        .expect("create Provider symlink root");
    let link = provider_root.join("provider-secret.txt");
    tokio::fs::write(&outside_file, "outside-secret")
        .await
        .expect("write outside secret");
    if !create_file_symlink(&outside_file, &link) {
        tokio::fs::remove_dir_all(&root).await.ok();
        tokio::fs::remove_dir_all(&outside).await.ok();
        return;
    }
    let resolver = local_resolver(Some(root.clone()));

    let error = resolver
        .resolve(&access_context(), &reference(&file_url(&link)))
        .await
        .expect_err("symlink escape must fail closed");

    tokio::fs::remove_dir_all(&root).await.ok();
    tokio::fs::remove_dir_all(&outside).await.ok();
    assert_eq!(error, KnowledgeEngineProviderCredentialError::AccessDenied);
}

#[tokio::test]
async fn local_file_resolution_rejects_empty_non_utf8_and_oversized_values() {
    let root = temporary_directory("invalid-files");
    let provider_root = root.join("dify");
    tokio::fs::create_dir_all(&provider_root)
        .await
        .expect("create Provider root");
    let empty_file = provider_root.join("empty");
    let non_utf8_file = provider_root.join("non-utf8");
    let oversized_file = provider_root.join("oversized");
    tokio::fs::write(&empty_file, [])
        .await
        .expect("write empty secret");
    tokio::fs::write(&non_utf8_file, [0xff, 0xfe])
        .await
        .expect("write non-UTF-8 secret");
    tokio::fs::write(&oversized_file, "0123456789")
        .await
        .expect("write oversized secret");
    let config = KnowledgebaseProviderCredentialResolverConfig::local(
        KnowledgebaseProviderCredentialEnvironment::Development,
        Some(root.clone()),
    )
    .expect("local config")
    .with_max_credential_bytes(8)
    .expect("bounded local config");
    let resolver = KnowledgebaseProviderCredentialResolver::local(config).expect("local resolver");

    let empty = resolver
        .resolve(&access_context(), &reference(&file_url(&empty_file)))
        .await;
    let non_utf8 = resolver
        .resolve(&access_context(), &reference(&file_url(&non_utf8_file)))
        .await;
    let oversized = resolver
        .resolve(&access_context(), &reference(&file_url(&oversized_file)))
        .await;

    tokio::fs::remove_dir_all(&root)
        .await
        .expect("remove invalid file root");
    assert!(matches!(
        empty,
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    ));
    assert!(matches!(
        non_utf8,
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    ));
    assert!(matches!(
        oversized,
        Err(KnowledgeEngineProviderCredentialError::ResponseTooLarge)
    ));
}

#[test]
fn staging_and_production_require_managed_sources() {
    for environment in [
        KnowledgebaseProviderCredentialEnvironment::Staging,
        KnowledgebaseProviderCredentialEnvironment::Production,
    ] {
        assert_eq!(
            KnowledgebaseProviderCredentialResolverConfig::local(environment, None),
            Err(KnowledgebaseProviderCredentialResolverConfigurationError::ManagedSourceRequired)
        );
    }
}

#[test]
fn managed_concurrency_limit_is_bounded() {
    let config = || {
        KnowledgebaseProviderCredentialResolverConfig::managed(
            KnowledgebaseProviderCredentialEnvironment::Production,
        )
        .expect("managed config")
    };

    assert_eq!(
        config().with_max_managed_concurrency(0),
        Err(KnowledgebaseProviderCredentialResolverConfigurationError::InvalidManagedConcurrency)
    );
    assert_eq!(
        config().with_max_managed_concurrency(
            KnowledgebaseProviderCredentialResolverConfig::MAX_MANAGED_CONCURRENCY + 1,
        ),
        Err(KnowledgebaseProviderCredentialResolverConfigurationError::InvalidManagedConcurrency)
    );
}

#[test]
fn managed_policy_rejects_environment_and_file_locators() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "managed-secret".to_string(),
    )));
    let resolver = managed_resolver(provider);

    assert_eq!(
        resolver.validate_reference_locator(
            "engine.knowledge.external.dify",
            "env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_DIFY_FORBIDDEN",
        ),
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    );
    assert_eq!(
        resolver
            .validate_reference_locator("engine.knowledge.external.dify", "file:///tmp/forbidden",),
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    );
}

#[test]
fn credential_sources_are_bound_to_the_reference_implementation() {
    let root = temporary_directory("implementation-root");
    let local = local_resolver(Some(root.clone()));
    let dify_env = "env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_DIFY_PRIMARY";
    let dify_file = file_url(&root.join("dify").join("primary"));
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "managed-secret".to_string(),
    )));
    let managed = managed_resolver(provider);

    assert_eq!(
        local.validate_reference_locator("engine.knowledge.external.ragflow", dify_env),
        Err(KnowledgeEngineProviderCredentialError::InvalidReference)
    );
    assert_eq!(
        local.validate_reference_locator("engine.knowledge.external.ragflow", &dify_file),
        Err(KnowledgeEngineProviderCredentialError::AccessDenied)
    );
    assert_eq!(
        managed.validate_reference_locator(
            "engine.knowledge.external.ragflow",
            "secret://knowledgebase/provider/dify/primary",
        ),
        Err(KnowledgeEngineProviderCredentialError::AccessDenied)
    );
}

#[tokio::test]
async fn managed_provider_receives_complete_binding_access_context() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "managed-secret".to_string(),
    )));
    let resolver = managed_resolver(provider.clone());

    let credential = resolver
        .resolve(
            &access_context(),
            &reference("secret://knowledgebase/provider/dify/primary"),
        )
        .await
        .expect("managed Provider secret");
    let requests = provider.requests();
    let request = requests.first().expect("managed access request");

    assert_eq!(credential.expose_secret(), "managed-secret");
    assert_eq!(request.secret_id, "knowledgebase/provider/dify/primary");
    assert_eq!(request.requester, "sdkwork-knowledgebase-provider-binding");
    assert_eq!(
        request.context.get("tenant_id").map(String::as_str),
        Some("11")
    );
    assert_eq!(
        request.context.get("organization_id").map(String::as_str),
        Some("22")
    );
    assert_eq!(
        request.context.get("space_id").map(String::as_str),
        Some("33")
    );
    assert_eq!(
        request.context.get("binding_id").map(String::as_str),
        Some("44")
    );
    assert_eq!(
        request
            .context
            .get("credential_reference_id")
            .map(String::as_str),
        Some("55")
    );
    assert_eq!(
        request
            .context
            .get("credential_reference_version")
            .map(String::as_str),
        Some("3")
    );
    assert_eq!(
        request.context.get("implementation_id").map(String::as_str),
        Some("engine.knowledge.external.dify")
    );
    assert_eq!(
        request.context.get("actor_id").map(String::as_str),
        Some("operator-7")
    );
    assert_eq!(
        request.context.get("operation").map(String::as_str),
        Some("search")
    );
    assert_eq!(
        request.context.get("trace_id").map(String::as_str),
        Some("trace-secret-1")
    );
    assert!(request.context.contains_key("deadline_unix_ms"));
}

#[tokio::test]
async fn managed_provider_errors_are_sanitized() {
    let sensitive_id = "knowledgebase/provider/private/customer-token";
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::NotFound(
        sensitive_id.to_string(),
    )));
    let resolver = managed_resolver(provider.clone());

    let not_found = resolver
        .resolve(
            &access_context(),
            &reference("secret://knowledgebase/provider/private/customer-token"),
        )
        .await
        .expect_err("not found must fail closed");
    provider.set_outcome(SecretOutcome::Denied(
        "tenant policy mentions private id".to_string(),
    ));
    let denied = resolver
        .resolve(
            &access_context(),
            &reference("secret://knowledgebase/provider/private/customer-token"),
        )
        .await
        .expect_err("denial must fail closed");

    for rendered in [not_found.to_string(), denied.to_string()] {
        assert!(!rendered.contains(sensitive_id));
        assert!(!rendered.contains("customer-token"));
        assert!(!rendered.contains("tenant policy"));
    }
}

#[tokio::test]
async fn managed_provider_result_size_is_bounded() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "0123456789".to_string(),
    )));
    let config = KnowledgebaseProviderCredentialResolverConfig::managed(
        KnowledgebaseProviderCredentialEnvironment::Production,
    )
    .expect("managed config")
    .with_max_credential_bytes(8)
    .expect("bounded config");
    let resolver = KnowledgebaseProviderCredentialResolver::managed(config, provider)
        .expect("managed resolver");

    let error = resolver
        .resolve(
            &access_context(),
            &reference("secret://knowledgebase/provider/dify/primary"),
        )
        .await
        .expect_err("oversized secret must fail closed");

    assert_eq!(
        error,
        KnowledgeEngineProviderCredentialError::ResponseTooLarge
    );
}

#[tokio::test]
async fn managed_provider_access_wait_is_independently_bounded() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "managed-secret".to_string(),
    )));
    provider.set_delay(std::time::Duration::from_millis(500));
    let config = KnowledgebaseProviderCredentialResolverConfig::managed(
        KnowledgebaseProviderCredentialEnvironment::Production,
    )
    .expect("managed config")
    .with_max_managed_resolution_duration(std::time::Duration::from_millis(10))
    .expect("bounded timeout config");
    let resolver = KnowledgebaseProviderCredentialResolver::managed(config, provider)
        .expect("managed resolver");
    let started = std::time::Instant::now();

    let result = resolver
        .resolve(
            &access_context(),
            &reference("secret://knowledgebase/provider/dify/primary"),
        )
        .await;

    assert!(matches!(
        result,
        Err(KnowledgeEngineProviderCredentialError::Unavailable)
    ));
    assert!(started.elapsed() < std::time::Duration::from_millis(400));
}

#[tokio::test]
async fn timed_out_managed_calls_keep_the_bulkhead_permit_until_the_provider_returns() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "managed-secret".to_string(),
    )));
    provider.set_delay(std::time::Duration::from_millis(200));
    let config = KnowledgebaseProviderCredentialResolverConfig::managed(
        KnowledgebaseProviderCredentialEnvironment::Production,
    )
    .expect("managed config")
    .with_max_managed_resolution_duration(std::time::Duration::from_millis(20))
    .expect("bounded timeout config")
    .with_max_managed_concurrency(1)
    .expect("single managed call capacity");
    let resolver = KnowledgebaseProviderCredentialResolver::managed(config, provider.clone())
        .expect("managed resolver");
    let locator = reference("secret://knowledgebase/provider/dify/primary");

    let first = resolver.resolve(&access_context(), &locator).await;
    let second_started = std::time::Instant::now();
    let second = resolver.resolve(&access_context(), &locator).await;

    assert!(matches!(
        first,
        Err(KnowledgeEngineProviderCredentialError::Unavailable)
    ));
    assert!(matches!(
        second,
        Err(KnowledgeEngineProviderCredentialError::Unavailable)
    ));
    assert!(second_started.elapsed() < std::time::Duration::from_millis(100));

    tokio::time::sleep(std::time::Duration::from_millis(220)).await;
    assert_eq!(provider.requests().len(), 1);
}

#[tokio::test]
async fn managed_provider_rotation_and_revocation_take_effect_without_cache() {
    let provider = Arc::new(RecordingSecretProvider::new(SecretOutcome::Granted(
        "version-one".to_string(),
    )));
    let resolver = managed_resolver(provider.clone());
    let locator = reference("secret://knowledgebase/provider/dify/primary");

    let first = resolver
        .resolve(&access_context(), &locator)
        .await
        .expect("first secret version");
    provider.set_outcome(SecretOutcome::Granted("version-two".to_string()));
    let second = resolver
        .resolve(&access_context(), &locator)
        .await
        .expect("rotated secret version");
    provider.set_outcome(SecretOutcome::NotFound(
        "knowledgebase/provider/dify/primary".to_string(),
    ));
    let revoked = resolver.resolve(&access_context(), &locator).await;

    assert_eq!(first.expose_secret(), "version-one");
    assert_eq!(second.expose_secret(), "version-two");
    assert!(matches!(
        revoked,
        Err(KnowledgeEngineProviderCredentialError::Unavailable)
    ));
    assert_eq!(provider.requests().len(), 3);
}

fn local_resolver(root: Option<std::path::PathBuf>) -> KnowledgebaseProviderCredentialResolver {
    let config = KnowledgebaseProviderCredentialResolverConfig::local(
        KnowledgebaseProviderCredentialEnvironment::Development,
        root,
    )
    .expect("local resolver config");
    KnowledgebaseProviderCredentialResolver::local(config).expect("local resolver")
}

fn managed_resolver(
    provider: Arc<RecordingSecretProvider>,
) -> KnowledgebaseProviderCredentialResolver {
    let config = KnowledgebaseProviderCredentialResolverConfig::managed(
        KnowledgebaseProviderCredentialEnvironment::Production,
    )
    .expect("managed resolver config");
    KnowledgebaseProviderCredentialResolver::managed(config, provider).expect("managed resolver")
}

fn reference(locator: &str) -> ResolvedKnowledgeEngineProviderCredential {
    ResolvedKnowledgeEngineProviderCredential {
        credential_reference_id: 55,
        implementation_id: "engine.knowledge.external.dify".to_string(),
        reference_locator: locator.to_string(),
        version: 3,
    }
}

fn access_context() -> KnowledgeEngineProviderCredentialAccessContext {
    KnowledgeEngineProviderCredentialAccessContext {
        tenant_id: 11,
        organization_id: 22,
        space_id: 33,
        binding_id: 44,
        credential_reference_id: 55,
        credential_reference_version: 3,
        implementation_id: "engine.knowledge.external.dify".to_string(),
        actor_id: "operator-7".to_string(),
        operation: sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineProviderOperation::Search,
        trace_id: "trace-secret-1".to_string(),
        deadline_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis() as u64
            + 30_000,
    }
}

async fn env_guard() -> tokio::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().await
}

fn temporary_directory(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "sdkwork-knowledgebase-provider-secret-{label}-{}",
        sdkwork_utils_rust::uuid()
    ))
}

fn file_url(path: &std::path::Path) -> String {
    url::Url::from_file_path(path)
        .expect("absolute file URL")
        .to_string()
}

#[cfg(unix)]
fn create_file_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_file_symlink(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::windows::fs::symlink_file(target, link).is_ok()
}
