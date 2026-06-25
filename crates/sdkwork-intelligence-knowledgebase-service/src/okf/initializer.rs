use super::governance_drive::{
    ensure_drive_permission_anchor, DRIVE_WORKSPACE_INIT_DRIVE_SPACE_REQUIRED,
};
use super::standard_bundle_catalog_sync::{
    sync_initialized_standard_bundle_catalog, StandardBundleCatalogSyncDeps,
    StandardBundleCatalogSyncError,
};
use super::{
    OkfBundleFileRegistryService, OkfBundleStandardFileService, PersistStandardFilesRequest,
    PersistedStandardFiles,
};
use crate::ports::{
    knowledge_drive_storage::{KnowledgeDriveStorage, KnowledgeStorageError},
    knowledge_drive_workspace::{KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError},
};
use thiserror::Error;

pub struct OkfBundleInitializerService<'a> {
    standard_files: OkfBundleStandardFileService<'a>,
    registry: Option<&'a OkfBundleFileRegistryService<'a>>,
    drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
}

impl<'a> OkfBundleInitializerService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self {
            standard_files: OkfBundleStandardFileService::new(drive),
            registry: None,
            drive_workspace: None,
        }
    }

    pub fn with_registry(mut self, registry: &'a OkfBundleFileRegistryService<'a>) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_drive_workspace(
        mut self,
        drive_workspace: &'a dyn KnowledgeDriveWorkspace,
    ) -> Self {
        self.drive_workspace = Some(drive_workspace);
        self
    }

    pub fn requires_drive_space_binding(&self) -> bool {
        self.drive_workspace.is_some()
    }

    /// Ensures a root-level drive folder exists so drive ACL can anchor space permissions.
    pub async fn ensure_drive_permission_anchor(
        &self,
        drive_space_id: Option<&str>,
    ) -> Result<(), OkfBundleInitializerServiceError> {
        if !self.requires_drive_space_binding() {
            return Ok(());
        }
        let drive_space_id = self.bound_drive_space_id(drive_space_id)?;
        let Some(drive_workspace) = self.drive_workspace else {
            return Ok(());
        };
        ensure_drive_permission_anchor(drive_workspace, drive_space_id.as_deref()).await?;
        Ok(())
    }

    pub async fn initialize_standard_files(
        &self,
        space_id: u64,
        space_name: &str,
        drive_space_id: Option<&str>,
    ) -> Result<PersistedStandardFiles, OkfBundleInitializerServiceError> {
        let bound_drive_space_id = self.bound_drive_space_id(drive_space_id)?;
        let files = self
            .standard_files
            .persist_standard_files(PersistStandardFilesRequest {
                space_name: space_name.to_string(),
                concepts: vec![],
                log_entries: vec![],
                drive_space_id: bound_drive_space_id.clone(),
            })
            .await
            .map_err(OkfBundleInitializerServiceError::Storage)?;

        sync_initialized_standard_bundle_catalog(
            StandardBundleCatalogSyncDeps {
                bundle_file_store: self.registry.map(|registry| registry.bundle_file_store()),
                drive_workspace: self.drive_workspace,
            },
            space_id,
            &files,
            bound_drive_space_id.as_deref(),
        )
        .await
        .map_err(OkfBundleInitializerServiceError::CatalogSync)?;

        Ok(files)
    }

    fn bound_drive_space_id(
        &self,
        drive_space_id: Option<&str>,
    ) -> Result<Option<String>, OkfBundleInitializerServiceError> {
        let bound = super::governance_drive::trim_bound_drive_space_id(drive_space_id);
        if self.drive_workspace.is_some() && bound.is_none() {
            return Err(OkfBundleInitializerServiceError::InvalidRequest(
                DRIVE_WORKSPACE_INIT_DRIVE_SPACE_REQUIRED.to_string(),
            ));
        }
        Ok(bound)
    }
}

#[derive(Debug, Error)]
pub enum OkfBundleInitializerServiceError {
    #[error("invalid knowledge okf bundle initialization request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    Registry(#[from] super::OkfBundleFileRegistryServiceError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
    #[error(transparent)]
    CatalogSync(#[from] StandardBundleCatalogSyncError),
}
