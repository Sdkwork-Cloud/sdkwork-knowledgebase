use async_trait::async_trait;
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeAgentBinding, KnowledgeAgentBindingRequest, KnowledgeAgentProfile,
    KnowledgeAgentProfileRequest,
};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeAgentProfileStore: Send + Sync {
    async fn create_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError>;

    async fn retrieve_profile(
        &self,
        profile_id: u64,
    ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError>;

    async fn update_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> Result<KnowledgeAgentProfile, KnowledgeAgentProfileStoreError>;

    async fn delete_profile(&self, profile_id: u64) -> Result<(), KnowledgeAgentProfileStoreError>;

    async fn list_bindings(
        &self,
        profile_id: u64,
    ) -> Result<Vec<KnowledgeAgentBinding>, KnowledgeAgentProfileStoreError>;

    async fn create_binding(
        &self,
        request: KnowledgeAgentBindingRequest,
    ) -> Result<KnowledgeAgentBinding, KnowledgeAgentProfileStoreError>;

    async fn update_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> Result<KnowledgeAgentBinding, KnowledgeAgentProfileStoreError>;

    async fn delete_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
    ) -> Result<(), KnowledgeAgentProfileStoreError>;

    /// Resolve the highest-priority active agent profile bound to a knowledge space.
    ///
    /// Route crates use this instead of issuing raw SQL against
    /// `kb_agent_knowledge_binding`, keeping persistence concerns inside the
    /// repository layer.
    async fn resolve_profile_id_for_space(
        &self,
        tenant_id: u64,
        space_id: u64,
    ) -> Result<Option<u64>, KnowledgeAgentProfileStoreError> {
        let _ = (tenant_id, space_id);
        Err(KnowledgeAgentProfileStoreError::Internal(
            "resolve_profile_id_for_space is not implemented for this store".to_string(),
        ))
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeAgentProfileStoreError {
    #[error("knowledge agent profile not found: {0}")]
    NotFound(u64),
    #[error("knowledge agent profile store conflict: {0}")]
    Conflict(String),
    #[error("knowledge agent profile store internal error: {0}")]
    Internal(String),
}
