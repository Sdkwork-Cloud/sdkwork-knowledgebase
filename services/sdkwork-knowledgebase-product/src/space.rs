use crate::ports::knowledge_drive_space::{
    CreateKnowledgeDriveSpaceRequest, KnowledgeDriveSpaceProvisioner,
    KnowledgeDriveSpaceProvisionerError,
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
        }
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

        let space = self
            .store
            .create_space(CreateKnowledgeSpaceRecord {
                name: request.name,
                description: request.description,
                llm_wiki_initialized: false,
            })
            .await
            .map_err(KnowledgeSpaceServiceError::Store)?;

        let space = if let Some(provisioner) = self.drive_space_provisioner {
            let binding = provisioner
                .create_knowledge_drive_space(CreateKnowledgeDriveSpaceRequest {
                    tenant_id: "default".to_string(),
                    knowledge_space_id: space.id,
                    knowledge_space_uuid: space.uuid.clone(),
                    display_name: space.name.clone(),
                    owner_subject_type: "system".to_string(),
                    owner_subject_id: "system".to_string(),
                    operator_id: "system".to_string(),
                })
                .await?;

            self.store
                .mark_drive_space_bound(space.id, binding.drive_space_id)
                .await
                .map_err(KnowledgeSpaceServiceError::Store)?
        } else {
            space
        };

        self.wiki_initializer
            .initialize_standard_files(space.id, &space.name, space.drive_space_id.as_deref())
            .await?;

        self.store
            .mark_llm_wiki_initialized(space.id)
            .await
            .map_err(KnowledgeSpaceServiceError::Store)
    }
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
}
