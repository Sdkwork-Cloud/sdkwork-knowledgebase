use async_trait::async_trait;
use sdkwork_utils_rust::is_blank;
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

use super::knowledge_provider_binding_store::ResolvedKnowledgeEngineProviderCredential;

pub struct KnowledgeEngineProviderCredential {
    value: String,
}

impl KnowledgeEngineProviderCredential {
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeEngineProviderCredentialError> {
        let value = value.into();
        if is_blank(Some(value.as_str())) {
            return Err(KnowledgeEngineProviderCredentialError::Unavailable(
                "Provider credential is empty".to_string(),
            ));
        }
        Ok(Self { value })
    }

    pub fn expose_secret(&self) -> &str {
        &self.value
    }

    pub fn into_secret(mut self) -> Zeroizing<String> {
        Zeroizing::new(std::mem::take(&mut self.value))
    }
}

impl std::fmt::Debug for KnowledgeEngineProviderCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("KnowledgeEngineProviderCredential([REDACTED])")
    }
}

impl Drop for KnowledgeEngineProviderCredential {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

#[async_trait]
pub trait KnowledgeEngineProviderCredentialResolver: Send + Sync {
    fn validate_reference_locator(
        &self,
        reference_locator: &str,
    ) -> Result<(), KnowledgeEngineProviderCredentialError>;

    async fn resolve(
        &self,
        reference: &ResolvedKnowledgeEngineProviderCredential,
    ) -> Result<KnowledgeEngineProviderCredential, KnowledgeEngineProviderCredentialError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineProviderCredentialError {
    #[error("Provider credential reference is invalid: {0}")]
    InvalidReference(String),
    #[error("Provider credential is unavailable: {0}")]
    Unavailable(String),
    #[error("Provider credential resolution failed")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_debug_is_redacted() {
        let credential =
            KnowledgeEngineProviderCredential::new("credential-secret-value").expect("credential");

        let rendered = format!("{credential:?}");

        assert_eq!(rendered, "KnowledgeEngineProviderCredential([REDACTED])");
        assert!(!rendered.contains("credential-secret-value"));
        assert_eq!(credential.expose_secret(), "credential-secret-value");
    }

    #[test]
    fn credential_rejects_empty_values() {
        let error = KnowledgeEngineProviderCredential::new("  ")
            .expect_err("empty credentials must fail closed");

        assert!(matches!(
            error,
            KnowledgeEngineProviderCredentialError::Unavailable(_)
        ));
    }
}
