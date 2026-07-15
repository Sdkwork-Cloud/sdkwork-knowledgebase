use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    context_binding::KnowledgeContextBindingService,
    ports::knowledge_access_control::KnowledgeAccessRole,
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeContextType, KnowledgeSpaceContextBinding,
    UpdateKnowledgeSpaceContextBindingRequest,
};
use sdkwork_knowledgebase_contract::space::KnowledgeSpace;
use sdkwork_utils_rust::SdkWorkPageData;

use crate::{
    hosted_access::{require_space_access, require_space_access_with_role},
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeContextBindingAppService,
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

    fn drive_space_id_for_space(space: &KnowledgeSpace) -> ApiResult<String> {
        space.drive_space_id.clone().ok_or_else(|| {
            ApiError::invalid_request(
                "knowledge_space_not_drive_bound",
                format!("knowledge space {} is not bound to a drive space", space.id),
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
        cursor: Option<String>,
        page_size: Option<u32>,
        context_type: Option<KnowledgeContextType>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeSpaceContextBinding>> {
        require_space_access(&self.runtime, &context, space_id).await?;
        let normalized_page_size = crate::pagination::normalize_api_page_size(page_size)?;
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        let listed = service
            .list_space_bindings(
                context.tenant_id,
                space_id,
                context_type,
                cursor,
                Some(normalized_page_size),
            )
            .await
            .map_err(ApiError::from)?;
        Ok(crate::pagination::cursor_page_data(
            listed.items,
            listed.next_cursor.clone(),
            listed.next_cursor.is_some(),
            normalized_page_size,
        ))
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
        let space = require_space_access_with_role(
            &self.runtime,
            &context,
            space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;

        let drive_space_id = Self::drive_space_id_for_space(&space)?;
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
        let binding = service
            .get_binding(context.tenant_id, binding_id)
            .await
            .map_err(ApiError::from)?;
        require_space_access(&self.runtime, &context, binding.space_id).await?;
        Ok(binding)
    }

    async fn update_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        let service = KnowledgeContextBindingService::new(self.runtime.context_binding_store());
        let existing = service
            .get_binding(context.tenant_id, binding_id)
            .await
            .map_err(ApiError::from)?;
        require_space_access_with_role(
            &self.runtime,
            &context,
            existing.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
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
        let space = require_space_access_with_role(
            &self.runtime,
            &context,
            binding.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
        let drive_space_id = Self::drive_space_id_for_space(&space)?;
        let operator_id = Self::created_by(&context, self.runtime.operator_id());
        service
            .unbind_context(context.tenant_id, binding_id, &drive_space_id, &operator_id)
            .await
            .map_err(Into::into)
    }
}
