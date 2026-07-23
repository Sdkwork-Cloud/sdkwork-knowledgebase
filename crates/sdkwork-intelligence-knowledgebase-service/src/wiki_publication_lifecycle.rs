use sdkwork_utils_rust::is_blank;
use thiserror::Error;

use crate::ports::{
    knowledge_wiki_persistence::{WikiPersistenceError, WikiVisibility},
    knowledge_wiki_publication_lifecycle::{
        ChangeWikiPageVisibilityRequest, ChangeWikiPublicationStatusRequest,
        PublishWikiPageRequest, UnpublishWikiPageRequest, WikiLifecycleAuditContext,
        WikiPageLifecycleResult, WikiPublicationLifecycleResult, WikiPublicationLifecycleStore,
    },
};

pub struct KnowledgeWikiPublicationLifecycleService<'a> {
    store: &'a dyn WikiPublicationLifecycleStore,
}

impl<'a> KnowledgeWikiPublicationLifecycleService<'a> {
    pub fn new(store: &'a dyn WikiPublicationLifecycleStore) -> Self {
        Self { store }
    }

    pub async fn change_publication_status(
        &self,
        request: ChangeWikiPublicationStatusRequest,
    ) -> Result<WikiPublicationLifecycleResult, KnowledgeWikiPublicationLifecycleError> {
        validate_common(
            request.scope.tenant_id,
            request.space_id,
            request.actor_id,
            &request.audit,
        )?;
        self.store
            .change_publication_status(request)
            .await
            .map_err(Into::into)
    }

    pub async fn publish_page(
        &self,
        request: PublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, KnowledgeWikiPublicationLifecycleError> {
        validate_common(
            request.scope.tenant_id,
            request.space_id,
            request.actor_id,
            &request.audit,
        )?;
        validate_source_file_uuid(&request.source_file_uuid)?;
        if request.visibility == WikiVisibility::Private {
            return Err(KnowledgeWikiPublicationLifecycleError::InvalidRequest(
                "publishing requires PUBLIC or UNLISTED visibility".to_string(),
            ));
        }
        self.store.publish_page(request).await.map_err(Into::into)
    }

    pub async fn unpublish_page(
        &self,
        request: UnpublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, KnowledgeWikiPublicationLifecycleError> {
        validate_common(
            request.scope.tenant_id,
            request.space_id,
            request.actor_id,
            &request.audit,
        )?;
        validate_source_file_uuid(&request.source_file_uuid)?;
        self.store.unpublish_page(request).await.map_err(Into::into)
    }

    pub async fn change_page_visibility(
        &self,
        request: ChangeWikiPageVisibilityRequest,
    ) -> Result<WikiPageLifecycleResult, KnowledgeWikiPublicationLifecycleError> {
        validate_common(
            request.scope.tenant_id,
            request.space_id,
            request.actor_id,
            &request.audit,
        )?;
        validate_source_file_uuid(&request.source_file_uuid)?;
        self.store
            .change_page_visibility(request)
            .await
            .map_err(Into::into)
    }
}

fn validate_common(
    tenant_id: u64,
    space_id: u64,
    actor_id: u64,
    audit: &WikiLifecycleAuditContext,
) -> Result<(), KnowledgeWikiPublicationLifecycleError> {
    if tenant_id == 0 || space_id == 0 || actor_id == 0 {
        return Err(KnowledgeWikiPublicationLifecycleError::InvalidRequest(
            "tenant_id, space_id, and actor_id must be greater than zero".to_string(),
        ));
    }
    let request_id = audit.request_id.trim();
    if request_id.is_empty() || request_id.len() > 128 {
        return Err(KnowledgeWikiPublicationLifecycleError::InvalidRequest(
            "audit request_id must contain between 1 and 128 bytes".to_string(),
        ));
    }
    if audit
        .trace_id
        .as_deref()
        .is_some_and(|trace_id| is_blank(Some(trace_id)) || trace_id.len() > 128)
    {
        return Err(KnowledgeWikiPublicationLifecycleError::InvalidRequest(
            "audit trace_id must contain between 1 and 128 bytes when present".to_string(),
        ));
    }
    Ok(())
}

fn validate_source_file_uuid(
    source_file_uuid: &str,
) -> Result<(), KnowledgeWikiPublicationLifecycleError> {
    let value = source_file_uuid.trim();
    if value.is_empty() || value.len() > 64 {
        return Err(KnowledgeWikiPublicationLifecycleError::InvalidRequest(
            "source_file_uuid must contain between 1 and 64 bytes".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeWikiPublicationLifecycleError {
    #[error("Wiki publication lifecycle invalid request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
}
