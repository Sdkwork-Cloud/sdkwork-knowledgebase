use crate::ports::{
    knowledge_wiki_drive_source::{
        KnowledgeWikiDriveSource, KnowledgeWikiDriveSourceError, ReadKnowledgeWikiSourceRequest,
        ResolveKnowledgeWikiSourceRequest, MAX_WIKI_SOURCE_READ_BYTES,
    },
    knowledge_wiki_persistence::{
        ClaimWikiSourceProcessingRequest, CompleteWikiSourceProcessingRequest,
        ListWikiDriveCheckpointsRequest, RetryWikiSourceProcessingRequest,
        WikiDriveCheckpointStore, WikiIndexState, WikiPersistenceError, WikiPersistenceScope,
        WikiPublication, WikiPublicationMode, WikiPublicationStore, WikiSourceFileKind,
        WikiSourceProjection, WikiSourceProjectionStore, WikiSourceState, WikiVisibility,
    },
    knowledge_wiki_publication_lifecycle::{
        PublishWikiPageRequest, WikiLifecycleAuditContext, WikiPublicationLifecycleStore,
    },
};
use crate::{
    wiki_publication_lifecycle::KnowledgeWikiPublicationLifecycleService,
    wiki_representation::{canonical_route_for_source, render_wiki_page, WikiRepresentationError},
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use thiserror::Error;

pub const MAX_WIKI_SOURCE_PROCESSING_PAGE_SIZE: u32 = 100;
pub const MAX_WIKI_CHECKPOINT_PROCESSING_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessKnowledgeWikiSourceCheckpointPageRequest {
    pub scope: WikiPersistenceScope,
    pub after_checkpoint_id: Option<u64>,
    pub worker_id: String,
    pub actor_id: u64,
    pub lease_seconds: u64,
    pub checkpoint_limit: u32,
    pub source_limit_per_checkpoint: u32,
    pub retry_delay_seconds: u64,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KnowledgeWikiSourceCheckpointPageResult {
    pub checkpoints_processed: usize,
    pub sources_claimed: usize,
    pub sources_ready: usize,
    pub sources_auto_published: usize,
    pub sources_retried: usize,
    pub sources_quarantined: usize,
    pub auto_publications_deferred: usize,
    pub next_after_checkpoint_id: Option<u64>,
}

pub struct KnowledgeWikiSourceProcessorService<'a> {
    publication_store: &'a dyn WikiPublicationStore,
    checkpoint_store: &'a dyn WikiDriveCheckpointStore,
    projection_store: &'a dyn WikiSourceProjectionStore,
    lifecycle_store: &'a dyn WikiPublicationLifecycleStore,
    drive_source: &'a dyn KnowledgeWikiDriveSource,
}

impl<'a> KnowledgeWikiSourceProcessorService<'a> {
    pub fn new(
        publication_store: &'a dyn WikiPublicationStore,
        checkpoint_store: &'a dyn WikiDriveCheckpointStore,
        projection_store: &'a dyn WikiSourceProjectionStore,
        lifecycle_store: &'a dyn WikiPublicationLifecycleStore,
        drive_source: &'a dyn KnowledgeWikiDriveSource,
    ) -> Self {
        Self {
            publication_store,
            checkpoint_store,
            projection_store,
            lifecycle_store,
            drive_source,
        }
    }

    pub async fn process_checkpoint_page(
        &self,
        request: ProcessKnowledgeWikiSourceCheckpointPageRequest,
    ) -> Result<KnowledgeWikiSourceCheckpointPageResult, KnowledgeWikiSourceProcessorError> {
        validate_request(&request)?;
        let page = self
            .checkpoint_store
            .list_checkpoints(ListWikiDriveCheckpointsRequest {
                scope: request.scope,
                after_checkpoint_id: request.after_checkpoint_id,
                limit: request.checkpoint_limit,
            })
            .await?;
        let mut result = KnowledgeWikiSourceCheckpointPageResult {
            next_after_checkpoint_id: page.next_after_checkpoint_id,
            ..KnowledgeWikiSourceCheckpointPageResult::default()
        };
        for checkpoint in page.checkpoints {
            let publication = self
                .publication_store
                .get_publication(request.scope, checkpoint.site_publication_id)
                .await?;
            validate_checkpoint_identity(&publication, &checkpoint.source_scope_uuid)?;
            let claimed = self
                .projection_store
                .claim_source_processing(ClaimWikiSourceProcessingRequest {
                    scope: request.scope,
                    site_publication_id: publication.id,
                    claim_owner: request.worker_id.clone(),
                    lease_seconds: request.lease_seconds,
                    after_id: None,
                    limit: request.source_limit_per_checkpoint,
                })
                .await?;
            result.checkpoints_processed += 1;
            result.sources_claimed += claimed.len();
            for projection in claimed {
                match self
                    .validate_and_complete(&publication, &projection, request.actor_id)
                    .await
                {
                    Ok(ready) => {
                        result.sources_ready += 1;
                        match self
                            .auto_publish(&publication, &ready, request.actor_id)
                            .await
                        {
                            Ok(true) => result.sources_auto_published += 1,
                            Ok(false) => {}
                            Err(error) => {
                                result.auto_publications_deferred += 1;
                                tracing::warn!(
                                    target: "sdkwork.knowledgebase.wiki",
                                    event = "knowledgebase.wiki.auto_publication_deferred",
                                    publication_id = publication.id,
                                    projection_id = ready.id,
                                    error_code = %error.code(),
                                    retry_scheduled = true,
                                    "Wiki source is READY but automatic publication was deferred"
                                );
                            }
                        }
                    }
                    Err(error) => {
                        let lease_token = projection.processing_lease_token.clone().ok_or(
                            KnowledgeWikiSourceProcessorError::Integrity(
                                "claimed source projection has no processing lease token"
                                    .to_string(),
                            ),
                        )?;
                        let retried = self
                            .projection_store
                            .retry_source_processing(RetryWikiSourceProcessingRequest {
                                scope: request.scope,
                                projection_id: projection.id,
                                lease_token,
                                processing_fence: projection.processing_fence,
                                error_code: error.code().to_string(),
                                error_summary: "Wiki source processing failed".to_string(),
                                retry_delay_seconds: request.retry_delay_seconds,
                                max_attempts: request.max_attempts,
                                actor_id: request.actor_id,
                            })
                            .await?;
                        if retried.source_state == WikiSourceState::Quarantined {
                            result.sources_quarantined += 1;
                        } else {
                            result.sources_retried += 1;
                        }
                        tracing::warn!(
                            target: "sdkwork.knowledgebase.wiki",
                            event = "knowledgebase.wiki.source_processing_failed",
                            publication_id = publication.id,
                            projection_id = projection.id,
                            error_code = %error.code(),
                            quarantined = retried.source_state == WikiSourceState::Quarantined,
                            "Wiki source processing failed without logging source content"
                        );
                    }
                }
            }
        }
        Ok(result)
    }

    async fn validate_and_complete(
        &self,
        publication: &WikiPublication,
        projection: &WikiSourceProjection,
        actor_id: u64,
    ) -> Result<WikiSourceProjection, KnowledgeWikiSourceProcessorError> {
        let source_scope_uuid = publication.source_scope_uuid.as_deref().ok_or_else(|| {
            KnowledgeWikiSourceProcessorError::Integrity(
                "Wiki publication has no bound Drive source scope".to_string(),
            )
        })?;
        if projection.size_bytes > MAX_WIKI_SOURCE_READ_BYTES {
            return Err(KnowledgeWikiSourceProcessorError::Unsupported(
                "Wiki source exceeds the bounded public content limit".to_string(),
            ));
        }
        validate_passive_source(projection)?;
        let resource = self
            .drive_source
            .resolve_source(ResolveKnowledgeWikiSourceRequest {
                subscription_uuid: source_scope_uuid.to_string(),
                relative_path: projection.source_path.clone(),
                pinned_generation: None,
                pinned_node_version_id: Some(projection.drive_version_uuid.clone()),
            })
            .await?;
        if resource.subscription_uuid != source_scope_uuid
            || resource.normalized_relative_path != projection.source_path
            || resource.drive_node_id != projection.drive_node_uuid
            || resource.drive_node_version_id != projection.drive_version_uuid
            || resource.content_length != projection.size_bytes
            || resource.checksum_sha256_hex != projection.content_sha256
            || resource.content_type != projection.media_type
            || resource.scope_status != "ACTIVE"
            || resource.node_status != "ACTIVE"
            || resource.eligibility != "ELIGIBLE"
        {
            return Err(KnowledgeWikiSourceProcessorError::Integrity(
                "resolved Drive source does not match the claimed Wiki projection".to_string(),
            ));
        }
        if projection.file_kind == WikiSourceFileKind::Page {
            let bytes = self
                .drive_source
                .read_pinned_source(ReadKnowledgeWikiSourceRequest {
                    resource,
                    maximum_bytes: MAX_WIKI_SOURCE_READ_BYTES,
                })
                .await?;
            if bytes.len() as u64 != projection.size_bytes
                || format!("sha256:{}", sha256_hash(&bytes)) != projection.content_sha256
            {
                return Err(KnowledgeWikiSourceProcessorError::Integrity(
                    "Drive source bytes do not match the claimed Wiki projection".to_string(),
                ));
            }
            render_wiki_page(&projection.source_path, projection.file_kind, &bytes)?.ok_or_else(
                || {
                    KnowledgeWikiSourceProcessorError::Unsupported(
                        "Wiki page renderer did not produce a safe representation".to_string(),
                    )
                },
            )?;
        }
        let canonical_route =
            canonical_route_for_source(&projection.source_path, projection.file_kind)?;
        let lease_token = projection.processing_lease_token.clone().ok_or_else(|| {
            KnowledgeWikiSourceProcessorError::Integrity(
                "claimed source projection has no processing lease token".to_string(),
            )
        })?;
        self.projection_store
            .complete_source_processing(CompleteWikiSourceProcessingRequest {
                scope: projection.scope,
                site_publication_id: publication.id,
                projection_id: projection.id,
                lease_token,
                processing_fence: projection.processing_fence,
                canonical_route,
                index_state: if projection.file_kind == WikiSourceFileKind::Page {
                    WikiIndexState::Ready
                } else {
                    WikiIndexState::NotRequired
                },
                actor_id,
            })
            .await
            .map_err(Into::into)
    }

    async fn auto_publish(
        &self,
        publication: &WikiPublication,
        projection: &WikiSourceProjection,
        actor_id: u64,
    ) -> Result<bool, KnowledgeWikiSourceProcessorError> {
        let current = self
            .publication_store
            .get_publication(projection.scope, publication.id)
            .await?;
        if current.publication_mode != WikiPublicationMode::AutoPublicAfterChecks
            || current.default_visibility == WikiVisibility::Private
        {
            return Ok(false);
        }
        let result = KnowledgeWikiPublicationLifecycleService::new(self.lifecycle_store)
            .publish_page(PublishWikiPageRequest {
                scope: projection.scope,
                space_id: current.space_id,
                source_file_uuid: projection.uuid.clone(),
                visibility: current.default_visibility,
                expected_publication_version: current.version,
                expected_page_version: projection.version,
                actor_id,
                audit: WikiLifecycleAuditContext {
                    request_id: format!(
                        "wiki-source-processor:{}:{}",
                        projection.id, projection.processing_fence
                    ),
                    trace_id: None,
                },
            })
            .await
            .map_err(|error| KnowledgeWikiSourceProcessorError::Publication(error.to_string()))?;
        Ok(result.page.public_drive_version_uuid.as_deref()
            == Some(projection.drive_version_uuid.as_str()))
    }
}

fn validate_checkpoint_identity(
    publication: &WikiPublication,
    checkpoint_source_scope_uuid: &str,
) -> Result<(), KnowledgeWikiSourceProcessorError> {
    if publication.source_scope_uuid.as_deref() != Some(checkpoint_source_scope_uuid) {
        return Err(KnowledgeWikiSourceProcessorError::Integrity(
            "Wiki checkpoint does not match the publication source scope".to_string(),
        ));
    }
    Ok(())
}

fn validate_passive_source(
    projection: &WikiSourceProjection,
) -> Result<(), KnowledgeWikiSourceProcessorError> {
    let media_type = projection.media_type.to_ascii_lowercase();
    let extension = projection
        .source_path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    if matches!(
        media_type.split(';').next().unwrap_or_default().trim(),
        "application/javascript"
            | "application/ecmascript"
            | "text/javascript"
            | "text/ecmascript"
            | "image/svg+xml"
            | "application/wasm"
    ) || matches!(extension.as_str(), "js" | "mjs" | "cjs" | "svg" | "wasm")
    {
        return Err(KnowledgeWikiSourceProcessorError::Unsupported(
            "active content is not eligible for publication on a Wiki origin".to_string(),
        ));
    }
    Ok(())
}

fn validate_request(
    request: &ProcessKnowledgeWikiSourceCheckpointPageRequest,
) -> Result<(), KnowledgeWikiSourceProcessorError> {
    if request.scope.tenant_id == 0
        || request.actor_id == 0
        || is_blank(Some(&request.worker_id))
        || request.worker_id.len() > 128
        || request.lease_seconds == 0
        || request.lease_seconds > 3_600
        || request.checkpoint_limit == 0
        || request.checkpoint_limit > MAX_WIKI_CHECKPOINT_PROCESSING_PAGE_SIZE
        || request.source_limit_per_checkpoint == 0
        || request.source_limit_per_checkpoint > MAX_WIKI_SOURCE_PROCESSING_PAGE_SIZE
        || request.retry_delay_seconds == 0
        || request.retry_delay_seconds > 86_400
        || request.max_attempts == 0
        || request.max_attempts > 100
    {
        return Err(KnowledgeWikiSourceProcessorError::InvalidRequest(
            "Wiki source processing request is outside its bounded contract".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiSourceProcessorError {
    #[error("Wiki source processor request is invalid: {0}")]
    InvalidRequest(String),
    #[error("Wiki source processor integrity validation failed: {0}")]
    Integrity(String),
    #[error("Wiki source processor does not support this content: {0}")]
    Unsupported(String),
    #[error("Wiki automatic publication failed: {0}")]
    Publication(String),
    #[error(transparent)]
    Drive(#[from] KnowledgeWikiDriveSourceError),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
    #[error(transparent)]
    Representation(#[from] WikiRepresentationError),
}

impl KnowledgeWikiSourceProcessorError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "wiki_source_processing_request_invalid",
            Self::Integrity(_) => "wiki_source_processing_integrity_failed",
            Self::Unsupported(_) => "wiki_source_processing_unsupported",
            Self::Publication(_) => "wiki_source_auto_publication_failed",
            Self::Drive(error) => error.code(),
            Self::Persistence(_) => "wiki_source_processing_persistence_failed",
            Self::Representation(_) => "wiki_source_rendering_failed",
        }
    }
}
