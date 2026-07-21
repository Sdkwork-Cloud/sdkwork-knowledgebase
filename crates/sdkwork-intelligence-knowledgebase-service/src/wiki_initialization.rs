use crate::ports::{
    knowledge_drive_node_tree::{
        DriveNodeKind, KnowledgeDriveNodeTree, KnowledgeDriveNodeTreeError,
        ResolveKnowledgeDriveNodePathRequest,
    },
    knowledge_drive_workspace::{
        EnsureKnowledgeDriveNodeKind, EnsureKnowledgeDriveNodeRequest,
        EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
    },
    knowledge_wiki_drive_source::{
        EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveScope, KnowledgeWikiDriveSourceError,
        KnowledgebaseRawScope, KNOWLEDGEBASE_RAW_CONSUMER_KIND,
    },
    knowledge_wiki_persistence::{
        BindWikiSourceScopeRequest, ProvisionWikiDriveCheckpointRequest, WikiDriveCheckpoint,
        WikiDriveCheckpointStore, WikiPersistenceError, WikiPersistenceScope, WikiPublication,
        WikiPublicationStore,
    },
};
use thiserror::Error;

pub const KNOWLEDGE_WIKI_SOURCE_PARENT_PATH: &str = "sources";
pub const KNOWLEDGE_WIKI_SOURCE_ROOT_PATH: &str = "sources/raw";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializeKnowledgeWikiRequest {
    pub scope: WikiPersistenceScope,
    pub space_id: u64,
    pub knowledgebase_uuid: String,
    pub drive_space_uuid: String,
    pub actor_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWikiInitializationResult {
    pub publication: WikiPublication,
    pub source_scope: KnowledgebaseRawScope,
    pub checkpoint: WikiDriveCheckpoint,
}

pub struct KnowledgeWikiInitializationService<'a> {
    publication_store: &'a dyn WikiPublicationStore,
    checkpoint_store: &'a dyn WikiDriveCheckpointStore,
    drive_workspace: &'a dyn KnowledgeDriveWorkspace,
    drive_tree: &'a dyn KnowledgeDriveNodeTree,
    drive_scope: &'a dyn KnowledgeWikiDriveScope,
}

impl<'a> KnowledgeWikiInitializationService<'a> {
    pub fn new(
        publication_store: &'a dyn WikiPublicationStore,
        checkpoint_store: &'a dyn WikiDriveCheckpointStore,
        drive_workspace: &'a dyn KnowledgeDriveWorkspace,
        drive_tree: &'a dyn KnowledgeDriveNodeTree,
        drive_scope: &'a dyn KnowledgeWikiDriveScope,
    ) -> Self {
        Self {
            publication_store,
            checkpoint_store,
            drive_workspace,
            drive_tree,
            drive_scope,
        }
    }

    pub async fn initialize(
        &self,
        request: InitializeKnowledgeWikiRequest,
    ) -> Result<KnowledgeWikiInitializationResult, KnowledgeWikiInitializationError> {
        validate_request(&request)?;
        let publication = self
            .publication_store
            .get_publication_for_space(request.scope, request.space_id)
            .await?
            .ok_or(
                KnowledgeWikiInitializationError::PublicationNotProvisioned {
                    space_id: request.space_id,
                },
            )?;
        if publication.drive_space_uuid != request.drive_space_uuid {
            return Err(KnowledgeWikiInitializationError::IdentityConflict(
                "Wiki publication Drive Space does not match the knowledge space binding"
                    .to_string(),
            ));
        }

        self.drive_workspace
            .ensure_nodes(EnsureKnowledgeDriveNodesRequest {
                drive_space_id: request.drive_space_uuid.clone(),
                nodes: vec![
                    folder_node(KNOWLEDGE_WIKI_SOURCE_PARENT_PATH),
                    folder_node(KNOWLEDGE_WIKI_SOURCE_ROOT_PATH),
                ],
            })
            .await?;
        let raw_folder = self
            .drive_tree
            .resolve_path(ResolveKnowledgeDriveNodePathRequest {
                drive_space_id: request.drive_space_uuid.clone(),
                logical_path: KNOWLEDGE_WIKI_SOURCE_ROOT_PATH.to_string(),
            })
            .await?
            .ok_or(KnowledgeWikiInitializationError::RawSourceRootMissing)?;
        if raw_folder.kind != DriveNodeKind::Folder
            || raw_folder.path != KNOWLEDGE_WIKI_SOURCE_ROOT_PATH
        {
            return Err(KnowledgeWikiInitializationError::IdentityConflict(
                "sources/raw must resolve to the canonical Drive folder".to_string(),
            ));
        }

        let source_scope = self
            .drive_scope
            .ensure_raw_scope(EnsureKnowledgebaseRawScopeRequest {
                drive_space_id: request.drive_space_uuid.clone(),
                knowledgebase_uuid: request.knowledgebase_uuid.clone(),
                raw_folder_node_id: raw_folder.drive_node_id.clone(),
            })
            .await?;
        validate_source_scope(&request, &raw_folder.drive_node_id, &source_scope)?;

        let publication = self
            .publication_store
            .bind_source_scope(BindWikiSourceScopeRequest {
                scope: request.scope,
                site_publication_id: publication.id,
                source_root_node_uuid: raw_folder.drive_node_id,
                source_scope_uuid: source_scope.subscription_uuid.clone(),
                expected_version: publication.version,
                actor_id: request.actor_id,
            })
            .await?;
        let checkpoint = self
            .checkpoint_store
            .provision_checkpoint(ProvisionWikiDriveCheckpointRequest {
                scope: request.scope,
                site_publication_id: publication.id,
                drive_space_uuid: request.drive_space_uuid,
                source_scope_uuid: source_scope.subscription_uuid.clone(),
                actor_id: request.actor_id,
            })
            .await?;

        Ok(KnowledgeWikiInitializationResult {
            publication,
            source_scope,
            checkpoint,
        })
    }
}

fn folder_node(logical_path: &str) -> EnsureKnowledgeDriveNodeRequest {
    EnsureKnowledgeDriveNodeRequest {
        logical_path: logical_path.to_string(),
        kind: EnsureKnowledgeDriveNodeKind::Folder,
        object_ref: None,
    }
}

fn validate_request(
    request: &InitializeKnowledgeWikiRequest,
) -> Result<(), KnowledgeWikiInitializationError> {
    if request.scope.tenant_id == 0
        || request.space_id == 0
        || request.actor_id == 0
        || request.knowledgebase_uuid.trim().is_empty()
        || request.drive_space_uuid.trim().is_empty()
    {
        return Err(KnowledgeWikiInitializationError::InvalidRequest(
            "tenant_id, space_id, actor_id, knowledgebase_uuid, and drive_space_uuid are required"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_source_scope(
    request: &InitializeKnowledgeWikiRequest,
    raw_folder_node_id: &str,
    source_scope: &KnowledgebaseRawScope,
) -> Result<(), KnowledgeWikiInitializationError> {
    if source_scope.drive_space_id != request.drive_space_uuid
        || source_scope.knowledgebase_uuid != request.knowledgebase_uuid
        || source_scope.raw_folder_node_id != raw_folder_node_id
        || source_scope.consumer_kind != KNOWLEDGEBASE_RAW_CONSUMER_KIND
        || source_scope.scope_status != "ACTIVE"
    {
        return Err(KnowledgeWikiInitializationError::IdentityConflict(
            "Drive returned a root scope that does not match the canonical Knowledgebase raw root"
                .to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiInitializationError {
    #[error("Wiki initialization request is invalid: {0}")]
    InvalidRequest(String),
    #[error("Wiki publication is not provisioned for knowledge space {space_id}")]
    PublicationNotProvisioned { space_id: u64 },
    #[error("Wiki initialization identity conflict: {0}")]
    IdentityConflict(String),
    #[error("canonical Wiki source root sources/raw is missing after Drive initialization")]
    RawSourceRootMissing,
    #[error(transparent)]
    Workspace(#[from] KnowledgeDriveWorkspaceError),
    #[error(transparent)]
    DriveTree(#[from] KnowledgeDriveNodeTreeError),
    #[error(transparent)]
    DriveSource(#[from] KnowledgeWikiDriveSourceError),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
}
