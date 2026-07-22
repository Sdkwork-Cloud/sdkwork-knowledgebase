use sdkwork_drive_contract::drive::events::{
    derive_webhook_signing_key, verify_webhook_signature, DriveEventEnvelope,
    DriveNodeDeletedV1Data, DriveNodeEligibility, DriveNodeEligibilityChangedV1Data,
    DriveNodePathChangedV1Data, DriveNodeVersionCommittedV1Data, DriveRootScopeEffect,
    DriveRootScopeKind, EVENT_SOURCE, EVENT_SPEC_VERSION,
};
use sdkwork_utils_rust::{hmac_sha256, sha256_hash};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use thiserror::Error;

use crate::ports::{
    knowledge_wiki_drive_source::{
        KnowledgeWikiDriveSource, KnowledgeWikiDriveSourceError, KnowledgeWikiSourceResource,
        ResolveKnowledgeWikiSourceRequest,
    },
    knowledge_wiki_persistence::{
        ApplyWikiDriveEventRequest, ClaimWikiDriveEventsRequest, CompleteWikiDriveEventRequest,
        ListWikiDriveCheckpointsRequest, ReceiveWikiDriveEventRequest, RetryWikiDriveEventRequest,
        WikiDriveCheckpointStore, WikiDriveEventApplicationResult, WikiDriveEventInboxStore,
        WikiDriveEventProcessingState, WikiDriveEventReceipt, WikiDriveEventType,
        WikiDriveProjectionMutation, WikiDriveSourceMetadata, WikiPagePublicationState,
        WikiPersistenceError, WikiPersistenceScope, WikiPublication, WikiPublicationStore,
        WikiSourceFileKind, WikiSourceState,
    },
};

const MAX_EVENT_BATCH_SIZE: u32 = 100;
const MAX_CHECKPOINT_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveKnowledgeWikiDriveWebhookRequest {
    pub channel_id: String,
    pub event_id: String,
    pub timestamp: String,
    pub signature: String,
    pub retry_count: String,
    pub idempotency_key: String,
    pub payload_json: String,
}

/// Trusted process-local ingestion request used by standalone Drive relays.
///
/// Standalone does not need an HTTP signature hop, but it still supplies the
/// root-scope identity so cloud and embedded delivery share authority checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveKnowledgeWikiDriveTrustedEventRequest {
    pub scope: WikiPersistenceScope,
    pub source_scope_uuid: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWikiDriveTrustedEventTargets {
    pub scope: WikiPersistenceScope,
    pub drive_space_uuid: String,
    pub source_scope_uuids: Vec<String>,
}

/// Resolves the Knowledgebase raw scopes explicitly addressed by a Drive event.
///
/// Embedded relays use this before calling `receive_trusted` once per scope. The returned set is
/// derived only from the event's authoritative root-scope effects; it never walks Drive paths or
/// infers membership from a Space identifier.
pub fn resolve_knowledge_wiki_drive_trusted_event_targets(
    payload_json: &str,
) -> Result<KnowledgeWikiDriveTrustedEventTargets, KnowledgeWikiDriveEventConsumerError> {
    let parsed = parse_drive_event(payload_json)?;
    let tenant_id = parse_positive_i64_string("tenantId", parsed.tenant_id())?;
    let organization_id = parsed
        .organization_id()
        .map(|value| parse_nonnegative_i64_string("organizationId", value))
        .transpose()?
        .unwrap_or(0);
    let mut unique = HashSet::new();
    let mut source_scope_uuids = Vec::new();
    for effect in parsed.root_scope_effects() {
        if effect.scope_kind != DriveRootScopeKind::KnowledgebaseRaw {
            continue;
        }
        uuid::Uuid::parse_str(&effect.scope_id).map_err(|_| {
            KnowledgeWikiDriveEventConsumerError::InvalidEvent(
                "Knowledgebase raw root scope id must be a UUID".to_string(),
            )
        })?;
        validate_relative_path(&effect.relative_path)?;
        if unique.insert(effect.scope_id.clone()) {
            source_scope_uuids.push(effect.scope_id.clone());
        }
    }
    Ok(KnowledgeWikiDriveTrustedEventTargets {
        scope: WikiPersistenceScope {
            tenant_id,
            organization_id,
        },
        drive_space_uuid: parsed.space_id().to_string(),
        source_scope_uuids,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessKnowledgeWikiDriveEventsRequest {
    pub scope: WikiPersistenceScope,
    pub checkpoint_id: u64,
    pub worker_id: String,
    pub actor_id: u64,
    pub lease_seconds: u64,
    pub limit: u32,
    pub retry_delay_seconds: u64,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessKnowledgeWikiDriveCheckpointPageRequest {
    pub scope: WikiPersistenceScope,
    pub after_checkpoint_id: Option<u64>,
    pub worker_id: String,
    pub actor_id: u64,
    pub lease_seconds: u64,
    pub checkpoint_limit: u32,
    pub event_limit_per_checkpoint: u32,
    pub retry_delay_seconds: u64,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KnowledgeWikiDriveEventBatchResult {
    pub applied: usize,
    pub retried: usize,
    pub dead_lettered: usize,
    pub public_changes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KnowledgeWikiDriveCheckpointPageResult {
    pub checkpoints_processed: usize,
    pub events: KnowledgeWikiDriveEventBatchResult,
    pub next_after_checkpoint_id: Option<u64>,
}

pub struct KnowledgeWikiDriveEventConsumerService<'a> {
    publication_store: &'a dyn WikiPublicationStore,
    checkpoint_store: &'a dyn WikiDriveCheckpointStore,
    inbox_store: &'a dyn WikiDriveEventInboxStore,
    drive_source: &'a dyn KnowledgeWikiDriveSource,
    webhook_signing_secrets: Vec<Vec<u8>>,
}

impl<'a> KnowledgeWikiDriveEventConsumerService<'a> {
    pub fn new(
        publication_store: &'a dyn WikiPublicationStore,
        checkpoint_store: &'a dyn WikiDriveCheckpointStore,
        inbox_store: &'a dyn WikiDriveEventInboxStore,
        drive_source: &'a dyn KnowledgeWikiDriveSource,
    ) -> Self {
        Self {
            publication_store,
            checkpoint_store,
            inbox_store,
            drive_source,
            webhook_signing_secrets: Vec::new(),
        }
    }

    pub fn with_webhook_signing_secrets(mut self, secrets: Vec<String>) -> Self {
        self.webhook_signing_secrets = secrets
            .into_iter()
            .filter(|secret| !secret.is_empty())
            .map(|secret| secret.into_bytes())
            .collect();
        self
    }

    pub async fn receive_webhook(
        &self,
        request: ReceiveKnowledgeWikiDriveWebhookRequest,
    ) -> Result<WikiDriveEventReceipt, KnowledgeWikiDriveEventConsumerError> {
        validate_webhook_request(&request)?;
        let subscription_uuid = request.channel_id.strip_prefix("kbraw:").ok_or_else(|| {
            KnowledgeWikiDriveEventConsumerError::InvalidRequest(
                "x-sdkwork-drive-channel-id is not a Knowledgebase root scope channel".to_string(),
            )
        })?;
        uuid::Uuid::parse_str(subscription_uuid).map_err(|_| {
            KnowledgeWikiDriveEventConsumerError::InvalidRequest(
                "x-sdkwork-drive-channel-id does not contain a valid root scope UUID".to_string(),
            )
        })?;
        let signature_valid = self
            .webhook_signing_secrets
            .iter()
            .map(|secret| hmac_sha256(subscription_uuid.as_bytes(), secret))
            .map(|token| derive_webhook_signing_key(&token))
            .any(|signing_key| {
                verify_webhook_signature(
                    &request.timestamp,
                    request.payload_json.as_bytes(),
                    signing_key.as_bytes(),
                    &request.signature,
                )
            });
        if !signature_valid {
            return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
                "Drive webhook signature is invalid".to_string(),
            ));
        }
        let parsed = parse_drive_event(&request.payload_json)?;
        if parsed.id() != request.event_id {
            return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
                "Drive webhook event id does not match the signed payload".to_string(),
            ));
        }
        let tenant_id = parse_positive_i64_string("tenantId", parsed.tenant_id())?;
        let organization_id = parsed
            .organization_id()
            .map(|value| parse_nonnegative_i64_string("organizationId", value))
            .transpose()?
            .unwrap_or(0);
        self.receive_trusted(ReceiveKnowledgeWikiDriveTrustedEventRequest {
            scope: WikiPersistenceScope {
                tenant_id,
                organization_id,
            },
            source_scope_uuid: subscription_uuid.to_string(),
            payload_json: request.payload_json,
        })
        .await
    }

    pub async fn receive_trusted(
        &self,
        request: ReceiveKnowledgeWikiDriveTrustedEventRequest,
    ) -> Result<WikiDriveEventReceipt, KnowledgeWikiDriveEventConsumerError> {
        validate_trusted_receive_request(&request)?;
        let parsed = parse_drive_event(&request.payload_json)?;
        let checkpoint = self
            .checkpoint_store
            .find_checkpoint_by_drive_scope(
                request.scope,
                parsed.space_id(),
                &request.source_scope_uuid,
            )
            .await?
            .ok_or_else(|| {
                KnowledgeWikiDriveEventConsumerError::Integrity(
                    "Drive event root scope has no active Knowledgebase checkpoint".to_string(),
                )
            })?;
        let publication = self
            .publication_store
            .get_publication(request.scope, checkpoint.site_publication_id)
            .await?;
        validate_event_authority(
            request.scope,
            &publication,
            checkpoint.site_publication_id,
            &checkpoint.drive_space_uuid,
            &checkpoint.source_scope_uuid,
            &parsed,
        )?;

        self.inbox_store
            .receive_event(ReceiveWikiDriveEventRequest {
                scope: request.scope,
                site_publication_id: checkpoint.site_publication_id,
                checkpoint_id: checkpoint.id,
                source_event_id: parsed.id().to_string(),
                event_type: parsed.event_type(),
                sequence_no: parsed.sequence_no()?,
                drive_node_uuid: parsed.node_id().to_string(),
                drive_version_uuid: parsed.drive_version_id().map(str::to_string),
                payload_sha256: format!("sha256:{}", sha256_hash(request.payload_json.as_bytes())),
                payload_json: request.payload_json,
                source_event_time: parsed.time().to_string(),
            })
            .await
            .map_err(Into::into)
    }

    pub async fn process_batch(
        &self,
        request: ProcessKnowledgeWikiDriveEventsRequest,
    ) -> Result<KnowledgeWikiDriveEventBatchResult, KnowledgeWikiDriveEventConsumerError> {
        validate_process_request(&request)?;
        let mut result = KnowledgeWikiDriveEventBatchResult::default();
        while result.applied + result.retried + result.dead_lettered < request.limit as usize {
            let mut claimed = self
                .inbox_store
                .claim_events(ClaimWikiDriveEventsRequest {
                    scope: request.scope,
                    checkpoint_id: request.checkpoint_id,
                    claim_owner: request.worker_id.clone(),
                    lease_seconds: request.lease_seconds,
                    after_id: None,
                    limit: 1,
                })
                .await?;
            let Some(event) = claimed.pop() else {
                break;
            };
            let lease_token = event.lease_token.clone().ok_or_else(|| {
                KnowledgeWikiDriveEventConsumerError::Integrity(
                    "claimed Drive event has no lease token".to_string(),
                )
            })?;
            match self
                .apply_claimed_event(request.scope, request.actor_id, &event, &lease_token)
                .await
            {
                Ok(applied) => {
                    result.applied += 1;
                    if applied.public_route_change.is_some() {
                        result.public_changes += 1;
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        target: "sdkwork.knowledgebase.wiki",
                        event_id = event.id,
                        source_event_id = %event.source_event_id,
                        error = %error,
                        "Wiki Drive event application will be retried without advancing its checkpoint"
                    );
                    let retried = self
                        .inbox_store
                        .retry_event(RetryWikiDriveEventRequest {
                            scope: request.scope,
                            event_id: event.id,
                            lease_token,
                            error_code: error.code().to_string(),
                            error_summary: "Wiki Drive event application failed".to_string(),
                            retry_delay_seconds: request.retry_delay_seconds,
                            max_attempts: request.max_attempts,
                        })
                        .await?;
                    if retried.processing_state == WikiDriveEventProcessingState::DeadLetter {
                        result.dead_lettered += 1;
                    } else {
                        result.retried += 1;
                    }
                    break;
                }
            }
        }
        Ok(result)
    }

    pub async fn process_checkpoint_page(
        &self,
        request: ProcessKnowledgeWikiDriveCheckpointPageRequest,
    ) -> Result<KnowledgeWikiDriveCheckpointPageResult, KnowledgeWikiDriveEventConsumerError> {
        validate_checkpoint_page_request(&request)?;
        let page = self
            .checkpoint_store
            .list_checkpoints(ListWikiDriveCheckpointsRequest {
                scope: request.scope,
                after_checkpoint_id: request.after_checkpoint_id,
                limit: request.checkpoint_limit,
            })
            .await?;
        let mut result = KnowledgeWikiDriveCheckpointPageResult {
            next_after_checkpoint_id: page.next_after_checkpoint_id,
            ..KnowledgeWikiDriveCheckpointPageResult::default()
        };
        for checkpoint in page.checkpoints {
            let batch = self
                .process_batch(ProcessKnowledgeWikiDriveEventsRequest {
                    scope: request.scope,
                    checkpoint_id: checkpoint.id,
                    worker_id: request.worker_id.clone(),
                    actor_id: request.actor_id,
                    lease_seconds: request.lease_seconds,
                    limit: request.event_limit_per_checkpoint,
                    retry_delay_seconds: request.retry_delay_seconds,
                    max_attempts: request.max_attempts,
                })
                .await?;
            result.checkpoints_processed += 1;
            result.events.applied += batch.applied;
            result.events.retried += batch.retried;
            result.events.dead_lettered += batch.dead_lettered;
            result.events.public_changes += batch.public_changes;
        }
        Ok(result)
    }

    async fn apply_claimed_event(
        &self,
        scope: WikiPersistenceScope,
        actor_id: u64,
        event: &crate::ports::knowledge_wiki_persistence::WikiDriveInboxEvent,
        lease_token: &str,
    ) -> Result<WikiDriveEventApplicationResult, KnowledgeWikiDriveEventConsumerError> {
        let parsed = parse_drive_event(&event.payload_json)?;
        let publication = self
            .publication_store
            .get_publication(scope, event.site_publication_id)
            .await?;
        let checkpoint = self
            .checkpoint_store
            .get_checkpoint(scope, event.checkpoint_id)
            .await?;
        validate_event_authority(
            scope,
            &publication,
            checkpoint.site_publication_id,
            &checkpoint.drive_space_uuid,
            &checkpoint.source_scope_uuid,
            &parsed,
        )?;
        validate_claimed_event_integrity(scope, event, &parsed)?;
        let mutation = self
            .build_mutation(&publication, &checkpoint.source_scope_uuid, &parsed)
            .await?;
        self.inbox_store
            .apply_event(ApplyWikiDriveEventRequest {
                complete: CompleteWikiDriveEventRequest {
                    scope,
                    event_id: event.id,
                    lease_token: lease_token.to_string(),
                    actor_id,
                },
                mutation,
            })
            .await
            .map_err(Into::into)
    }

    async fn build_mutation(
        &self,
        publication: &WikiPublication,
        source_scope_uuid: &str,
        event: &ParsedDriveEvent,
    ) -> Result<WikiDriveProjectionMutation, KnowledgeWikiDriveEventConsumerError> {
        match event {
            ParsedDriveEvent::Version(envelope) => {
                let Some(path) = scoped_path(&envelope.data.root_scopes, source_scope_uuid)? else {
                    return Ok(WikiDriveProjectionMutation::None);
                };
                validate_sha256_checksum(&envelope.data.checksum_sha256_hex)?;
                Ok(WikiDriveProjectionMutation::Upsert(
                    WikiDriveSourceMetadata {
                        drive_version_uuid: envelope.data.drive_version_id.clone(),
                        source_path: path.clone(),
                        file_kind: classify_source_kind(&path, &envelope.data.content_type),
                        media_type: envelope.data.content_type.clone(),
                        size_bytes: parse_nonnegative_i64_string(
                            "contentLength",
                            &envelope.data.content_length,
                        )?,
                        content_sha256: envelope.data.checksum_sha256_hex.clone(),
                    },
                ))
            }
            ParsedDriveEvent::Path(envelope) => {
                let old_path = scoped_path(&envelope.data.old_root_scopes, source_scope_uuid)?;
                let new_path = scoped_path(&envelope.data.new_root_scopes, source_scope_uuid)?;
                match (old_path, new_path) {
                    (None, None) => Ok(WikiDriveProjectionMutation::None),
                    (Some(_), None) => Ok(revocation(
                        WikiSourceState::Deleted,
                        WikiPagePublicationState::Archived,
                        "moved_outside_source_root",
                    )),
                    (Some(_), Some(new_path)) => Ok(WikiDriveProjectionMutation::MoveWithin {
                        source_path: new_path,
                    }),
                    (None, Some(new_path)) => {
                        self.resolve_upsert(publication, source_scope_uuid, &new_path, None)
                            .await
                    }
                }
            }
            ParsedDriveEvent::Eligibility(envelope) => {
                let Some(path) = scoped_path(&envelope.data.root_scopes, source_scope_uuid)? else {
                    return Ok(WikiDriveProjectionMutation::None);
                };
                match envelope.data.new_eligibility {
                    DriveNodeEligibility::Eligible => {
                        self.resolve_upsert(
                            publication,
                            source_scope_uuid,
                            &path,
                            envelope.data.drive_version_id.as_deref(),
                        )
                        .await
                    }
                    DriveNodeEligibility::Ineligible => {
                        let quarantined = is_quarantine_reason(&envelope.data.reason);
                        Ok(revocation(
                            if quarantined {
                                WikiSourceState::Quarantined
                            } else {
                                WikiSourceState::Error
                            },
                            WikiPagePublicationState::Unpublished,
                            if quarantined {
                                "drive_quarantined"
                            } else {
                                "drive_ineligible"
                            },
                        ))
                    }
                }
            }
            ParsedDriveEvent::Deleted(envelope) => {
                if scoped_path(&envelope.data.root_scopes, source_scope_uuid)?.is_none() {
                    return Ok(WikiDriveProjectionMutation::None);
                }
                Ok(revocation(
                    WikiSourceState::Deleted,
                    WikiPagePublicationState::Archived,
                    "drive_deleted",
                ))
            }
        }
    }

    async fn resolve_upsert(
        &self,
        publication: &WikiPublication,
        source_scope_uuid: &str,
        relative_path: &str,
        pinned_node_version_id: Option<&str>,
    ) -> Result<WikiDriveProjectionMutation, KnowledgeWikiDriveEventConsumerError> {
        let resource = self
            .drive_source
            .resolve_source(ResolveKnowledgeWikiSourceRequest {
                subscription_uuid: source_scope_uuid.to_string(),
                relative_path: relative_path.to_string(),
                pinned_generation: None,
                pinned_node_version_id: pinned_node_version_id.map(str::to_string),
            })
            .await?;
        validate_resolved_source(publication, source_scope_uuid, relative_path, &resource)?;
        Ok(WikiDriveProjectionMutation::Upsert(
            WikiDriveSourceMetadata {
                drive_version_uuid: resource.drive_node_version_id,
                source_path: resource.normalized_relative_path.clone(),
                file_kind: classify_source_kind(
                    &resource.normalized_relative_path,
                    &resource.content_type,
                ),
                media_type: resource.content_type,
                size_bytes: resource.content_length,
                content_sha256: resource.checksum_sha256_hex,
            },
        ))
    }
}

enum ParsedDriveEvent {
    Version(DriveEventEnvelope<DriveNodeVersionCommittedV1Data>),
    Path(DriveEventEnvelope<DriveNodePathChangedV1Data>),
    Eligibility(DriveEventEnvelope<DriveNodeEligibilityChangedV1Data>),
    Deleted(DriveEventEnvelope<DriveNodeDeletedV1Data>),
}

impl ParsedDriveEvent {
    fn id(&self) -> &str {
        match self {
            Self::Version(value) => &value.id,
            Self::Path(value) => &value.id,
            Self::Eligibility(value) => &value.id,
            Self::Deleted(value) => &value.id,
        }
    }

    fn event_type(&self) -> WikiDriveEventType {
        match self {
            Self::Version(_) => WikiDriveEventType::VersionCommitted,
            Self::Path(_) => WikiDriveEventType::PathChanged,
            Self::Eligibility(_) => WikiDriveEventType::EligibilityChanged,
            Self::Deleted(_) => WikiDriveEventType::Deleted,
        }
    }

    fn declared_type(&self) -> &str {
        match self {
            Self::Version(value) => &value.event_type,
            Self::Path(value) => &value.event_type,
            Self::Eligibility(value) => &value.event_type,
            Self::Deleted(value) => &value.event_type,
        }
    }

    fn source(&self) -> &str {
        match self {
            Self::Version(value) => &value.source,
            Self::Path(value) => &value.source,
            Self::Eligibility(value) => &value.source,
            Self::Deleted(value) => &value.source,
        }
    }

    fn specversion(&self) -> &str {
        match self {
            Self::Version(value) => &value.specversion,
            Self::Path(value) => &value.specversion,
            Self::Eligibility(value) => &value.specversion,
            Self::Deleted(value) => &value.specversion,
        }
    }

    fn time(&self) -> &str {
        match self {
            Self::Version(value) => &value.time,
            Self::Path(value) => &value.time,
            Self::Eligibility(value) => &value.time,
            Self::Deleted(value) => &value.time,
        }
    }

    fn tenant_id(&self) -> &str {
        match self {
            Self::Version(value) => &value.tenant_id,
            Self::Path(value) => &value.tenant_id,
            Self::Eligibility(value) => &value.tenant_id,
            Self::Deleted(value) => &value.tenant_id,
        }
    }

    fn organization_id(&self) -> Option<&str> {
        match self {
            Self::Version(value) => value.organization_id.as_deref(),
            Self::Path(value) => value.organization_id.as_deref(),
            Self::Eligibility(value) => value.organization_id.as_deref(),
            Self::Deleted(value) => value.organization_id.as_deref(),
        }
    }

    fn sequence_no(&self) -> Result<u64, KnowledgeWikiDriveEventConsumerError> {
        let value = match self {
            Self::Version(value) => &value.sequence_no,
            Self::Path(value) => &value.sequence_no,
            Self::Eligibility(value) => &value.sequence_no,
            Self::Deleted(value) => &value.sequence_no,
        };
        parse_positive_i64_string("sequenceNo", value)
    }

    fn space_id(&self) -> &str {
        match self {
            Self::Version(value) => &value.data.space_id,
            Self::Path(value) => &value.data.space_id,
            Self::Eligibility(value) => &value.data.space_id,
            Self::Deleted(value) => &value.data.space_id,
        }
    }

    fn node_id(&self) -> &str {
        match self {
            Self::Version(value) => &value.data.node_id,
            Self::Path(value) => &value.data.node_id,
            Self::Eligibility(value) => &value.data.node_id,
            Self::Deleted(value) => &value.data.node_id,
        }
    }

    fn drive_version_id(&self) -> Option<&str> {
        match self {
            Self::Version(value) => Some(&value.data.drive_version_id),
            Self::Path(_) => None,
            Self::Eligibility(value) => value.data.drive_version_id.as_deref(),
            Self::Deleted(value) => value.data.drive_version_id.as_deref(),
        }
    }

    fn root_scope_effects(&self) -> Vec<&DriveRootScopeEffect> {
        match self {
            Self::Version(value) => value.data.root_scopes.iter().collect(),
            Self::Path(value) => value
                .data
                .old_root_scopes
                .iter()
                .chain(value.data.new_root_scopes.iter())
                .collect(),
            Self::Eligibility(value) => value.data.root_scopes.iter().collect(),
            Self::Deleted(value) => value.data.root_scopes.iter().collect(),
        }
    }
}

fn parse_drive_event(
    payload_json: &str,
) -> Result<ParsedDriveEvent, KnowledgeWikiDriveEventConsumerError> {
    if payload_json.is_empty() || payload_json.len() > 65_536 {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive event payload must contain between 1 and 65536 bytes".to_string(),
        ));
    }
    let value: serde_json::Value = serde_json::from_str(payload_json).map_err(|_| {
        KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive event payload is not valid JSON".to_string(),
        )
    })?;
    let event_type = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            KnowledgeWikiDriveEventConsumerError::InvalidEvent(
                "Drive event type is required".to_string(),
            )
        })?;
    match event_type {
        "drive.node.version.committed.v1" => {
            parse_typed(payload_json).map(ParsedDriveEvent::Version)
        }
        "drive.node.path.changed.v1" => parse_typed(payload_json).map(ParsedDriveEvent::Path),
        "drive.node.eligibility.changed.v1" => {
            parse_typed(payload_json).map(ParsedDriveEvent::Eligibility)
        }
        "drive.node.deleted.v1" => parse_typed(payload_json).map(ParsedDriveEvent::Deleted),
        _ => Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive event type is not supported by the Wiki source stream".to_string(),
        )),
    }
}

fn parse_typed<T: DeserializeOwned>(
    payload_json: &str,
) -> Result<DriveEventEnvelope<T>, KnowledgeWikiDriveEventConsumerError> {
    serde_json::from_str(payload_json).map_err(|_| {
        KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive event does not match its versioned contract".to_string(),
        )
    })
}

fn validate_event_authority(
    scope: WikiPersistenceScope,
    publication: &WikiPublication,
    checkpoint_publication_id: u64,
    checkpoint_drive_space_uuid: &str,
    checkpoint_source_scope_uuid: &str,
    event: &ParsedDriveEvent,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    let expected_organization = scope.organization_id.to_string();
    let organization_matches = event.organization_id() == Some(expected_organization.as_str())
        || (scope.organization_id == 0 && event.organization_id().is_none());
    if event.source() != EVENT_SOURCE
        || event.specversion() != EVENT_SPEC_VERSION
        || event.declared_type() != event.event_type().as_str()
        || event.tenant_id() != scope.tenant_id.to_string()
        || !organization_matches
        || event.id().trim().is_empty()
        || event.time().trim().is_empty()
        || event.sequence_no()? == 0
        || publication.scope != scope
        || publication.id != checkpoint_publication_id
        || publication.drive_space_uuid != checkpoint_drive_space_uuid
        || publication.drive_space_uuid != event.space_id()
        || publication.source_scope_uuid.as_deref() != Some(checkpoint_source_scope_uuid)
        || event.node_id().trim().is_empty()
    {
        return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
            "Drive event authority does not match the bound Wiki source checkpoint".to_string(),
        ));
    }
    Ok(())
}

fn scoped_path(
    effects: &[DriveRootScopeEffect],
    source_scope_uuid: &str,
) -> Result<Option<String>, KnowledgeWikiDriveEventConsumerError> {
    let mut matched = None;
    for effect in effects {
        if effect.scope_id != source_scope_uuid
            || effect.scope_kind != DriveRootScopeKind::KnowledgebaseRaw
        {
            continue;
        }
        validate_relative_path(&effect.relative_path)?;
        if matched
            .as_deref()
            .is_some_and(|path| path != effect.relative_path)
        {
            return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
                "Drive event contains conflicting paths for one Wiki root scope".to_string(),
            ));
        }
        matched = Some(effect.relative_path.clone());
    }
    Ok(matched)
}

fn validate_relative_path(path: &str) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    if path.is_empty()
        || path.len() > 4_096
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive root-scope path is not normalized".to_string(),
        ));
    }
    Ok(())
}

fn validate_resolved_source(
    publication: &WikiPublication,
    source_scope_uuid: &str,
    expected_path: &str,
    resource: &KnowledgeWikiSourceResource,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    validate_relative_path(&resource.normalized_relative_path)?;
    validate_sha256_checksum(&resource.checksum_sha256_hex)?;
    if resource.subscription_uuid != source_scope_uuid
        || resource.normalized_relative_path != expected_path
        || resource.drive_node_id.trim().is_empty()
        || resource.drive_node_version_id.trim().is_empty()
        || resource.scope_status != "ACTIVE"
        || resource.node_status != "ACTIVE"
        || resource.eligibility != "ELIGIBLE"
        || publication.source_scope_uuid.as_deref() != Some(source_scope_uuid)
    {
        return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
            "Drive source resolution does not match the active Wiki root scope".to_string(),
        ));
    }
    Ok(())
}

fn validate_claimed_event_integrity(
    scope: WikiPersistenceScope,
    event: &crate::ports::knowledge_wiki_persistence::WikiDriveInboxEvent,
    parsed: &ParsedDriveEvent,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    let expected_payload_sha256 = format!("sha256:{}", sha256_hash(event.payload_json.as_bytes()));
    if event.scope != scope
        || parsed.id() != event.source_event_id
        || parsed.event_type() != event.event_type
        || parsed.sequence_no()? != event.sequence_no
        || parsed.node_id() != event.drive_node_uuid
        || parsed.drive_version_id() != event.drive_version_uuid.as_deref()
        || parsed.time() != event.source_event_time
        || event.payload_sha256 != expected_payload_sha256
    {
        return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
            "persisted inbox metadata does not match its exact Drive event payload".to_string(),
        ));
    }
    Ok(())
}

fn validate_sha256_checksum(checksum: &str) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    let Some(digest) = checksum.strip_prefix("sha256:") else {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive checksum must use the canonical sha256:<lowercase-hex> format".to_string(),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(
            "Drive checksum must use the canonical sha256:<lowercase-hex> format".to_string(),
        ));
    }
    Ok(())
}

fn classify_source_kind(path: &str, media_type: &str) -> WikiSourceFileKind {
    let extension = path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "md" | "markdown" | "mdx" => WikiSourceFileKind::Page,
        "pdf" | "doc" | "docx" | "odt" | "rtf" => WikiSourceFileKind::Document,
        "ppt" | "pptx" | "odp" => WikiSourceFileKind::Presentation,
        "xls" | "xlsx" | "ods" | "csv" => WikiSourceFileKind::Spreadsheet,
        "rs" | "go" | "java" | "kt" | "kts" | "swift" | "py" | "rb" | "php" | "c" | "h" | "cc"
        | "cpp" | "cs" | "ts" | "tsx" | "jsx" | "vue" | "sql" | "sh" | "ps1" | "yaml" | "yml"
        | "toml" => WikiSourceFileKind::Code,
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" => WikiSourceFileKind::Archive,
        _ if media_type.starts_with("image/")
            || media_type.starts_with("audio/")
            || media_type.starts_with("video/") =>
        {
            WikiSourceFileKind::Media
        }
        _ => WikiSourceFileKind::Asset,
    }
}

fn revocation(
    source_state: WikiSourceState,
    publication_state: WikiPagePublicationState,
    reason_code: &str,
) -> WikiDriveProjectionMutation {
    WikiDriveProjectionMutation::Revoke {
        source_state,
        publication_state,
        reason_code: reason_code.to_string(),
    }
}

fn is_quarantine_reason(reason: &str) -> bool {
    let reason = reason.to_ascii_uppercase();
    ["MALWARE", "QUARANTINE", "ABUSE", "PHISHING"]
        .iter()
        .any(|needle| reason.contains(needle))
}

fn parse_positive_i64_string(
    field: &str,
    value: &str,
) -> Result<u64, KnowledgeWikiDriveEventConsumerError> {
    let parsed = parse_nonnegative_i64_string(field, value)?;
    if parsed == 0 {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(format!(
            "{field} must be greater than zero"
        )));
    }
    Ok(parsed)
}

fn parse_nonnegative_i64_string(
    field: &str,
    value: &str,
) -> Result<u64, KnowledgeWikiDriveEventConsumerError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(format!(
            "{field} must be a canonical nonnegative int64 string"
        )));
    }
    let parsed = value.parse::<u64>().map_err(|_| {
        KnowledgeWikiDriveEventConsumerError::InvalidEvent(format!(
            "{field} exceeds unsigned int64"
        ))
    })?;
    if parsed > i64::MAX as u64 {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidEvent(format!(
            "{field} exceeds signed int64"
        )));
    }
    Ok(parsed)
}

fn validate_trusted_receive_request(
    request: &ReceiveKnowledgeWikiDriveTrustedEventRequest,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    if request.scope.tenant_id == 0
        || request.source_scope_uuid.trim().is_empty()
        || request.source_scope_uuid.len() > 160
        || request.payload_json.is_empty()
        || request.payload_json.len() > 65_536
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "Wiki Drive trusted event request is outside bounded limits".to_string(),
        ));
    }
    Ok(())
}

const WEBHOOK_REPLAY_WINDOW_SECONDS: i64 = 300;

fn validate_webhook_request(
    request: &ReceiveKnowledgeWikiDriveWebhookRequest,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    if request.channel_id.len() > 80
        || request.event_id.is_empty()
        || request.event_id.len() > 128
        || request.timestamp.is_empty()
        || request.signature.len() != 67
        || !request.signature.starts_with("v1=")
        || request.retry_count.is_empty()
        || request.idempotency_key.is_empty()
        || request.idempotency_key.len() > 256
        || request.idempotency_key.chars().any(char::is_whitespace)
        || request.payload_json.is_empty()
        || request.payload_json.len() > 65_536
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "Drive webhook headers and bounded body are required".to_string(),
        ));
    }
    let timestamp = request.timestamp.parse::<i64>().map_err(|_| {
        KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "x-sdkwork-event-timestamp must be epoch seconds".to_string(),
        )
    })?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| {
            KnowledgeWikiDriveEventConsumerError::InvalidRequest(
                "system clock is before Unix epoch".to_string(),
            )
        })?
        .as_secs() as i64;
    if (now - timestamp).abs() > WEBHOOK_REPLAY_WINDOW_SECONDS {
        return Err(KnowledgeWikiDriveEventConsumerError::Integrity(
            "Drive webhook timestamp is outside the replay window".to_string(),
        ));
    }
    let retry_count = request.retry_count.parse::<u32>().map_err(|_| {
        KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "x-sdkwork-event-retry-count must be a bounded integer".to_string(),
        )
    })?;
    if retry_count > 100 {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "x-sdkwork-event-retry-count exceeds the delivery limit".to_string(),
        ));
    }
    if !request.signature.strip_prefix("v1=").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "x-sdkwork-event-signature must use v1=<64 lowercase hex>".to_string(),
        ));
    }
    Ok(())
}

fn validate_process_request(
    request: &ProcessKnowledgeWikiDriveEventsRequest,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    if request.scope.tenant_id == 0
        || request.checkpoint_id == 0
        || request.actor_id == 0
        || request.worker_id.trim().is_empty()
        || request.worker_id.len() > 128
        || request.limit == 0
        || request.limit > MAX_EVENT_BATCH_SIZE
        || request.lease_seconds == 0
        || request.lease_seconds > 3_600
        || request.retry_delay_seconds == 0
        || request.retry_delay_seconds > 86_400
        || request.max_attempts == 0
        || request.max_attempts > 100
    {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            "Wiki Drive event worker request is outside bounded limits".to_string(),
        ));
    }
    Ok(())
}

fn validate_checkpoint_page_request(
    request: &ProcessKnowledgeWikiDriveCheckpointPageRequest,
) -> Result<(), KnowledgeWikiDriveEventConsumerError> {
    if request.checkpoint_limit == 0 || request.checkpoint_limit > MAX_CHECKPOINT_PAGE_SIZE {
        return Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(
            format!("checkpoint_limit must be between 1 and {MAX_CHECKPOINT_PAGE_SIZE}"),
        ));
    }
    validate_process_request(&ProcessKnowledgeWikiDriveEventsRequest {
        scope: request.scope,
        checkpoint_id: 1,
        worker_id: request.worker_id.clone(),
        actor_id: request.actor_id,
        lease_seconds: request.lease_seconds,
        limit: request.event_limit_per_checkpoint,
        retry_delay_seconds: request.retry_delay_seconds,
        max_attempts: request.max_attempts,
    })
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiDriveEventConsumerError {
    #[error("Wiki Drive event request is invalid: {0}")]
    InvalidRequest(String),
    #[error("Wiki Drive event contract is invalid: {0}")]
    InvalidEvent(String),
    #[error("Wiki Drive event integrity failed: {0}")]
    Integrity(String),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
    #[error(transparent)]
    Drive(#[from] KnowledgeWikiDriveSourceError),
}

impl KnowledgeWikiDriveEventConsumerError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "wiki_event_request_invalid",
            Self::InvalidEvent(_) => "wiki_event_contract_invalid",
            Self::Integrity(_) => "wiki_event_integrity_failed",
            Self::Persistence(_) => "wiki_event_persistence_failed",
            Self::Drive(_) => "wiki_event_drive_failed",
        }
    }
}
