use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineCapability, KnowledgeEngineError, KnowledgeEngineHealthStatus,
    KnowledgeEngineProviderErrorCategory,
};
use sdkwork_knowledgebase_contract::provider_binding::{
    CreateKnowledgeEngineProviderBindingRequest,
    CreateKnowledgeEngineProviderCredentialReferenceRequest, KnowledgeEngineExecutionContext,
    KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingList,
    KnowledgeEngineProviderCredentialReference, ListKnowledgeEngineProviderBindingsRequest,
    UpdateKnowledgeEngineProviderBindingRequest,
};
use thiserror::Error;

use crate::ports::knowledge_engine::KnowledgeEngineRegistry;
use crate::ports::knowledge_provider_binding_store::{
    KnowledgeEngineProviderBindingStore, KnowledgeEngineProviderBindingStoreError,
    KnowledgeEngineProviderScope, RecordKnowledgeEngineProviderTestResult,
};

pub const KNOWLEDGE_PROVIDER_MANAGE_PERMISSION: &str = "knowledge.providers.manage";

pub struct KnowledgeEngineProviderBindingService<R> {
    store: Arc<dyn KnowledgeEngineProviderBindingStore>,
    registry: Arc<R>,
}

impl<R> KnowledgeEngineProviderBindingService<R>
where
    R: KnowledgeEngineRegistry + 'static,
{
    pub fn new(store: Arc<dyn KnowledgeEngineProviderBindingStore>, registry: Arc<R>) -> Self {
        Self { store, registry }
    }

    pub async fn create_credential_reference(
        &self,
        context: &KnowledgeEngineExecutionContext,
        request: CreateKnowledgeEngineProviderCredentialReferenceRequest,
    ) -> Result<
        KnowledgeEngineProviderCredentialReference,
        KnowledgeEngineProviderBindingServiceError,
    > {
        validate_management_context(context, None)?;
        self.require_executable_external_implementation(&request.implementation_id)?;
        self.store
            .create_credential_reference(provider_scope(context), &context.actor_id, request)
            .await
            .map_err(Into::into)
    }

    pub async fn create_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        request: CreateKnowledgeEngineProviderBindingRequest,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        validate_management_context(context, Some(request.space_id))?;
        self.require_executable_external_implementation(&request.implementation_id)?;
        self.store
            .create_binding(provider_scope(context), &context.actor_id, request)
            .await
            .map_err(Into::into)
    }

    pub async fn get_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        binding_id: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        validate_management_context(context, None)?;
        let binding = self
            .store
            .get_binding(provider_scope(context), binding_id)
            .await?;
        require_space_scope(context, binding.space_id)?;
        Ok(binding)
    }

    pub async fn list_bindings(
        &self,
        context: &KnowledgeEngineExecutionContext,
        request: ListKnowledgeEngineProviderBindingsRequest,
    ) -> Result<KnowledgeEngineProviderBindingList, KnowledgeEngineProviderBindingServiceError>
    {
        let space_id = request.space_id.ok_or_else(|| {
            KnowledgeEngineProviderBindingServiceError::InvalidRequest(
                "space_id is required for paginated Provider binding lists".to_string(),
            )
        })?;
        validate_management_context(context, Some(space_id))?;
        self.store
            .list_bindings(provider_scope(context), request)
            .await
            .map_err(Into::into)
    }

    pub async fn update_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        binding_id: u64,
        request: UpdateKnowledgeEngineProviderBindingRequest,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        let binding = self.get_binding(context, binding_id).await?;
        validate_management_context(context, Some(binding.space_id))?;
        self.store
            .update_draft_binding(
                provider_scope(context),
                binding_id,
                &context.actor_id,
                request,
            )
            .await
            .map_err(Into::into)
    }

    pub async fn test_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        binding_id: u64,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        let current = self.get_binding(context, binding_id).await?;
        validate_management_context(context, Some(current.space_id))?;
        let testing = self
            .store
            .begin_binding_test(
                provider_scope(context),
                binding_id,
                &context.actor_id,
                expected_version,
            )
            .await?;
        let engine = self
            .registry
            .resolve_by_id(&testing.implementation_id)
            .map_err(KnowledgeEngineProviderBindingServiceError::Engine)?
            .bind_provider(&testing)
            .map_err(KnowledgeEngineProviderBindingServiceError::Engine)?;
        let descriptor = engine.descriptor();
        let health = engine
            .health()
            .await
            .map_err(KnowledgeEngineProviderBindingServiceError::Engine)?;
        let (capabilities, error_category) = match health.status {
            KnowledgeEngineHealthStatus::Available => (descriptor.capabilities, None),
            KnowledgeEngineHealthStatus::Degraded | KnowledgeEngineHealthStatus::Unavailable => (
                Vec::new(),
                Some(KnowledgeEngineProviderErrorCategory::Unavailable),
            ),
        };
        self.store
            .record_binding_test_result(
                provider_scope(context),
                binding_id,
                RecordKnowledgeEngineProviderTestResult {
                    expected_version: testing.version,
                    capabilities,
                    error_category,
                    updated_by: context.actor_id.clone(),
                },
            )
            .await
            .map_err(Into::into)
    }

    pub async fn activate_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        binding_id: u64,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        let binding = self.get_binding(context, binding_id).await?;
        validate_management_context(context, Some(binding.space_id))?;
        if !binding
            .capability_snapshot
            .contains(&KnowledgeEngineCapability::Health)
            || !binding
                .capability_snapshot
                .contains(&KnowledgeEngineCapability::Search)
        {
            return Err(
                KnowledgeEngineProviderBindingServiceError::InvalidLifecycle(
                    "Provider activation requires tested health and search capabilities"
                        .to_string(),
                ),
            );
        }
        self.store
            .activate_binding(
                provider_scope(context),
                binding_id,
                &context.actor_id,
                expected_version,
            )
            .await
            .map_err(Into::into)
    }

    pub async fn disable_binding(
        &self,
        context: &KnowledgeEngineExecutionContext,
        binding_id: u64,
        expected_version: u64,
    ) -> Result<KnowledgeEngineProviderBinding, KnowledgeEngineProviderBindingServiceError> {
        let binding = self.get_binding(context, binding_id).await?;
        validate_management_context(context, Some(binding.space_id))?;
        self.store
            .disable_binding(
                provider_scope(context),
                binding_id,
                &context.actor_id,
                expected_version,
            )
            .await
            .map_err(Into::into)
    }

    fn require_executable_external_implementation(
        &self,
        implementation_id: &str,
    ) -> Result<(), KnowledgeEngineProviderBindingServiceError> {
        let engine = self
            .registry
            .resolve_by_id(implementation_id)
            .map_err(KnowledgeEngineProviderBindingServiceError::Engine)?;
        let descriptor = engine.descriptor();
        if descriptor.native
            || !descriptor.supports(KnowledgeEngineCapability::Health)
            || !descriptor.supports(KnowledgeEngineCapability::Search)
        {
            return Err(KnowledgeEngineProviderBindingServiceError::InvalidRequest(
                format!(
                    "implementation_id={implementation_id} is not an executable external Provider"
                ),
            ));
        }
        Ok(())
    }
}

fn validate_management_context(
    context: &KnowledgeEngineExecutionContext,
    space_id: Option<u64>,
) -> Result<(), KnowledgeEngineProviderBindingServiceError> {
    if context.tenant_id == 0 {
        return Err(
            KnowledgeEngineProviderBindingServiceError::PermissionDenied(
                "tenant scope is required".to_string(),
            ),
        );
    }
    if context.actor_id.trim().is_empty() {
        return Err(
            KnowledgeEngineProviderBindingServiceError::PermissionDenied(
                "authenticated actor is required".to_string(),
            ),
        );
    }
    let may_manage = context.permission_scope.iter().any(|permission| {
        matches!(
            permission.as_str(),
            KNOWLEDGE_PROVIDER_MANAGE_PERMISSION | "knowledge.admin" | "knowledge.*"
        )
    });
    if !may_manage {
        return Err(
            KnowledgeEngineProviderBindingServiceError::PermissionDenied(format!(
                "{KNOWLEDGE_PROVIDER_MANAGE_PERMISSION} is required"
            )),
        );
    }
    if context.trace_id.trim().is_empty() {
        return Err(KnowledgeEngineProviderBindingServiceError::InvalidRequest(
            "trace_id is required".to_string(),
        ));
    }
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| KnowledgeEngineProviderBindingServiceError::Internal(error.to_string()))?
        .as_millis();
    if u128::from(context.deadline_unix_ms) <= now_ms {
        return Err(KnowledgeEngineProviderBindingServiceError::DeadlineExceeded);
    }
    if context.space_id != 0 {
        require_space_scope(context, context.space_id)?;
    }
    if let Some(space_id) = space_id {
        require_space_scope(context, space_id)?;
    }
    Ok(())
}

fn require_space_scope(
    context: &KnowledgeEngineExecutionContext,
    space_id: u64,
) -> Result<(), KnowledgeEngineProviderBindingServiceError> {
    if context.data_scope.allowed_space_ids.is_empty()
        || !context.data_scope.allowed_space_ids.contains(&space_id)
    {
        return Err(
            KnowledgeEngineProviderBindingServiceError::PermissionDenied(format!(
                "space_id={space_id} is outside the authenticated data scope"
            )),
        );
    }
    Ok(())
}

fn provider_scope(context: &KnowledgeEngineExecutionContext) -> KnowledgeEngineProviderScope {
    KnowledgeEngineProviderScope {
        tenant_id: context.tenant_id,
        organization_id: context.organization_id,
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeEngineProviderBindingServiceError {
    #[error("invalid Provider binding request: {0}")]
    InvalidRequest(String),
    #[error("Provider binding permission denied: {0}")]
    PermissionDenied(String),
    #[error("Provider binding request deadline exceeded")]
    DeadlineExceeded,
    #[error("invalid Provider binding lifecycle: {0}")]
    InvalidLifecycle(String),
    #[error(transparent)]
    Store(#[from] KnowledgeEngineProviderBindingStoreError),
    #[error(transparent)]
    Engine(#[from] KnowledgeEngineError),
    #[error("Provider binding internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineDataScope;

    #[test]
    fn management_context_requires_permission_and_exact_space_scope() {
        let mut context = valid_context();
        context.permission_scope.clear();
        assert!(matches!(
            validate_management_context(&context, Some(42)),
            Err(KnowledgeEngineProviderBindingServiceError::PermissionDenied(_))
        ));

        let mut context = valid_context();
        context.data_scope.allowed_space_ids = vec![7];
        assert!(matches!(
            validate_management_context(&context, Some(42)),
            Err(KnowledgeEngineProviderBindingServiceError::PermissionDenied(_))
        ));
    }

    #[test]
    fn management_context_rejects_expired_deadline_before_store_access() {
        let mut context = valid_context();
        context.deadline_unix_ms = 1;
        assert!(matches!(
            validate_management_context(&context, Some(42)),
            Err(KnowledgeEngineProviderBindingServiceError::DeadlineExceeded)
        ));
    }

    fn valid_context() -> KnowledgeEngineExecutionContext {
        let deadline_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis() as u64
            + 60_000;
        KnowledgeEngineExecutionContext {
            tenant_id: 1,
            organization_id: 7,
            actor_id: "tenant-admin".to_string(),
            permission_scope: vec![KNOWLEDGE_PROVIDER_MANAGE_PERMISSION.to_string()],
            data_scope: KnowledgeEngineDataScope {
                allowed_space_ids: vec![42],
                allowed_source_ids: Vec::new(),
                allowed_document_ids: Vec::new(),
            },
            space_id: 42,
            binding_id: None,
            trace_id: "trace-provider-test".to_string(),
            deadline_unix_ms,
        }
    }
}
