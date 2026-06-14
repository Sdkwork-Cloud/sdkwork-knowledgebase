use super::{
    KnowledgeWikiFileRegistryService, LlmWikiStandardFileService, PersistStandardFilesRequest,
    PersistedStandardFiles,
};
use crate::ports::{
    knowledge_drive_storage::{KnowledgeDriveStorage, KnowledgeStorageError},
    knowledge_drive_workspace::{
        EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
        EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
    },
};
use thiserror::Error;

pub struct KnowledgeWikiInitializerService<'a> {
    standard_files: LlmWikiStandardFileService<'a>,
    registry: Option<&'a KnowledgeWikiFileRegistryService<'a>>,
    drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
}

impl<'a> KnowledgeWikiInitializerService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self {
            standard_files: LlmWikiStandardFileService::new(drive),
            registry: None,
            drive_workspace: None,
        }
    }

    pub fn with_registry(mut self, registry: &'a KnowledgeWikiFileRegistryService<'a>) -> Self {
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

    pub async fn initialize_standard_files(
        &self,
        space_id: u64,
        space_name: &str,
        drive_space_id: Option<&str>,
    ) -> Result<PersistedStandardFiles, KnowledgeWikiInitializerServiceError> {
        let drive_space_id = self.required_drive_space_id(drive_space_id)?;
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

        if let Some(drive_workspace) = self.drive_workspace {
            drive_workspace
                .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
                    drive_space_id,
                    nodes: standard_drive_nodes(&files),
                })
                .await?;
        }

        Ok(files)
    }

    fn required_drive_space_id(
        &self,
        drive_space_id: Option<&str>,
    ) -> Result<String, KnowledgeWikiInitializerServiceError> {
        if self.drive_workspace.is_none() {
            return Ok(String::new());
        }
        drive_space_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                KnowledgeWikiInitializerServiceError::InvalidRequest(
                    "drive_space_id is required when drive workspace initialization is enabled"
                        .to_string(),
                )
            })
    }
}

fn standard_drive_nodes(files: &PersistedStandardFiles) -> Vec<EnsureKnowledgeDriveNodeRequest> {
    const FOLDERS: &[&str] = &[
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
        "wiki",
        "wiki/schema",
        "wiki/pages",
        "wiki/pages/sources",
        "wiki/pages/entities",
        "wiki/pages/concepts",
        "wiki/pages/topics",
        "wiki/pages/references",
        "wiki/pages/how_to",
        "wiki/pages/faq",
        "wiki/pages/glossary",
        "wiki/pages/answers",
        "wiki/pages/comparisons",
        "wiki/pages/presentations",
        "wiki/pages/charts",
        "wiki/pages/indexes",
        "wiki/pages/policies",
        "wiki/pages/runbooks",
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

    let mut nodes = Vec::with_capacity(FOLDERS.len() + 4);
    nodes.extend(
        FOLDERS
            .iter()
            .map(|logical_path| EnsureKnowledgeDriveNodeRequest {
                logical_path: (*logical_path).to_string(),
                kind: EnsureKnowledgeDriveNodeKind::Folder,
                object_ref: None,
            }),
    );
    nodes.push(file_node(&files.agents_md));
    nodes.push(file_node(&files.schema_yaml));
    nodes.push(file_node(&files.index_md));
    nodes.push(file_node(&files.log_md));
    nodes
}

fn file_node(
    object_ref: &crate::ports::knowledge_drive_storage::KnowledgeObjectRef,
) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: object_ref.logical_path.clone(),
        kind: EnsureKnowledgeDriveNodeKind::File,
        object_ref: Some(object_ref.clone()),
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiInitializerServiceError {
    #[error("invalid knowledge wiki initialization request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] KnowledgeStorageError),
    #[error(transparent)]
    Registry(#[from] super::KnowledgeWikiFileRegistryServiceError),
    #[error(transparent)]
    DriveWorkspace(#[from] KnowledgeDriveWorkspaceError),
}
