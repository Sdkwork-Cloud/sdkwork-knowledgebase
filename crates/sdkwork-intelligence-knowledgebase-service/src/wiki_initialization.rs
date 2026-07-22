use crate::ports::{
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
    drive_scope: &'a dyn KnowledgeWikiDriveScope,
}

impl<'a> KnowledgeWikiInitializationService<'a> {
    pub fn new(
        publication_store: &'a dyn WikiPublicationStore,
        checkpoint_store: &'a dyn WikiDriveCheckpointStore,
        drive_scope: &'a dyn KnowledgeWikiDriveScope,
    ) -> Self {
        Self {
            publication_store,
            checkpoint_store,
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

        let source_scope = self
            .drive_scope
            .ensure_raw_scope(EnsureKnowledgebaseRawScopeRequest {
                drive_space_id: request.drive_space_uuid.clone(),
                knowledgebase_uuid: request.knowledgebase_uuid.clone(),
            })
            .await?;
        validate_source_scope(&request, &source_scope)?;

        let publication = self
            .publication_store
            .bind_source_scope(BindWikiSourceScopeRequest {
                scope: request.scope,
                site_publication_id: publication.id,
                source_root_node_uuid: source_scope.raw_folder_node_id.clone(),
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
    source_scope: &KnowledgebaseRawScope,
) -> Result<(), KnowledgeWikiInitializationError> {
    if source_scope.drive_space_id != request.drive_space_uuid
        || source_scope.knowledgebase_uuid != request.knowledgebase_uuid
        || source_scope.raw_folder_node_id.trim().is_empty()
        || source_scope.consumer_kind != KNOWLEDGEBASE_RAW_CONSUMER_KIND
        || !source_scope.scope_status.eq_ignore_ascii_case("ACTIVE")
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
    #[error(transparent)]
    DriveSource(#[from] KnowledgeWikiDriveSourceError),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
}
