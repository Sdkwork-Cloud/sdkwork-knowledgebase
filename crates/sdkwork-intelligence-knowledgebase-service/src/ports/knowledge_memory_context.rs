use async_trait::async_trait;
use sdkwork_knowledgebase_contract::rag::KnowledgeMemoryContextFragment;
use thiserror::Error;

#[async_trait]
pub trait KnowledgeMemoryContextProvider: Send + Sync {
    async fn build_memory_context(
        &self,
        request: KnowledgeMemoryContextRequest,
    ) -> Result<KnowledgeMemoryContextResult, KnowledgeMemoryContextProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeMemoryContextRequest {
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
    pub query: String,
    pub memory_policy_ref: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnowledgeMemoryContextResult {
    pub fragments: Vec<KnowledgeMemoryContextFragment>,
    pub truncated: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeMemoryContextProviderError {
    #[error("invalid knowledge memory context request: {0}")]
    InvalidRequest(String),
    #[error("knowledge memory context upstream error: {0}")]
    Upstream(String),
    #[error("knowledge memory context internal error: {0}")]
    Internal(String),
}
