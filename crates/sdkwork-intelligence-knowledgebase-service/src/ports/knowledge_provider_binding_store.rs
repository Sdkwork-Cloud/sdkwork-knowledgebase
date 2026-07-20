use async_trait::async_trait;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineCapability, KnowledgeEngineProviderErrorCategory,
};
use sdkwork_knowledgebase_contract::provider_binding::{
    CreateKnowledgeEngineProviderBindingRequest,
    CreateKnowledgeEngineProviderCredentialReferenceRequest, KnowledgeEngineProviderBinding,
    KnowledgeEngineProviderBindingList, KnowledgeEngineProviderCredentialReference,
    KnowledgeEngineProviderCredentialReferenceList, ListKnowledgeEngineProviderBindingsRequest,
    ListKnowledgeEngineProviderCredentialReferencesRequest,
    RevokeKnowledgeEngineProviderCredentialReferenceRequest,
    RotateKnowledgeEngineProviderCredentialReferenceRequest,
    UpdateKnowledgeEngineProviderBindingRequest,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgeEngineProviderScope {
    pub tenant_id: u64,
    pub organization_id: u64,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedKnowledgeEngineProviderCredential {
    pub credential_reference_id: u64,
    pub implementation_id: String,
    pub reference_locator: String,
    pub version: u64,
}

impl std::fmt::Debug for ResolvedKnowledgeEngineProviderCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ResolvedKnowledgeEngineProviderCredential")
            .field("credential_reference_id", &self.credential_reference_id)
            .field("implementation_id", &self.implementation_id)
            .field("reference_locator", &"[REDACTED]")
            .field("version", &self.version)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordKnowledgeEngineProviderTestResult {
    pub expected_version: u64,
    pub capabilities: Vec<KnowledgeEngineCapability>,
    pub error_category: Option<KnowledgeEngineProviderErrorCategory>,
    pub updated_by: String,
}

#[async_trait]
pub trait KnowledgeEngineProviderBindingStore: Send + Sync {
    async fn create_credential_reference(
        &self,
        scope: KnowledgeEngineProviderScope,
        actor_id: &str,
        request: CreateKnowledgeEngineProviderCredentialReferenceRequest,
    ) -> Result<KnowledgeEngineProviderCredentialReference, KnowledgeEngineProviderBindingStoreError>;

    async fn resolve_credential_reference(
        &self,
        scope: KnowledgeEngineProviderScope,
        credential_reference_id: u64,
        implementation_id: &str,
    ) -> Result<ResolvedKnowledgeEngineProviderCredential, KnowledgeEngineProviderBindingStoreError>;

    async fn get_credential_reference(
        &self,
        scope: KnowledgeEngineProviderScope,
        credential_reference_id: u64,
    ) -> Result<KnowledgeEngineProviderCredentialReference, KnowledgeEngineProviderBindingStoreError>;

    async fn list_credential_references(
        &self,
        scope: KnowledgeEngineProviderScope,
        request: ListKnowledgeEngineProviderCredentialReferencesRequest,
    ) -> Result<
        KnowledgeEngineProviderCredentialReferenceList,
        KnowledgeEngineProviderBindingStoreError,
    >;

    async fn rotate_credential_reference(
        &self,
        scope: KnowledgeEngineProviderScope,
        credential_reference_id: u64,
        actor_id: &str,
        request: RotateKnowledgeEngineProviderCredentialReferenceRequest,
    ) -> Result<KnowledgeEngineProviderCredentialReference, KnowledgeEngineProviderBindingStoreError>;

    async fn revoke_credential_reference(
        &self,
        scope: KnowledgeEngineProviderScope,
        credential_reference_id: u64,
        actor_id: &str,
        request: RevokeKnowledgeEngineProviderCredentialReferenceRequest,
    ) -> Result<KnowledgeEngineProviderCredentialReference, KnowledgeEngineProviderBindingStoreError>;

    async fn create_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        actor_id: &str,
        request: CreateKnowledgeEngineProviderBindingRequest,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn get_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn get_active_binding_for_space(
        &self,
        scope: KnowledgeEngineProviderScope,
        space_id: u64,
    ) -> Result<Option<KnowledgeEngineProviderBinding>, KnowledgeEngineProviderBindingStoreError>;

    async fn list_bindings(
        &self,
        scope: KnowledgeEngineProviderScope,
        request: ListKnowledgeEngineProviderBindingsRequest,
    ) -> Result<KnowledgeEngineProviderBindingList, KnowledgeEngineProviderBindingStoreError>;

    async fn update_draft_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
        actor_id: &str,
        request: UpdateKnowledgeEngineProviderBindingRequest,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn begin_binding_test(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
        actor_id: &str,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn record_binding_test_result(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
        result: RecordKnowledgeEngineProviderTestResult,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn activate_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
        actor_id: &str,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;

    async fn disable_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        binding_id: u64,
        actor_id: &str,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineProviderBindingStoreError {
    #[error("knowledge engine provider binding invalid request: {0}")]
    InvalidRequest(String),
    #[error("knowledge engine provider binding not found: {0}")]
    NotFound(u64),
    #[error("knowledge engine provider binding conflict: {0}")]
    Conflict(String),
    #[error("knowledge engine provider binding invalid lifecycle: {0}")]
    InvalidLifecycle(String),
    #[error("knowledge engine provider binding credential unavailable: {0}")]
    CredentialUnavailable(u64),
    #[error("knowledge engine provider binding internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_credential_reference_debug_redacts_locator() {
        let reference = ResolvedKnowledgeEngineProviderCredential {
            credential_reference_id: 73,
            implementation_id: "engine.knowledge.external.dify".to_string(),
            reference_locator: "secret://knowledgebase/dify/private".to_string(),
            version: 4,
        };

        let rendered = format!("{reference:?}");

        assert!(rendered.contains("credential_reference_id: 73"));
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("secret://knowledgebase/dify/private"));
    }
}
