use crate::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, DeleteKnowledgeDriveSpaceRequest,
    KnowledgeDriveSpaceProvisioner, KnowledgeDriveSpaceProvisionerError,
};
use crate::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use crate::wiki::{KnowledgeWikiInitializerService, KnowledgeWikiInitializerServiceError};
use sdkwork_knowledgebase_contract::space::{CreateKnowledgeSpaceRequest, KnowledgeSpace};
use thiserror::Error;

pub struct KnowledgeSpaceService<'a> {
    store: &'a dyn KnowledgeSpaceStore,
    wiki_initializer: &'a KnowledgeWikiInitializerService<'a>,
    drive_space_provisioner: Option<&'a dyn KnowledgeDriveSpaceProvisioner>,
    drive_context: Option<KnowledgeSpaceDriveContext>,
}

impl<'a> KnowledgeSpaceService<'a> {
    pub fn new(
        store: &'a dyn KnowledgeSpaceStore,
        wiki_initializer: &'a KnowledgeWikiInitializerService<'a>,
    ) -> Self {
        Self {
            store,
            wiki_initializer,
            drive_space_provisioner: None,
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

    pub async fn create_space(
        &self,
        request: CreateKnowledgeSpaceRequest,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        if request.name.trim().is_empty() {
            return Err(KnowledgeSpaceServiceError::InvalidRequest(
                "name is required".to_string(),
            ));
        }

        let drive_context = if self.drive_space_provisioner.is_some() {
            Some(self.require_drive_context()?.clone())
        } else {
            if self.wiki_initializer.requires_drive_space_binding() {
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
                llm_wiki_initialized: false,
            })
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let space_id = space.id;
        let space = match self
            .initialize_created_space(space, drive_context.as_ref())
            .await
        {
            Ok(space) => space,
            Err(error) => return Err(self.cleanup_created_space(space_id, error).await),
        };

        Ok(space)
    }

    async fn initialize_created_space(
        &self,
        mut space: KnowledgeSpace,
        drive_context: Option<&KnowledgeSpaceDriveContext>,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceServiceError> {
        let mut drive_cleanup = None;
        if let Some(provisioner) = self.drive_space_provisioner {
            let drive_context = drive_context.ok_or_else(|| {
                KnowledgeSpaceServiceError::InvalidRequest(
                    "drive tenant_id and operator_id are required when drive space provisioning is enabled"
                        .to_string(),
                )
            })?;
            let owner_subject_type = "app".to_string();
            let owner_subject_id = format!("sdkwork-knowledgebase:{}", space.uuid);
            let binding = provisioner
                .create_knowledge_drive_space(CreateKnowledgeDriveSpaceRequest {
                    tenant_id: drive_context.tenant_id.clone(),
                    knowledge_space_id: space.id,
                    knowledge_space_uuid: space.uuid.clone(),
                    display_name: space.name.clone(),
                    owner_subject_type: owner_subject_type.clone(),
                    owner_subject_id: owner_subject_id.clone(),
                    operator_id: drive_context.operator_id.clone(),
                })
                .await?;
            drive_cleanup = Some(DeleteKnowledgeDriveSpaceRequest {
                tenant_id: drive_context.tenant_id.clone(),
                drive_space_id: binding.drive_space_id.clone(),
                owner_subject_type,
                owner_subject_id,
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

        if let Err(error) = self
            .wiki_initializer
            .initialize_standard_files(space.id, &space.name, space.drive_space_id.as_deref())
            .await
            .map_err(KnowledgeSpaceServiceError::WikiInitializer)
        {
            return Err(self
                .cleanup_created_drive_space(drive_cleanup.as_ref(), error)
                .await);
        }

        match self
            .store
            .mark_llm_wiki_initialized(space.id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)
        {
            Ok(space) => Ok(space),
            Err(error) => Err(self
                .cleanup_created_drive_space(drive_cleanup.as_ref(), error)
                .await),
        }
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
    #[error(transparent)]
    Store(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    WikiInitializer(#[from] KnowledgeWikiInitializerServiceError),
    #[error(transparent)]
    DriveSpaceProvisioner(#[from] KnowledgeDriveSpaceProvisionerError),
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
