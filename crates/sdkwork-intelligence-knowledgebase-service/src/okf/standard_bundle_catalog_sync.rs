use super::governance_drive::{
    ensure_drive_workspace_nodes, file_drive_node, folder_drive_node,
    INITIALIZED_STANDARD_BUNDLE_FOLDERS,
};
use super::standard_bundle_refresh::DynamicStandardBundleFiles;
use super::{
    OkfBundleFileRegistryService, OkfBundleFileRegistryServiceError, PersistedStandardFiles,
};
use crate::ports::knowledge_drive_workspace::{
    KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStore;
use sdkwork_knowledgebase_contract::okf_bundle_file::OkfBundleFileKind;
use thiserror::Error;

#[derive(Clone, Copy)]
pub(crate) struct StandardBundleCatalogSyncDeps<'a> {
    pub bundle_file_store: Option<&'a dyn KnowledgeOkfBundleFileStore>,
    pub drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
}

#[derive(Debug, Error)]
pub enum StandardBundleCatalogSyncError {
    #[error(transparent)]
    Registry(#[from] OkfBundleFileRegistryServiceError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
}

pub(crate) async fn sync_full_standard_bundle_catalog(
    deps: StandardBundleCatalogSyncDeps<'_>,
    space_id: u64,
    files: &PersistedStandardFiles,
    drive_space_id: Option<&str>,
) -> Result<(), StandardBundleCatalogSyncError> {
    if let Some(bundle_file_store) = deps.bundle_file_store {
        OkfBundleFileRegistryService::new(bundle_file_store)
            .register_standard_files(space_id, files)
            .await?;
    }

    if let Some(drive_workspace) = deps.drive_workspace {
        ensure_drive_workspace_nodes(
            drive_workspace,
            drive_space_id,
            persisted_standard_file_drive_nodes(files),
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn sync_initialized_standard_bundle_catalog(
    deps: StandardBundleCatalogSyncDeps<'_>,
    space_id: u64,
    files: &PersistedStandardFiles,
    drive_space_id: Option<&str>,
) -> Result<(), StandardBundleCatalogSyncError> {
    if let Some(bundle_file_store) = deps.bundle_file_store {
        OkfBundleFileRegistryService::new(bundle_file_store)
            .register_standard_files(space_id, files)
            .await?;
    }

    if let Some(drive_workspace) = deps.drive_workspace {
        ensure_drive_workspace_nodes(
            drive_workspace,
            drive_space_id,
            initialized_standard_bundle_drive_nodes(files),
        )
        .await?;
    }

    Ok(())
}

pub(crate) async fn sync_dynamic_standard_bundle_catalog(
    deps: StandardBundleCatalogSyncDeps<'_>,
    space_id: u64,
    dynamic: &DynamicStandardBundleFiles,
    drive_space_id: Option<&str>,
) -> Result<(), StandardBundleCatalogSyncError> {
    if let Some(bundle_file_store) = deps.bundle_file_store {
        let registry = OkfBundleFileRegistryService::new(bundle_file_store);
        registry
            .upsert_object_ref_file(
                space_id,
                &dynamic.root_index_md,
                OkfBundleFileKind::BundleIndex,
            )
            .await?;
        registry
            .upsert_object_ref_file(space_id, &dynamic.log_md, OkfBundleFileKind::BundleLog)
            .await?;
    }

    if let Some(drive_workspace) = deps.drive_workspace {
        ensure_drive_workspace_nodes(
            drive_workspace,
            drive_space_id,
            dynamic_standard_bundle_drive_nodes(dynamic),
        )
        .await?;
    }

    Ok(())
}

pub(crate) fn persisted_standard_file_drive_nodes(
    files: &PersistedStandardFiles,
) -> Vec<crate::ports::knowledge_drive_workspace::EnsureKnowledgeDriveNodeRequest> {
    vec![
        file_drive_node(&files.agents_md),
        file_drive_node(&files.profile_yaml),
        file_drive_node(&files.index_md),
        file_drive_node(&files.log_md),
    ]
}

pub(crate) fn initialized_standard_bundle_drive_nodes(
    files: &PersistedStandardFiles,
) -> Vec<crate::ports::knowledge_drive_workspace::EnsureKnowledgeDriveNodeRequest> {
    let mut nodes = Vec::with_capacity(INITIALIZED_STANDARD_BUNDLE_FOLDERS.len() + 4);
    nodes.extend(
        INITIALIZED_STANDARD_BUNDLE_FOLDERS
            .iter()
            .map(|logical_path| folder_drive_node(logical_path)),
    );
    nodes.extend(persisted_standard_file_drive_nodes(files));
    nodes
}

pub(crate) fn dynamic_standard_bundle_drive_nodes(
    dynamic: &DynamicStandardBundleFiles,
) -> Vec<crate::ports::knowledge_drive_workspace::EnsureKnowledgeDriveNodeRequest> {
    let mut nodes = dynamic
        .index_object_refs
        .iter()
        .map(file_drive_node)
        .collect::<Vec<_>>();
    nodes.push(file_drive_node(&dynamic.log_md));
    nodes
}
