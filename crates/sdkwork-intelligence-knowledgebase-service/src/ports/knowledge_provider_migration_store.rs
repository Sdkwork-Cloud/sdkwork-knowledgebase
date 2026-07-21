use std::time::Duration;

use async_trait::async_trait;
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineProviderErrorCategory;
use sdkwork_knowledgebase_contract::provider_binding::{
    CreateKnowledgeEngineProviderMigrationOperationRequest,
    KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationOperationList,
    KnowledgeEngineProviderMigrationState, ListKnowledgeEngineProviderMigrationOperationsRequest,
};
use serde_json::Value;
use thiserror::Error;

use super::knowledge_provider_binding_store::KnowledgeEngineProviderScope;

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimedKnowledgeEngineProviderMigration {
    pub operation: KnowledgeEngineProviderMigrationOperation,
    pub claim_token: String,
    pub checkpoint: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvanceClaimedKnowledgeEngineProviderMigration {
    pub expected_state: KnowledgeEngineProviderMigrationState,
    pub next_state: KnowledgeEngineProviderMigrationState,
    pub checkpoint: Value,
    pub observation_until: Option<String>,
    pub error_category: Option<KnowledgeEngineProviderErrorCategory>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CutoverClaimedKnowledgeEngineProviderMigration {
    pub operation_id: u64,
    pub claim_token: String,
    pub expected_version: u64,
    pub actor_id: String,
    pub observation_until: String,
    pub checkpoint: Value,
}

#[async_trait]
pub trait KnowledgeEngineProviderMigrationStore: Send + Sync {
    async fn create_operation(
        &self,
        scope: KnowledgeEngineProviderScope,
        space_id: u64,
        actor_id: &str,
        request: CreateKnowledgeEngineProviderMigrationOperationRequest,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;

    async fn get_operation(
        &self,
        scope: KnowledgeEngineProviderScope,
        operation_id: u64,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;

    async fn list_operations(
        &self,
        scope: KnowledgeEngineProviderScope,
        request: ListKnowledgeEngineProviderMigrationOperationsRequest,
    ) -> Result<
        KnowledgeEngineProviderMigrationOperationList,
        KnowledgeEngineProviderMigrationStoreError,
    >;

    async fn request_rollback(
        &self,
        scope: KnowledgeEngineProviderScope,
        operation_id: u64,
        actor_id: &str,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;

    async fn claim_next(
        &self,
        scope: KnowledgeEngineProviderScope,
        worker_id: &str,
        lease_duration: Duration,
    ) -> Result<
        Option<ClaimedKnowledgeEngineProviderMigration>,
        KnowledgeEngineProviderMigrationStoreError,
    >;

    async fn advance_claimed(
        &self,
        scope: KnowledgeEngineProviderScope,
        operation_id: u64,
        claim_token: &str,
        expected_version: u64,
        transition: AdvanceClaimedKnowledgeEngineProviderMigration,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;

    async fn cutover_claimed(
        &self,
        scope: KnowledgeEngineProviderScope,
        command: CutoverClaimedKnowledgeEngineProviderMigration,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;

    async fn rollback_claimed(
        &self,
        scope: KnowledgeEngineProviderScope,
        operation_id: u64,
        claim_token: &str,
        expected_version: u64,
        actor_id: &str,
        checkpoint: Value,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineProviderMigrationStoreError {
    #[error("Provider migration invalid request: {0}")]
    InvalidRequest(String),
    #[error("Provider migration operation not found: {0}")]
    NotFound(u64),
    #[error("Provider migration conflict: {0}")]
    Conflict(String),
    #[error("Provider migration invalid lifecycle: {0}")]
    InvalidLifecycle(String),
    #[error("Provider migration claim lost: {0}")]
    ClaimLost(u64),
    #[error("Provider migration internal error: {0}")]
    Internal(String),
}
