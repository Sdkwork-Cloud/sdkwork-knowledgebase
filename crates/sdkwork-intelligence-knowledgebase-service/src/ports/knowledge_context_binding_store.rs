use async_trait::async_trait;
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest,
    KnowledgeSpaceContextBinding, KnowledgeSpaceContextBindingList,
    ListContextBoundSpacesRequest, ListKnowledgeSpaceContextBindingsRequest,
    UpdateKnowledgeSpaceContextBindingRequest,
};
use thiserror::Error;

#[async_trait]
pub trait KnowledgeContextBindingStore: Send + Sync {
    async fn create_binding(
        &self,
        tenant_id: u64,
        created_by: &str,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError>;

    async fn get_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError>;

    async fn update_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingStoreError>;

    async fn delete_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
    ) -> Result<(), KnowledgeContextBindingStoreError>;

    async fn list_space_bindings(
        &self,
        tenant_id: u64,
        request: ListKnowledgeSpaceContextBindingsRequest,
    ) -> Result<KnowledgeSpaceContextBindingList, KnowledgeContextBindingStoreError>;

    async fn list_context_bound_spaces(
        &self,
        tenant_id: u64,
        request: ListContextBoundSpacesRequest,
    ) -> Result<Vec<u64>, KnowledgeContextBindingStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeContextBindingStoreError {
    #[error("context binding invalid request: {0}")]
    InvalidRequest(String),
    #[error("context binding not found: {0}")]
    NotFound(u64),
    #[error("context binding conflict: {0}")]
    Conflict(String),
    #[error("context binding internal error: {0}")]
    Internal(String),
}
