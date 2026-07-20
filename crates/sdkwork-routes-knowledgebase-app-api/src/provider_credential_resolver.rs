use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_binding_store::ResolvedKnowledgeEngineProviderCredential;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_provider_credential_resolver::{
    KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError,
    KnowledgeEngineProviderCredentialResolver,
};

const MAX_PROVIDER_CREDENTIAL_BYTES: u64 = 64 * 1024;

#[derive(Debug, Default)]
pub(crate) struct RuntimeKnowledgeEngineProviderCredentialResolver;

#[async_trait]
impl KnowledgeEngineProviderCredentialResolver
    for RuntimeKnowledgeEngineProviderCredentialResolver
{
    fn validate_reference_locator(
        &self,
        reference_locator: &str,
    ) -> Result<(), KnowledgeEngineProviderCredentialError> {
        if let Some(variable) = reference_locator.strip_prefix("env://") {
            return validate_environment_reference(variable);
        }
        if reference_locator.starts_with("file://") {
            validate_file_reference(reference_locator)?;
            return Ok(());
        }
        Err(KnowledgeEngineProviderCredentialError::InvalidReference(
            "unsupported Provider credential reference scheme".to_string(),
        ))
    }

    async fn resolve(
        &self,
        reference: &ResolvedKnowledgeEngineProviderCredential,
    ) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
        self.validate_reference_locator(&reference.reference_locator)?;
        if let Some(variable) = reference.reference_locator.strip_prefix("env://") {
            return resolve_environment_credential(variable);
        }
        if reference.reference_locator.starts_with("file://") {
            return resolve_file_credential(&reference.reference_locator).await;
        }
        Err(KnowledgeEngineProviderCredentialError::InvalidReference(
            "unsupported Provider credential reference scheme".to_string(),
        ))
    }
}

fn resolve_environment_credential(
    variable: &str,
) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
    validate_environment_reference(variable)?;
    let value = std::env::var(variable).map_err(|_| {
        KnowledgeEngineProviderCredentialError::Unavailable(
            "Provider credential environment value is unavailable".to_string(),
        )
    })?;
    KnowledgeEngineProviderCredential::new(value)
}

fn validate_environment_reference(
    variable: &str,
) -> Result<(), KnowledgeEngineProviderCredentialError> {
    if variable.is_empty()
        || variable.len() > 128
        || !variable
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference(
            "Provider credential environment reference is invalid".to_string(),
        ));
    }
    Ok(())
}

async fn resolve_file_credential(
    locator: &str,
) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError> {
    let path = validate_file_reference(locator)?;
    let metadata = tokio::fs::metadata(&path).await.map_err(|_| {
        KnowledgeEngineProviderCredentialError::Unavailable(
            "Provider credential file is unavailable".to_string(),
        )
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_PROVIDER_CREDENTIAL_BYTES
    {
        return Err(KnowledgeEngineProviderCredentialError::InvalidReference(
            "Provider credential file must be a non-empty regular file within the size limit"
                .to_string(),
        ));
    }
    let value = tokio::fs::read_to_string(path).await.map_err(|_| {
        KnowledgeEngineProviderCredentialError::Unavailable(
            "Provider credential file cannot be read".to_string(),
        )
    })?;
    KnowledgeEngineProviderCredential::new(value.trim().to_string())
}

fn validate_file_reference(
    locator: &str,
) -> Result<std::path::PathBuf, KnowledgeEngineProviderCredentialError> {
    let url = url::Url::parse(locator).map_err(|_| {
        KnowledgeEngineProviderCredentialError::InvalidReference(
            "Provider credential file reference is invalid".to_string(),
        )
    })?;
    url.to_file_path().map_err(|_| {
        KnowledgeEngineProviderCredentialError::InvalidReference(
            "Provider credential file reference is invalid".to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(locator: &str) -> ResolvedKnowledgeEngineProviderCredential {
        ResolvedKnowledgeEngineProviderCredential {
            credential_reference_id: 73,
            implementation_id: "engine.knowledge.external.test".to_string(),
            reference_locator: locator.to_string(),
            version: 2,
        }
    }

    #[tokio::test]
    async fn resolves_environment_reference_without_exposing_value() {
        let key = "SDKWORK_KNOWLEDGEBASE_TEST_PROVIDER_CREDENTIAL_RESOLVER";
        std::env::set_var(key, "environment-secret");
        let resolver = RuntimeKnowledgeEngineProviderCredentialResolver;

        let credential = resolver
            .resolve(&reference(&format!("env://{key}")))
            .await
            .expect("resolve environment credential");

        std::env::remove_var(key);
        assert_eq!(credential.expose_secret(), "environment-secret");
        assert!(!format!("{credential:?}").contains("environment-secret"));
    }

    #[tokio::test]
    async fn resolves_bounded_file_reference() {
        let root = std::env::temp_dir().join(format!(
            "sdkwork-knowledgebase-provider-credential-{}",
            sdkwork_utils_rust::uuid()
        ));
        tokio::fs::create_dir_all(&root)
            .await
            .expect("create credential test directory");
        let path = root.join("credential.txt");
        tokio::fs::write(&path, "file-secret\n")
            .await
            .expect("write credential file");
        let locator = url::Url::from_file_path(&path)
            .expect("file URL")
            .to_string();
        let resolver = RuntimeKnowledgeEngineProviderCredentialResolver;

        let credential = resolver
            .resolve(&reference(&locator))
            .await
            .expect("resolve file credential");

        tokio::fs::remove_dir_all(root)
            .await
            .expect("remove credential test directory");
        assert_eq!(credential.expose_secret(), "file-secret");
    }

    #[tokio::test]
    async fn rejects_unknown_or_unsafe_references_without_echoing_locator() {
        let resolver = RuntimeKnowledgeEngineProviderCredentialResolver;
        for locator in [
            "secret://knowledgebase/private/provider",
            "env://mixedCase",
            "env://",
        ] {
            let error = resolver
                .resolve(&reference(locator))
                .await
                .expect_err("unsupported or unsafe reference must fail closed");
            let rendered = error.to_string();
            assert!(!rendered.contains(locator));
            assert!(!rendered.contains("knowledgebase/private/provider"));
        }
    }
}
