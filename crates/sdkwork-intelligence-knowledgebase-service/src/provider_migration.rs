use std::sync::Arc;
use std::time::Duration;

use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineCapability, KnowledgeEngineProviderErrorCategory,
};
use sdkwork_knowledgebase_contract::provider_binding::{
    CreateKnowledgeEngineProviderMigrationOperationRequest, KnowledgeEngineProviderBindingState,
    KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationOperationList,
    KnowledgeEngineProviderMigrationState, ListKnowledgeEngineProviderMigrationOperationsRequest,
};
use serde_json::{json, Value};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::ports::knowledge_provider_binding_store::{
    KnowledgeEngineProviderBindingStore, KnowledgeEngineProviderBindingStoreError,
    KnowledgeEngineProviderScope,
};
use crate::ports::knowledge_provider_migration_store::{
    AdvanceClaimedKnowledgeEngineProviderMigration, ClaimedKnowledgeEngineProviderMigration,
    KnowledgeEngineProviderMigrationStore, KnowledgeEngineProviderMigrationStoreError,
};

const MAX_BATCH_SIZE: u32 = 200;

pub struct KnowledgeEngineProviderMigrationService<B, M> {
    binding_store: Arc<B>,
    migration_store: Arc<M>,
    scope: KnowledgeEngineProviderScope,
}

impl<B, M> Clone for KnowledgeEngineProviderMigrationService<B, M> {
    fn clone(&self) -> Self {
        Self {
            binding_store: self.binding_store.clone(),
            migration_store: self.migration_store.clone(),
            scope: self.scope,
        }
    }
}

impl<B, M> KnowledgeEngineProviderMigrationService<B, M>
where
    B: KnowledgeEngineProviderBindingStore,
    M: KnowledgeEngineProviderMigrationStore,
{
    pub fn new(
        binding_store: Arc<B>,
        migration_store: Arc<M>,
        scope: KnowledgeEngineProviderScope,
    ) -> Self {
        Self {
            binding_store,
            migration_store,
            scope,
        }
    }

    pub async fn create_operation(
        &self,
        space_id: u64,
        actor_id: &str,
        request: CreateKnowledgeEngineProviderMigrationOperationRequest,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, ProviderMigrationServiceError> {
        self.migration_store
            .create_operation(self.scope, space_id, actor_id, request)
            .await
            .map_err(Into::into)
    }

    pub async fn get_operation(
        &self,
        operation_id: u64,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, ProviderMigrationServiceError> {
        self.migration_store
            .get_operation(self.scope, operation_id)
            .await
            .map_err(Into::into)
    }

    pub async fn list_operations(
        &self,
        request: ListKnowledgeEngineProviderMigrationOperationsRequest,
    ) -> Result<KnowledgeEngineProviderMigrationOperationList, ProviderMigrationServiceError> {
        self.migration_store
            .list_operations(self.scope, request)
            .await
            .map_err(Into::into)
    }

    pub async fn request_rollback(
        &self,
        operation_id: u64,
        actor_id: &str,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, ProviderMigrationServiceError> {
        self.migration_store
            .request_rollback(self.scope, operation_id, actor_id, expected_version)
            .await
            .map_err(Into::into)
    }

    pub async fn process_batch(
        &self,
        worker_id: &str,
        lease_duration: Duration,
        limit: u32,
    ) -> Result<ProviderMigrationBatchResult, ProviderMigrationServiceError> {
        if limit == 0 || limit > MAX_BATCH_SIZE {
            return Err(ProviderMigrationServiceError::InvalidRequest(format!(
                "limit must be between 1 and {MAX_BATCH_SIZE}"
            )));
        }
        let mut result = ProviderMigrationBatchResult::default();
        for _ in 0..limit {
            let Some(claimed) = self
                .migration_store
                .claim_next(self.scope, worker_id, lease_duration)
                .await?
            else {
                break;
            };
            let processed_operation = match self.process_claimed(worker_id, &claimed).await {
                Ok(operation) => operation,
                Err(error) if error.should_fail_operation() => {
                    self.fail_claimed(&claimed, error.provider_error_category())
                        .await?
                }
                Err(error) => return Err(error),
            };
            sdkwork_knowledgebase_observability::record_provider_migration_transition(
                self.scope.tenant_id,
                worker_id,
                processed_operation.id,
                processed_operation.space_id,
                claimed.operation.operation_state.as_str(),
                processed_operation.operation_state.as_str(),
                processed_operation.version,
            )
            .await?;
            let terminal_state = processed_operation.operation_state;
            result.processed += 1;
            match terminal_state {
                KnowledgeEngineProviderMigrationState::Completed => result.completed += 1,
                KnowledgeEngineProviderMigrationState::RolledBack => result.rolled_back += 1,
                KnowledgeEngineProviderMigrationState::Failed => result.failed += 1,
                _ => {}
            }
        }
        Ok(result)
    }

    async fn process_claimed(
        &self,
        worker_id: &str,
        claimed: &ClaimedKnowledgeEngineProviderMigration,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, ProviderMigrationServiceError> {
        let operation = &claimed.operation;
        let result = match operation.operation_state {
            KnowledgeEngineProviderMigrationState::DryRun => {
                self.validate_pre_cutover(operation, &claimed.checkpoint)
                    .await?;
                let checkpoint = checkpoint_with_stage(&claimed.checkpoint, "dryRunValidatedAt")?;
                self.advance(
                    &claimed,
                    KnowledgeEngineProviderMigrationState::Preparing,
                    checkpoint,
                    None,
                )
                .await
            }
            KnowledgeEngineProviderMigrationState::Preparing => {
                self.validate_pre_cutover(operation, &claimed.checkpoint)
                    .await?;
                let mut checkpoint =
                    checkpoint_with_stage(&claimed.checkpoint, "targetPreparedAt")?;
                checkpoint["preparationMode"] = json!("pre_provisioned_target");
                self.advance(
                    &claimed,
                    KnowledgeEngineProviderMigrationState::Validating,
                    checkpoint,
                    None,
                )
                .await
            }
            KnowledgeEngineProviderMigrationState::Validating => {
                self.validate_pre_cutover(operation, &claimed.checkpoint)
                    .await?;
                let checkpoint = checkpoint_with_stage(&claimed.checkpoint, "validatedAt")?;
                self.advance(
                    &claimed,
                    KnowledgeEngineProviderMigrationState::Cutover,
                    checkpoint,
                    None,
                )
                .await
            }
            KnowledgeEngineProviderMigrationState::Cutover => {
                self.validate_pre_cutover(operation, &claimed.checkpoint)
                    .await?;
                let observation_seconds =
                    checkpoint_u64(&claimed.checkpoint, "observationSeconds")?;
                let observation_until = (OffsetDateTime::now_utc()
                    + time::Duration::seconds(i64::try_from(observation_seconds).map_err(
                        |_| {
                            ProviderMigrationServiceError::InvalidCheckpoint(
                                "observationSeconds is outside i64 range".to_string(),
                            )
                        },
                    )?))
                .format(&Rfc3339)
                .map_err(|error| ProviderMigrationServiceError::Internal(error.to_string()))?;
                self.migration_store
                    .cutover_claimed(
                        self.scope,
                        operation.id,
                        &claimed.claim_token,
                        operation.version,
                        worker_id,
                        &observation_until,
                        claimed.checkpoint.clone(),
                    )
                    .await
            }
            KnowledgeEngineProviderMigrationState::Observing => {
                self.validate_post_cutover(operation).await?;
                let checkpoint =
                    checkpoint_with_stage(&claimed.checkpoint, "observationCompletedAt")?;
                self.advance(
                    &claimed,
                    KnowledgeEngineProviderMigrationState::Completed,
                    checkpoint,
                    operation.observation_until.clone(),
                )
                .await
            }
            KnowledgeEngineProviderMigrationState::RollingBack => {
                self.migration_store
                    .rollback_claimed(
                        self.scope,
                        operation.id,
                        &claimed.claim_token,
                        operation.version,
                        worker_id,
                        claimed.checkpoint.clone(),
                    )
                    .await
            }
            state @ (KnowledgeEngineProviderMigrationState::Completed
            | KnowledgeEngineProviderMigrationState::RolledBack
            | KnowledgeEngineProviderMigrationState::Failed) => {
                return Err(ProviderMigrationServiceError::InvalidLifecycle(format!(
                    "terminal state {} cannot be processed",
                    state.as_str()
                )));
            }
        }?;
        Ok(result)
    }

    async fn advance(
        &self,
        claimed: &ClaimedKnowledgeEngineProviderMigration,
        next_state: KnowledgeEngineProviderMigrationState,
        checkpoint: Value,
        observation_until: Option<String>,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>
    {
        self.migration_store
            .advance_claimed(
                self.scope,
                claimed.operation.id,
                &claimed.claim_token,
                claimed.operation.version,
                AdvanceClaimedKnowledgeEngineProviderMigration {
                    expected_state: claimed.operation.operation_state,
                    next_state,
                    checkpoint,
                    observation_until,
                    error_category: None,
                },
            )
            .await
    }

    async fn fail_claimed(
        &self,
        claimed: &ClaimedKnowledgeEngineProviderMigration,
        error_category: KnowledgeEngineProviderErrorCategory,
    ) -> Result<KnowledgeEngineProviderMigrationOperation, KnowledgeEngineProviderMigrationStoreError>
    {
        self.migration_store
            .advance_claimed(
                self.scope,
                claimed.operation.id,
                &claimed.claim_token,
                claimed.operation.version,
                AdvanceClaimedKnowledgeEngineProviderMigration {
                    expected_state: claimed.operation.operation_state,
                    next_state: KnowledgeEngineProviderMigrationState::Failed,
                    checkpoint: claimed.checkpoint.clone(),
                    observation_until: claimed.operation.observation_until.clone(),
                    error_category: Some(error_category),
                },
            )
            .await
    }

    async fn validate_pre_cutover(
        &self,
        operation: &KnowledgeEngineProviderMigrationOperation,
        checkpoint: &Value,
    ) -> Result<(), ProviderMigrationServiceError> {
        let source = self
            .binding_store
            .get_binding(self.scope, operation.source_binding_id)
            .await?;
        let target = self
            .binding_store
            .get_binding(self.scope, operation.target_binding_id)
            .await?;
        let expected_source_version = checkpoint_u64(checkpoint, "expectedSourceVersion")?;
        let expected_target_version = checkpoint_u64(checkpoint, "expectedTargetVersion")?;
        if source.space_id != operation.space_id
            || target.space_id != operation.space_id
            || source.lifecycle_state != KnowledgeEngineProviderBindingState::Active
            || target.lifecycle_state != KnowledgeEngineProviderBindingState::Testing
            || source.version != expected_source_version
            || target.version != expected_target_version
        {
            return Err(ProviderMigrationServiceError::InvalidLifecycle(
                "Provider migration Binding scope, state, or version changed".to_string(),
            ));
        }
        if target.last_tested_at.is_none()
            || !target
                .capability_snapshot
                .contains(&KnowledgeEngineCapability::Health)
            || !target
                .capability_snapshot
                .contains(&KnowledgeEngineCapability::Search)
        {
            return Err(ProviderMigrationServiceError::InvalidLifecycle(
                "target Binding must have a successful health and search capability test"
                    .to_string(),
            ));
        }
        Ok(())
    }

    async fn validate_post_cutover(
        &self,
        operation: &KnowledgeEngineProviderMigrationOperation,
    ) -> Result<(), ProviderMigrationServiceError> {
        let active = self
            .binding_store
            .get_active_binding_for_space(self.scope, operation.space_id)
            .await?
            .ok_or_else(|| {
                ProviderMigrationServiceError::InvalidLifecycle(
                    "migration space has no active Binding during observation".to_string(),
                )
            })?;
        if active.id != operation.target_binding_id {
            return Err(ProviderMigrationServiceError::InvalidLifecycle(
                "migration target is not the active Binding during observation".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProviderMigrationBatchResult {
    pub processed: usize,
    pub completed: usize,
    pub rolled_back: usize,
    pub failed: usize,
}

#[derive(Debug, Error)]
pub enum ProviderMigrationServiceError {
    #[error("Provider migration invalid request: {0}")]
    InvalidRequest(String),
    #[error("Provider migration invalid lifecycle: {0}")]
    InvalidLifecycle(String),
    #[error("Provider migration checkpoint is invalid: {0}")]
    InvalidCheckpoint(String),
    #[error("Provider migration store error: {0}")]
    MigrationStore(#[from] KnowledgeEngineProviderMigrationStoreError),
    #[error("Provider Binding store error: {0}")]
    BindingStore(#[from] KnowledgeEngineProviderBindingStoreError),
    #[error("Provider migration internal error: {0}")]
    Internal(String),
    #[error("Provider migration audit error: {0}")]
    Audit(#[from] sdkwork_knowledgebase_observability::AuditPersistenceError),
}

impl ProviderMigrationServiceError {
    fn should_fail_operation(&self) -> bool {
        match self {
            Self::InvalidRequest(_) | Self::InvalidLifecycle(_) | Self::InvalidCheckpoint(_) => {
                true
            }
            Self::MigrationStore(
                KnowledgeEngineProviderMigrationStoreError::InvalidRequest(_)
                | KnowledgeEngineProviderMigrationStoreError::NotFound(_)
                | KnowledgeEngineProviderMigrationStoreError::Conflict(_)
                | KnowledgeEngineProviderMigrationStoreError::InvalidLifecycle(_),
            ) => true,
            Self::BindingStore(
                KnowledgeEngineProviderBindingStoreError::InvalidRequest(_)
                | KnowledgeEngineProviderBindingStoreError::NotFound(_)
                | KnowledgeEngineProviderBindingStoreError::Conflict(_)
                | KnowledgeEngineProviderBindingStoreError::InvalidLifecycle(_)
                | KnowledgeEngineProviderBindingStoreError::CredentialUnavailable(_),
            ) => true,
            Self::MigrationStore(
                KnowledgeEngineProviderMigrationStoreError::ClaimLost(_)
                | KnowledgeEngineProviderMigrationStoreError::Internal(_),
            )
            | Self::BindingStore(KnowledgeEngineProviderBindingStoreError::Internal(_))
            | Self::Internal(_)
            | Self::Audit(_) => false,
        }
    }

    pub fn provider_error_category(&self) -> KnowledgeEngineProviderErrorCategory {
        match self {
            Self::InvalidRequest(_) | Self::InvalidCheckpoint(_) => {
                KnowledgeEngineProviderErrorCategory::Validation
            }
            Self::InvalidLifecycle(_) => KnowledgeEngineProviderErrorCategory::InvalidTarget,
            Self::MigrationStore(KnowledgeEngineProviderMigrationStoreError::NotFound(_))
            | Self::BindingStore(KnowledgeEngineProviderBindingStoreError::NotFound(_)) => {
                KnowledgeEngineProviderErrorCategory::NotFound
            }
            Self::MigrationStore(_)
            | Self::BindingStore(_)
            | Self::Internal(_)
            | Self::Audit(_) => KnowledgeEngineProviderErrorCategory::Internal,
        }
    }
}

fn checkpoint_with_stage(
    checkpoint: &Value,
    field: &str,
) -> Result<Value, ProviderMigrationServiceError> {
    let mut checkpoint = checkpoint.clone();
    let object = checkpoint.as_object_mut().ok_or_else(|| {
        ProviderMigrationServiceError::InvalidCheckpoint("root must be an object".to_string())
    })?;
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| ProviderMigrationServiceError::Internal(error.to_string()))?;
    object.insert(field.to_string(), json!(now));
    Ok(checkpoint)
}

fn checkpoint_u64(value: &Value, key: &str) -> Result<u64, ProviderMigrationServiceError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| ProviderMigrationServiceError::InvalidCheckpoint(format!("missing {key}")))
}
