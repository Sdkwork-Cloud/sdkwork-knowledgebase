use crate::ports::knowledge_drive_storage::KnowledgeObjectRef;
use crate::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};

pub(crate) const DRIVE_PERMISSION_ANCHOR_FOLDER: &str = "workspace";

pub(crate) const DRIVE_WORKSPACE_SYNC_DRIVE_SPACE_REQUIRED: &str =
    "drive_space_id is required when drive workspace synchronization is enabled";

pub(crate) const DRIVE_WORKSPACE_INIT_DRIVE_SPACE_REQUIRED: &str =
    "drive_space_id is required when drive workspace initialization is enabled";

pub(crate) const INITIALIZED_STANDARD_BUNDLE_FOLDERS: &[&str] = &[
    "manifest",
    "inbox",
    "inbox/uploads",
    "inbox/drive-imports",
    "inbox/api",
    "sources",
    "sources/raw",
    "sources/urls",
    "sources/repos",
    "sources/message_archives",
    "sources/media",
    "parsed",
    "okf",
    "okf/schema",
    ".sdkwork",
    ".sdkwork/governance",
    ".sdkwork/governance/revisions",
    "graph",
    "candidates",
    "indexes",
    "datasets",
    "inventory",
    "context_packs",
    "eval",
    "output",
    "output/answers",
    "output/reports",
    "output/decks",
    "output/charts",
    "output/plans",
    "output/study_guides",
    "output/exports",
    "mirror",
    "logs",
];

pub(crate) fn trim_bound_drive_space_id(drive_space_id: Option<&str>) -> Option<String> {
    drive_space_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn governance_revision_path(concept_id: &str, revision_no: u64) -> String {
    format!(".sdkwork/governance/revisions/{concept_id}/r{revision_no}.md")
}

pub(crate) fn governance_revision_drive_nodes(
    concept_id: &str,
    revision_ref: Option<&KnowledgeObjectRef>,
    published_ref: Option<&KnowledgeObjectRef>,
) -> Vec<EnsureKnowledgeDriveNodeRequest> {
    let mut nodes = vec![
        folder_drive_node(".sdkwork/governance/revisions"),
        folder_drive_node(&format!(".sdkwork/governance/revisions/{concept_id}")),
    ];
    if let Some(revision_ref) = revision_ref {
        nodes.push(file_drive_node(revision_ref));
    }
    if let Some(published_ref) = published_ref {
        nodes.push(file_drive_node(published_ref));
    }
    nodes
}

pub(crate) fn file_drive_node(object_ref: &KnowledgeObjectRef) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: object_ref.logical_path.clone(),
        kind: EnsureKnowledgeDriveNodeKind::File,
        object_ref: Some(object_ref.clone()),
    }
}

pub(crate) fn folder_drive_node(logical_path: &str) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::Folder,
        object_ref: None,
    }
}

pub(crate) async fn ensure_drive_workspace_nodes(
    drive_workspace: &dyn KnowledgeDriveWorkspace,
    drive_space_id: Option<&str>,
    nodes: Vec<EnsureKnowledgeDriveNodeRequest>,
) -> Result<(), KnowledgeDriveWorkspaceError> {
    if nodes.is_empty() {
        return Ok(());
    }
    let drive_space_id = trim_bound_drive_space_id(drive_space_id).ok_or_else(|| {
        KnowledgeDriveWorkspaceError::InvalidRequest(
            DRIVE_WORKSPACE_SYNC_DRIVE_SPACE_REQUIRED.to_string(),
        )
    })?;
    drive_workspace
        .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
            drive_space_id,
            nodes,
        })
        .await
}

pub(crate) async fn ensure_drive_permission_anchor(
    drive_workspace: &dyn KnowledgeDriveWorkspace,
    drive_space_id: Option<&str>,
) -> Result<(), KnowledgeDriveWorkspaceError> {
    ensure_drive_workspace_nodes(
        drive_workspace,
        drive_space_id,
        vec![folder_drive_node(DRIVE_PERMISSION_ANCHOR_FOLDER)],
    )
    .await
}
