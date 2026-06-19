use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    context_binding::KnowledgeContextBindingService,
    ports::knowledge_space_store::KnowledgeSpaceStore,
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeSpaceContextBinding,
    KnowledgeSpaceContextBindingList, UpdateKnowledgeSpaceContextBindingRequest,
};

use crate::{
    runtime::KnowledgebaseRuntime, ApiError, ApiResult, KnowledgeAppRequestContext,
    KnowledgeContextBindingAppService,
};

#[derive(Clone)]
pub(crate) struct HostedContextBindingService {
    runtime: KnowledgebaseRuntime,
}

impl HostedContextBindingService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn created_by(context: &KnowledgeAppRequestContext, fallback: &str) -> String {
        context
            .actor_id
            .map(|actor_id| actor_id.to_string())
            .unwrap_or_else(|| fallback.to_string())
    }

    async fn drive_space_id_for_space(&self, space_id: u64) -> ApiResult<String> {
        let space = self
            .runtime
            .space_store()
            .get_space(space_id)
            .await
            .map_err(ApiError::from)?;
        space.drive_space_id.ok_or_else(|| {
            ApiError::invalid_request(
                "knowledge_space_not_drive_bound",
                format!("knowledge space {space_id} is not bound to a drive space"),
            )
        })
    }
}

#[async_trait]
impl KnowledgeContextBindingAppService for HostedContextBindingService {
    async fn list_space_context_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBindingList> {
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        service
            .list_space_bindings(context.tenant_id, space_id, None)
            .await
            .map_err(Into::into)
    }

    async fn create_space_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        if request.space_id != space_id {
            return Err(ApiError::invalid_request(
                "space_id_mismatch",
                "spaceId in body must match spaceId in path",
            ));
        }

        let drive_space_id = self.drive_space_id_for_space(space_id).await?;
        let created_by = Self::created_by(&context, self.runtime.operator_id());
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        service
            .bind_context(context.tenant_id, &created_by, &drive_space_id, request)
            .await
            .map_err(Into::into)
    }

    async fn retrieve_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        service
            .get_binding(context.tenant_id, binding_id)
            .await
            .map_err(Into::into)
    }

    async fn update_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        service
            .update_binding(context.tenant_id, binding_id, request)
            .await
            .map_err(Into::into)
    }

    async fn delete_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<()> {
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        let binding = service
            .get_binding(context.tenant_id, binding_id)
            .await
            .map_err(ApiError::from)?;
        let drive_space_id = self.drive_space_id_for_space(binding.space_id).await?;
        let operator_id = Self::created_by(&context, self.runtime.operator_id());
        service
            .unbind_context(context.tenant_id, binding_id, &drive_space_id, &operator_id)
            .await
            .map_err(Into::into)
    }
}
