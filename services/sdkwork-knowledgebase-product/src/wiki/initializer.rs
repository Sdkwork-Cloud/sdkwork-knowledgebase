use super::{
    KnowledgeWikiFileRegistryService, LlmWikiStandardFileService, PersistStandardFilesRequest,
    PersistedStandardFiles,
};
use crate::ports::knowledge_drive_storage::{KnowledgeDriveStorage, KnowledgeStorageError};
use thiserror::Error;

pub struct KnowledgeWikiInitializerService<'a> {
    standard_files: LlmWikiStandardFileService<'a>,
    registry: Option<&'a KnowledgeWikiFileRegistryService<'a>>,
}

impl<'a> KnowledgeWikiInitializerService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self {
            standard_files: LlmWikiStandardFileService::new(drive),
            registry: None,
        }
    }

    pub fn with_registry(mut self, registry: &'a KnowledgeWikiFileRegistryService<'a>) -> Self {
        self.registry = Some(registry);
        self
    }

    pub async fn initialize_standard_files(
        &self,
        space_id: u64,
        space_name: &str,
    ) -> Result<PersistedStandardFiles, KnowledgeWikiInitializerServiceError> {
        let files = self
            .standard_files
            .persist_standard_files(PersistStandardFilesRequest {
                space_name: space_name.to_string(),
                pages: vec![],
                log_entries: vec![],
            })
            .await
            .map_err(KnowledgeWikiInitializerServiceError::Storage)?;

        if let Some(registry) = self.registry {
            registry.register_standard_files(space_id, &files).await?;
        }

        Ok(files)
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiInitializerServiceError {
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    Registry(#[from] super::KnowledgeWikiFileRegistryServiceError),
}
