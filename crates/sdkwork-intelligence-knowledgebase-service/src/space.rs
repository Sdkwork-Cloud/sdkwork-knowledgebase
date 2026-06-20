use crate::okf::{OkfBundleInitializerService, OkfBundleInitializerServiceError};
use crate::ports::{
    knowledge_access_control::{
        KnowledgeAccessControl, KnowledgeAccessControlError, KnowledgeAccessRole,
        KnowledgeSubjectType,
    },
    knowledge_drive_space::{
        CreateKnowledgeDriveSpaceRequest, DeleteKnowledgeDriveSpaceRequest,
        KnowledgeDriveSpaceProvisioner, KnowledgeDriveSpaceProvisionerError,
    },
    knowledge_space_store::{
        CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
    },
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::space::{CreateKnowledgeSpaceRequest, KnowledgeSpace};
use thiserror::Error;

pub struct KnowledgeSpaceService<'a> {
    store: &'a dyn KnowledgeSpaceStore,
    okf_bundle_initializer: &'a OkfBundleInitializerService<'a>,
    drive_space_provisioner: Option<&'a dyn KnowledgeDriveSpaceProvisioner>,
    access_control: Option<&'a dyn KnowledgeAccessControl>,
    drive_context: Option<KnowledgeSpaceDriveContext>,
}

impl<'a> KnowledgeSpaceService<'a> {
    pub fn new(
        store: &'a dyn KnowledgeSpaceStore,
        okf_bundle_initializer: &'a OkfBundleInitializerService<'a>,
    ) -> Self {
        Self {
            store,
            okf_bundle_initializer,
            drive_space_provisioner: None,
            access_control: None,
            drive_context: None,
        }
    }

    pub fn with_drive_context(
        mut self,
        tenant_id: impl Into<String>,
        operator_id: impl Into<String>,
    ) -> Self {
        self.drive_context = Some(KnowledgeSpaceDriveContext {
            tenant_id: tenant_id.into().trim().to_string(),
            operator_id: operator_id.into().trim().to_string(),
        });
        self
    }

    pub fn with_drive_space_provisioner(
        mut self,
        drive_space_provisioner: &'a dyn KnowledgeDriveSpaceProvisioner,
    ) -> Self {
        self.drive_space_provisioner = Some(drive_space_provisioner);
        self
    }

    pub fn with_access_control(mut self, access_control: &'a dyn KnowledgeAccessControl) -> Self {
        self.access_control = Some(access_control);
        self
    }

    pub async fn create_space(
        &self,
        request: CreateKnowledgeSpaceRequest,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        if request.name.trim().is_empty() {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "name is required".to_string(),
            ));
        }

        let owner_subject_type = request
            .owner_subject_type
            .unwrap_or_else(|| "user".to_string());
        let owner_subject_id = request
            .owner_subject_id
            .unwrap_or_else(|| "unknown".to_string());

        let drive_context = if self.drive_space_provisioner.is_some() {
            Some(self.require_drive_context()?.clone())
        } else {
            if self.okf_bundle_initializer.requires_drive_space_binding() {
                return Err(KnowledgeSpaceServiceError::InvalidRequest(
                    "drive_space_id is required when drive workspace initialization is enabled"
                        .to_string(),
                ));
            }
            None
        };

        let space = self
            .store
            .create_space(CreateKnowledgeSpaceRecord {
                name: request.name,
                description: request.description,
                okf_bundle_initialized: false,
                knowledge_mode: request.knowledge_mode,
            })
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let space_id = space.id;
        let space = match self
            .initialize_created_space(
                space,
                drive_context.as_ref(),
                &owner_subject_type,
                &owner_subject_id,
            )
            .await
        {
            Ok(space) => space,
            Err(error) => return Err(self.cleanup_created_space(space_id, error).await),
        };

        Ok(space)
    }

    pub async fn get_space_with_access_check(
        &self,
        space_id: u64,
        tenant_id: &str,
        actor_id: &str,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        let space = self
            .store
            .get_space(space_id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        if let (Some(access), Some(drive_space_id)) =
            (self.access_control, space.drive_space_id.as_ref())
        {
            let grant = access
                .check_space_access(
                    crate::ports::knowledge_access_control::KnowledgeAccessCheckRequest {
                        tenant_id: tenant_id.to_string(),
                        actor_id: actor_id.to_string(),
                        drive_space_id: drive_space_id.clone(),
                        required_role: KnowledgeAccessRole::Reader,
                    },
                )
                .await
                .map_err(KnowledgeSpaceServiceError::AccessControl)?;
            if !grant.allowed {
                return Err(KnowledgeSpaceServiceError::AccessDenied(format!(
                    "actor {actor_id} does not have access to space {space_id}"
                )));
            }
        }

        Ok(space)
    }

    pub async fn grant_space_member(
        &self,
        space_id: u64,
        tenant_id: &str,
        subject_type: KnowledgeSubjectType,
        subject_id: &str,
        role: KnowledgeAccessRole,
        operator_id: &str,
    ) -> Result<(), KnowledgeSpaceServiceError> {
        let space = self
            .store
            .get_space(space_id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let drive_space_id = space.drive_space_id.as_ref().ok_or_else(|| {
            KnowledgeSpaceServiceError::InvalidRequest(
                "space is not bound to a drive space".to_string(),
            )
        })?;

        let access = self.require_access_control()?;
        access
            .grant_space_access(
                crate::ports::knowledge_access_control::GrantKnowledgeSpaceAccessRequest {
                    tenant_id: tenant_id.to_string(),
                    drive_space_id: drive_space_id.clone(),
                    drive_node_id: None,
                    subject_type,
                    subject_id: subject_id.to_string(),
                    role,
                    operator_id: operator_id.to_string(),
                },
            )
            .await
            .map_err(KnowledgeSpaceServiceError::AccessControl)?;

        Ok(())
    }

    pub async fn revoke_space_member(
        &self,
        space_id: u64,
        tenant_id: &str,
        subject_type: KnowledgeSubjectType,
        subject_id: &str,
        operator_id: &str,
    ) -> Result<(), KnowledgeSpaceServiceError> {
        let space = self
            .store
            .get_space(space_id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let drive_space_id = space.drive_space_id.as_ref().ok_or_else(|| {
            KnowledgeSpaceServiceError::InvalidRequest(
                "space is not bound to a drive space".to_string(),
            )
        })?;

        let access = self.require_access_control()?;
        access
            .revoke_space_access(
                crate::ports::knowledge_access_control::RevokeKnowledgeSpaceAccessRequest {
                    tenant_id: tenant_id.to_string(),
                    drive_space_id: drive_space_id.clone(),
                    drive_node_id: None,
                    subject_type,
                    subject_id: subject_id.to_string(),
                    operator_id: operator_id.to_string(),
                },
            )
            .await
            .map_err(KnowledgeSpaceServiceError::AccessControl)?;

        Ok(())
    }

    pub async fn list_space_members(
        &self,
        space_id: u64,
        tenant_id: &str,
    ) -> Result<
        crate::ports::knowledge_access_control::KnowledgeSpaceMemberList,
        KnowledgeSpaceServiceError,
    > {
        let space = self
            .store
            .get_space(space_id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let drive_space_id = space.drive_space_id.as_ref().ok_or_else(|| {
            KnowledgeSpaceServiceError::InvalidRequest(
                "space is not bound to a drive space".to_string(),
            )
        })?;

        let access = self.require_access_control()?;
        let members = access
            .list_space_members(
                crate::ports::knowledge_access_control::ListKnowledgeSpaceMembersRequest {
                    tenant_id: tenant_id.to_string(),
                    drive_space_id: drive_space_id.clone(),
                    drive_node_id: None,
                    cursor: None,
                    page_size: None,
                },
            )
            .await
            .map_err(KnowledgeSpaceServiceError::AccessControl)?;

        Ok(members)
    }

    async fn initialize_created_space(
        &self,
        mut space: KnowledgeSpace,
        drive_context: Option<&KnowledgeSpaceDriveContext>,
        owner_subject_type: &str,
        owner_subject_id: &str,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        let mut drive_cleanup = None;
        if let Some(provisioner) = self.drive_space_provisioner {
            let drive_context = drive_context.ok_or_else(|| {
                KnowledgeSpaceServiceError::InvalidRequest(
                    "drive tenant_id and operator_id are required when drive space provisioning is enabled"
                        .to_string(),
                )
            })?;
            let binding = provisioner
                .create_knowledge_drive_space(CreateKnowledgeDriveSpaceRequest {
                    tenant_id: drive_context.tenant_id.clone(),
                    knowledge_space_id: space.id,
                    knowledge_space_uuid: space.uuid.clone(),
                    display_name: space.name.clone(),
                    owner_subject_type: owner_subject_type.to_string(),
                    owner_subject_id: owner_subject_id.to_string(),
                    operator_id: drive_context.operator_id.clone(),
                })
                .await?;
            drive_cleanup = Some(DeleteKnowledgeDriveSpaceRequest {
                tenant_id: drive_context.tenant_id.clone(),
                drive_space_id: binding.drive_space_id.clone(),
                owner_subject_type: owner_subject_type.to_string(),
                owner_subject_id: owner_subject_id.to_string(),
                operator_id: drive_context.operator_id.clone(),
            });

            space = match self
                .store
                .mark_drive_space_bound(space.id, binding.drive_space_id)
                .await
                .map_err(KnowledgeSpaceServiceError::Store)
            {
                Ok(space) => space,
                Err(error) => {
                    return Err(self
                        .cleanup_created_drive_space(drive_cleanup.as_ref(), error)
                        .await)
                }
            };
        }

        if space.knowledge_mode == KnowledgeAgentKnowledgeMode::OkfBundle {
            if let Err(error) = self
                .okf_bundle_initializer
                .initialize_standard_files(space.id, &space.name, space.drive_space_id.as_deref())
                .await
                .map_err(KnowledgeSpaceServiceError::OkfBundleInitializer)
            {
                return Err(self
                    .cleanup_created_drive_space(drive_cleanup.as_ref(), error)
                    .await);
            }

            return match self
                .store
                .mark_okf_bundle_initialized(space.id)
                .await
                .map_err(KnowledgeSpaceServiceError::Store)
            {
                Ok(space) => Ok(space),
                Err(error) => Err(self
                    .cleanup_created_drive_space(drive_cleanup.as_ref(), error)
                    .await),
            };
        }

        Ok(space)
    }

    async fn cleanup_created_drive_space(
        &self,
        request: Option<&DeleteKnowledgeDriveSpaceRequest>,
        error: KnowledgeSpaceServiceError,
    ) -> KnowledgeSpaceServiceError {
        let Some(request) = request else {
            return error;
        };
        let Some(provisioner) = self.drive_space_provisioner else {
            return error;
        };
        match provisioner
            .delete_knowledge_drive_space(request.clone())
            .await
        {
            Ok(()) => error,
            Err(cleanup) => KnowledgeSpaceServiceError::DriveSpaceCleanup {
                original: error.to_string(),
                cleanup,
            },
        }
    }

    async fn cleanup_created_space(
        &self,
        space_id: u64,
        error: KnowledgeSpaceServiceError,
    ) -> KnowledgeSpaceServiceError {
        match self.store.mark_space_deleted(space_id).await {
            Ok(()) => error,
            Err(cleanup) => KnowledgeSpaceServiceError::InitializationCleanup {
                original: error.to_string(),
                cleanup,
            },
        }
    }

    fn require_drive_context(
        &self,
    ) -> Result<&KnowledgeSpaceDriveContext, KnowledgeSpaceServiceError> {
        let Some(context) = self.drive_context.as_ref() else {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "drive tenant_id and operator_id are required when drive space provisioning is enabled"
                    .to_string(),
            ));
        };
        if context.tenant_id.trim().is_empty() {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "drive tenant_id is required".to_string(),
            ));
        }
        if context.operator_id.trim().is_empty() {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "drive operator_id is required".to_string(),
            ));
        }
        Ok(context)
    }

    fn require_access_control(
        &self,
    ) -> Result<&dyn KnowledgeAccessControl, KnowledgeSpaceServiceError> {
        self.access_control.ok_or_else(|| {
            KnowledgeSpaceServiceError::InvalidRequest(
                "access control is required for this operation".to_string(),
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSpaceDriveContext {
    pub tenant_id: String,
    pub operator_id: String,
}

#[derive(Debug, Error)]
pub enum KnowledgeSpaceServiceError {
    #[error("invalid knowledge space request: {0}")]
    InvalidRequest(String),
    #[error("knowledge space access denied: {0}")]
    AccessDenied(String),
    #[error(transparent)]
    Store(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    OkfBundleInitializer(#[from] OkfBundleInitializerServiceError),
    #[error(transparent)]
    DriveSpaceProvisioner(#[from] KnowledgeDriveSpaceProvisionerError),
    #[error(transparent)]
    AccessControl(#[from] KnowledgeAccessControlError),
    #[error("knowledge space initialization failed: {original}; cleanup failed: {cleanup}")]
    InitializationCleanup {
        original: String,
        cleanup: KnowledgeSpaceStoreError,
    },
    #[error("knowledge space initialization failed: {original}; drive cleanup failed: {cleanup}")]
    DriveSpaceCleanup {
        original: String,
        cleanup: KnowledgeDriveSpaceProvisionerError,
    },
}
