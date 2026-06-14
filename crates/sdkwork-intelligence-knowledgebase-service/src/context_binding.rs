use crate::ports::knowledge_context_binding_store::{
    KnowledgeContextBindingStore, KnowledgeContextBindingStoreError,
};
use crate::ports::knowledge_drive_permission::{
    GrantDrivePermissionRequest, KnowledgeDrivePermissionError, KnowledgeDrivePermissionProvider,
    RevokeDrivePermissionRequest,
};
use sdkwork_knowledgebase_contract::context_binding::{
    CreateKnowledgeSpaceContextBindingRequest, KnowledgeAccessLevel, KnowledgeContextType,
    KnowledgeSpaceContextBinding, KnowledgeSpaceContextBindingList,
    ListContextBoundSpacesRequest, ListKnowledgeSpaceContextBindingsRequest,
    UpdateKnowledgeSpaceContextBindingRequest,
};
use thiserror::Error;

pub struct KnowledgeContextBindingService<'a> {
    store: &'a dyn KnowledgeContextBindingStore,
    drive_permissions: Option<&'a dyn KnowledgeDrivePermissionProvider>,
}

impl<'a> KnowledgeContextBindingService<'a> {
    pub fn new(store: &'a dyn KnowledgeContextBindingStore) -> Self {
        Self {
            store,
            drive_permissions: None,
        }
    }

    pub fn with_drive_permissions(
        mut self,
        drive_permissions: &'a dyn KnowledgeDrivePermissionProvider,
    ) -> Self {
        self.drive_permissions = Some(drive_permissions);
        self
    }

    pub async fn bind_context(
        &self,
        tenant_id: u64,
        created_by: &str,
        drive_space_id: &str,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingServiceError> {
        if request.context_id.trim().is_empty() {
            return Err(KnowledgeContextBindingServiceError::InvalidRequest(
                "context_id is required".to_string(),
            ));
        }

        let access_level = request
            .access_level
            .unwrap_or(KnowledgeAccessLevel::Reader);

        let binding = self
            .store
            .create_binding(tenant_id, created_by, request)
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)?;

        if let Some(drive_perms) = self.drive_permissions {
            let grant = GrantDrivePermissionRequest {
                tenant_id: tenant_id.to_string(),
                drive_space_id: drive_space_id.to_string(),
                subject_type: "group".to_string(),
                subject_id: format!(
                    "{}:{}",
                    binding.context_type.as_str(),
                    binding.context_id
                ),
                role: access_level.as_str().to_string(),
                operator_id: created_by.to_string(),
            };

            if let Err(_e) = drive_perms.grant_space_access(grant).await {
                // Permission grant is best-effort; binding is already persisted.
                // The caller can retry permission sync separately.
            }
        }

        Ok(binding)
    }

    pub async fn unbind_context(
        &self,
        tenant_id: u64,
        binding_id: u64,
        drive_space_id: &str,
        operator_id: &str,
    ) -> Result<(), KnowledgeContextBindingServiceError> {
        let binding = self
            .store
            .get_binding(tenant_id, binding_id)
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)?;

        self.store
            .delete_binding(tenant_id, binding_id)
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)?;

        if let Some(drive_perms) = self.drive_permissions {
            let revoke = RevokeDrivePermissionRequest {
                tenant_id: tenant_id.to_string(),
                drive_space_id: drive_space_id.to_string(),
                subject_type: "group".to_string(),
                subject_id: format!(
                    "{}:{}",
                    binding.context_type.as_str(),
                    binding.context_id
                ),
                operator_id: operator_id.to_string(),
            };

            if let Err(_e) = drive_perms.revoke_space_access(revoke).await {
                // Permission revocation is best-effort; binding deletion is already persisted.
            }
        }

        Ok(())
    }

    pub async fn update_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingServiceError> {
        self.store
            .update_binding(tenant_id, binding_id, request)
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)
    }

    pub async fn get_binding(
        &self,
        tenant_id: u64,
        binding_id: u64,
    ) -> Result<KnowledgeSpaceContextBinding, KnowledgeContextBindingServiceError> {
        self.store
            .get_binding(tenant_id, binding_id)
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)
    }

    pub async fn list_space_bindings(
        &self,
        tenant_id: u64,
        space_id: u64,
        context_type: Option<KnowledgeContextType>,
    ) -> Result<KnowledgeSpaceContextBindingList, KnowledgeContextBindingServiceError> {
        self.store
            .list_space_bindings(
                tenant_id,
                ListKnowledgeSpaceContextBindingsRequest {
                    space_id,
                    context_type,
                    cursor: None,
                    page_size: None,
                },
            )
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)
    }

    pub async fn list_context_bound_spaces(
        &self,
        tenant_id: u64,
        context_type: KnowledgeContextType,
        context_id: &str,
    ) -> Result<Vec<u64>, KnowledgeContextBindingServiceError> {
        if context_id.trim().is_empty() {
            return Err(KnowledgeContextBindingServiceError::InvalidRequest(
                "context_id is required".to_string(),
            ));
        }

        self.store
            .list_context_bound_spaces(
                tenant_id,
                ListContextBoundSpacesRequest {
                    context_type,
                    context_id: context_id.to_string(),
                    cursor: None,
                    page_size: None,
                },
            )
            .await
            .map_err(KnowledgeContextBindingServiceError::Store)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeContextBindingServiceError {
    #[error("invalid context binding request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Store(#[from] KnowledgeContextBindingStoreError),
    #[error(transparent)]
    DrivePermission(#[from] KnowledgeDrivePermissionError),
}
