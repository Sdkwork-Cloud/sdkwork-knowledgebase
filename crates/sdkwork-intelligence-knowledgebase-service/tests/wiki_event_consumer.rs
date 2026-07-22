use std::sync::Mutex;

use async_trait::async_trait;
use sdkwork_drive_contract::drive::events::{
    derive_webhook_signing_key, sign_webhook, DriveEventEnvelope, DriveNodeDeletedV1Data,
    DriveNodeEligibility, DriveNodeEligibilityChangedV1Data, DriveNodePathChangedV1Data,
    DriveNodeVersionCommittedV1Data, DriveRootScopeEffect, DriveRootScopeKind,
};
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_wiki_drive_source::{
            EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveScope, KnowledgeWikiDriveSource,
            KnowledgeWikiDriveSourceError, KnowledgeWikiSourceResource, KnowledgebaseRawScope,
            ReadKnowledgeWikiSourceRequest, ResolveKnowledgeWikiSourceRequest,
        },
        knowledge_wiki_persistence::*,
    },
    wiki_event_consumer::{
        resolve_knowledge_wiki_drive_trusted_event_targets, KnowledgeWikiDriveEventBatchResult,
        KnowledgeWikiDriveEventConsumerError, KnowledgeWikiDriveEventConsumerService,
        ProcessKnowledgeWikiDriveCheckpointPageRequest, ProcessKnowledgeWikiDriveEventsRequest,
        ReceiveKnowledgeWikiDriveTrustedEventRequest, ReceiveKnowledgeWikiDriveWebhookRequest,
    },
};
use sdkwork_utils_rust::{hmac_sha256, sha256_hash};

const SCOPE: WikiPersistenceScope = WikiPersistenceScope {
    tenant_id: 101,
    organization_id: 202,
};
const PUBLICATION_ID: u64 = 501;
const CHECKPOINT_ID: u64 = 701;
const DRIVE_SPACE_ID: &str = "drive-space-501";
const SOURCE_SCOPE_ID: &str = "11111111-1111-4111-8111-111111111501";
const SECOND_SOURCE_SCOPE_ID: &str = "22222222-2222-4222-8222-222222222502";
const CURRENT_WEBHOOK_SECRET: &str = "current-knowledgebase-webhook-master-secret-501";
const PREVIOUS_WEBHOOK_SECRET: &str = "previous-knowledgebase-webhook-master-secret-501";

#[test]
fn embedded_relay_targets_are_resolved_only_from_authoritative_raw_scopes() {
    let targets = resolve_knowledge_wiki_drive_trusted_event_targets(&version_event(
        1,
        SOURCE_SCOPE_ID,
        "guide/start.md",
    ))
    .expect("resolve trusted event targets");

    assert_eq!(targets.scope, SCOPE);
    assert_eq!(targets.drive_space_uuid, DRIVE_SPACE_ID);
    assert_eq!(targets.source_scope_uuids, vec![SOURCE_SCOPE_ID]);
}

#[test]
fn embedded_path_relay_targets_include_deduplicated_old_and_new_raw_scopes() {
    let targets = resolve_knowledge_wiki_drive_trusted_event_targets(&path_event(
        2,
        Some((SOURCE_SCOPE_ID, "guide/start.md")),
        Some((SECOND_SOURCE_SCOPE_ID, "guide/renamed.md")),
    ))
    .expect("resolve old and new trusted event targets");

    assert_eq!(
        targets.source_scope_uuids,
        vec![SOURCE_SCOPE_ID, SECOND_SOURCE_SCOPE_ID]
    );
}

#[tokio::test]
async fn authoritative_event_is_received_with_exact_hash_and_deduplicated() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
    let service = service(&persistence, &drive);
    let payload = version_event(1, SOURCE_SCOPE_ID, "guide/start.md");

    let first = service
        .receive_trusted(receive_request(payload.clone()))
        .await
        .expect("receive authoritative event");
    let duplicate = service
        .receive_trusted(receive_request(payload.clone()))
        .await
        .expect("deduplicate exact replay");

    assert_eq!(first.disposition, WikiDriveEventReceiveDisposition::Ready);
    assert_eq!(
        duplicate.disposition,
        WikiDriveEventReceiveDisposition::Duplicate
    );
    let received = persistence.received();
    assert_eq!(received.len(), 2);
    assert_eq!(
        received[0].payload_sha256,
        format!("sha256:{}", sha256_hash(payload.as_bytes()))
    );
    assert_eq!(received[0].drive_version_uuid.as_deref(), Some("version-1"));
}

#[tokio::test]
async fn signed_webhook_accepts_current_and_previous_secrets_and_deduplicates() {
    for secret in [CURRENT_WEBHOOK_SECRET, PREVIOUS_WEBHOOK_SECRET] {
        let persistence = FakePersistence::new();
        let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
        let service = service(&persistence, &drive).with_webhook_signing_secrets(vec![
            CURRENT_WEBHOOK_SECRET.to_string(),
            PREVIOUS_WEBHOOK_SECRET.to_string(),
        ]);
        let payload = version_event(1, SOURCE_SCOPE_ID, "guide/start.md");
        let request = webhook_request(&payload, secret);

        let first = service
            .receive_webhook(request.clone())
            .await
            .expect("signed Drive webhook should be received");
        let replay = service
            .receive_webhook(request)
            .await
            .expect("exact signed replay should be idempotent");

        assert_eq!(first.disposition, WikiDriveEventReceiveDisposition::Ready);
        assert_eq!(
            replay.disposition,
            WikiDriveEventReceiveDisposition::Duplicate
        );
    }
}

#[tokio::test]
async fn signed_webhook_rejects_tampering_stale_delivery_and_noncanonical_signature() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
    let service = service(&persistence, &drive)
        .with_webhook_signing_secrets(vec![CURRENT_WEBHOOK_SECRET.to_string()]);
    let payload = version_event(1, SOURCE_SCOPE_ID, "guide/start.md");

    let mut tampered = webhook_request(&payload, CURRENT_WEBHOOK_SECRET);
    tampered.payload_json.push(' ');
    assert!(matches!(
        service.receive_webhook(tampered).await,
        Err(KnowledgeWikiDriveEventConsumerError::Integrity(_))
    ));

    let mut mismatched_event = webhook_request(&payload, CURRENT_WEBHOOK_SECRET);
    mismatched_event.event_id = "event-version-other".to_string();
    assert!(matches!(
        service.receive_webhook(mismatched_event).await,
        Err(KnowledgeWikiDriveEventConsumerError::Integrity(_))
    ));

    let mut stale = webhook_request(&payload, CURRENT_WEBHOOK_SECRET);
    stale.timestamp = "1".to_string();
    assert!(matches!(
        service.receive_webhook(stale).await,
        Err(KnowledgeWikiDriveEventConsumerError::Integrity(_))
    ));

    let mut uppercase = webhook_request(&payload, CURRENT_WEBHOOK_SECRET);
    uppercase.signature = uppercase.signature.to_ascii_uppercase();
    assert!(matches!(
        service.receive_webhook(uppercase).await,
        Err(KnowledgeWikiDriveEventConsumerError::InvalidRequest(_))
    ));
    assert!(persistence.received().is_empty());
}

#[tokio::test]
async fn receive_rejects_events_outside_the_bound_tenant_organization_and_space() {
    for (field, replacement) in [
        ("source", serde_json::json!("untrusted-drive")),
        ("specversion", serde_json::json!("0.3")),
        ("tenantId", serde_json::json!("999")),
        ("organizationId", serde_json::json!("999")),
    ] {
        let persistence = FakePersistence::new();
        let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
        let service = service(&persistence, &drive);
        let mut value: serde_json::Value =
            serde_json::from_str(&version_event(1, SOURCE_SCOPE_ID, "guide/start.md")).unwrap();
        value[field] = replacement;

        let error = service
            .receive_trusted(receive_request(value.to_string()))
            .await
            .expect_err("authority mismatch must be rejected");
        assert!(matches!(
            error,
            KnowledgeWikiDriveEventConsumerError::Integrity(_)
        ));
        assert!(persistence.received().is_empty());
    }

    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
    let service = service(&persistence, &drive);
    let mut value: serde_json::Value =
        serde_json::from_str(&version_event(1, SOURCE_SCOPE_ID, "guide/start.md")).unwrap();
    value["data"]["spaceId"] = serde_json::json!("drive-space-other");
    assert!(matches!(
        service
            .receive_trusted(receive_request(value.to_string()))
            .await
            .expect_err("Drive Space mismatch must be rejected"),
        KnowledgeWikiDriveEventConsumerError::Integrity(_)
    ));
}

#[tokio::test]
async fn version_move_quarantine_and_delete_events_map_to_bounded_mutations() {
    let cases = [
        (
            version_event(1, SOURCE_SCOPE_ID, "guide/start.md"),
            ExpectedMutation::Upsert,
        ),
        (
            path_event(
                1,
                Some((SOURCE_SCOPE_ID, "guide/start.md")),
                Some((SOURCE_SCOPE_ID, "guide/renamed.md")),
            ),
            ExpectedMutation::MoveWithin,
        ),
        (
            path_event(1, Some((SOURCE_SCOPE_ID, "guide/start.md")), None),
            ExpectedMutation::MovedOut,
        ),
        (
            eligibility_event(1, DriveNodeEligibility::Ineligible, "MALWARE_DETECTED"),
            ExpectedMutation::Quarantined,
        ),
        (deleted_event(1), ExpectedMutation::Deleted),
    ];

    for (payload, expected) in cases {
        let persistence = FakePersistence::new();
        let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
        let result = receive_and_process(&persistence, &drive, payload).await;
        assert_eq!(result.applied, 1);
        assert_eq!(result.retried, 0);
        let applied = persistence.applied();
        assert_eq!(applied.len(), 1);
        expected.assert_matches(&applied[0].mutation);
    }
}

#[tokio::test]
async fn eligible_event_resolves_the_pinned_drive_version_before_upsert() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-2"));

    let result = receive_and_process(
        &persistence,
        &drive,
        eligibility_event(1, DriveNodeEligibility::Eligible, "RESTORED"),
    )
    .await;

    assert_eq!(result.applied, 1);
    let requests = drive.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].subscription_uuid, SOURCE_SCOPE_ID);
    assert_eq!(requests[0].relative_path, "guide/start.md");
    assert_eq!(
        requests[0].pinned_node_version_id.as_deref(),
        Some("version-2")
    );
    ExpectedMutation::Upsert.assert_matches(&persistence.applied()[0].mutation);
}

#[tokio::test]
async fn unrelated_root_scope_event_advances_without_mutating_a_wiki_projection() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));

    let result = receive_and_process(
        &persistence,
        &drive,
        version_event(1, "another-root-scope", "guide/start.md"),
    )
    .await;

    assert_eq!(result.applied, 1);
    assert_eq!(
        persistence.applied()[0].mutation,
        WikiDriveProjectionMutation::None
    );
    assert!(drive.requests().is_empty());
}

#[tokio::test]
async fn checkpoint_page_processes_multiple_checkpoints_and_returns_keyset_cursor() {
    let persistence = FakePersistence::with_checkpoints(vec![
        checkpoint_with_id(701),
        checkpoint_with_id(702),
        checkpoint_with_id(703),
    ]);
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
    let service = service(&persistence, &drive);

    let first = service
        .process_checkpoint_page(ProcessKnowledgeWikiDriveCheckpointPageRequest {
            scope: SCOPE,
            after_checkpoint_id: None,
            worker_id: "wiki-worker-page-1".to_string(),
            actor_id: 9001,
            lease_seconds: 60,
            checkpoint_limit: 2,
            event_limit_per_checkpoint: 10,
            retry_delay_seconds: 30,
            max_attempts: 3,
        })
        .await
        .expect("process first checkpoint page");
    assert_eq!(first.checkpoints_processed, 2);
    assert_eq!(first.events, KnowledgeWikiDriveEventBatchResult::default());
    assert_eq!(first.next_after_checkpoint_id, Some(702));

    let second = service
        .process_checkpoint_page(ProcessKnowledgeWikiDriveCheckpointPageRequest {
            scope: SCOPE,
            after_checkpoint_id: first.next_after_checkpoint_id,
            worker_id: "wiki-worker-page-1".to_string(),
            actor_id: 9001,
            lease_seconds: 60,
            checkpoint_limit: 2,
            event_limit_per_checkpoint: 10,
            retry_delay_seconds: 30,
            max_attempts: 3,
        })
        .await
        .expect("process second checkpoint page");
    assert_eq!(second.checkpoints_processed, 1);
    assert_eq!(second.next_after_checkpoint_id, None);
}

#[tokio::test]
async fn drive_resolution_failure_retries_without_applying_or_advancing() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::failure(KnowledgeWikiDriveSourceError::Upstream(
        "temporary Drive outage".to_string(),
    ));

    let result = receive_and_process(
        &persistence,
        &drive,
        eligibility_event(1, DriveNodeEligibility::Eligible, "RESTORED"),
    )
    .await;

    assert_eq!(result.applied, 0);
    assert_eq!(result.retried, 1);
    assert!(persistence.applied().is_empty());
    assert_eq!(persistence.retried().len(), 1);
}

#[tokio::test]
async fn tampered_inbox_payload_hash_retries_without_applying() {
    let persistence = FakePersistence::new();
    let drive = FakeDriveSource::success(resolved_source("guide/start.md", "version-1"));
    let consumer = service(&persistence, &drive);
    consumer
        .receive_trusted(receive_request(version_event(
            1,
            SOURCE_SCOPE_ID,
            "guide/start.md",
        )))
        .await
        .expect("receive event");
    persistence.tamper_payload_hash();

    let result = consumer
        .process_batch(process_request())
        .await
        .expect("retry tampered inbox event");

    assert_eq!(result.applied, 0);
    assert_eq!(result.retried, 1);
    assert!(persistence.applied().is_empty());
    assert_eq!(
        persistence.retried()[0].error_code,
        "wiki_event_integrity_failed"
    );
}

async fn receive_and_process(
    persistence: &FakePersistence,
    drive: &FakeDriveSource,
    payload: String,
) -> KnowledgeWikiDriveEventBatchResult {
    let consumer = service(persistence, drive);
    consumer
        .receive_trusted(receive_request(payload))
        .await
        .expect("receive Drive event");
    consumer
        .process_batch(process_request())
        .await
        .expect("process Drive event")
}

fn service<'a>(
    persistence: &'a FakePersistence,
    drive: &'a FakeDriveSource,
) -> KnowledgeWikiDriveEventConsumerService<'a> {
    KnowledgeWikiDriveEventConsumerService::new(persistence, persistence, persistence, drive)
}

fn receive_request(payload_json: String) -> ReceiveKnowledgeWikiDriveTrustedEventRequest {
    ReceiveKnowledgeWikiDriveTrustedEventRequest {
        scope: SCOPE,
        source_scope_uuid: SOURCE_SCOPE_ID.to_string(),
        payload_json,
    }
}

fn webhook_request(
    payload_json: &str,
    master_secret: &str,
) -> ReceiveKnowledgeWikiDriveWebhookRequest {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("test clock should be after Unix epoch")
        .as_secs()
        .to_string();
    let verification_token = hmac_sha256(SOURCE_SCOPE_ID.as_bytes(), master_secret.as_bytes());
    let signing_key = derive_webhook_signing_key(&verification_token);
    ReceiveKnowledgeWikiDriveWebhookRequest {
        channel_id: format!("kbraw:{SOURCE_SCOPE_ID}"),
        event_id: "event-version-1".to_string(),
        timestamp: timestamp.clone(),
        signature: sign_webhook(&timestamp, payload_json.as_bytes(), signing_key.as_bytes()),
        retry_count: "0".to_string(),
        idempotency_key: format!("outbox-test:kbraw:{SOURCE_SCOPE_ID}"),
        payload_json: payload_json.to_string(),
    }
}

fn process_request() -> ProcessKnowledgeWikiDriveEventsRequest {
    ProcessKnowledgeWikiDriveEventsRequest {
        scope: SCOPE,
        checkpoint_id: CHECKPOINT_ID,
        worker_id: "wiki-event-test-worker".to_string(),
        actor_id: 9001,
        lease_seconds: 30,
        limit: 10,
        retry_delay_seconds: 1,
        max_attempts: 3,
    }
}

#[derive(Clone, Copy)]
enum ExpectedMutation {
    Upsert,
    MoveWithin,
    MovedOut,
    Quarantined,
    Deleted,
}

impl ExpectedMutation {
    fn assert_matches(self, mutation: &WikiDriveProjectionMutation) {
        match (self, mutation) {
            (Self::Upsert, WikiDriveProjectionMutation::Upsert(metadata)) => {
                assert_eq!(metadata.source_path, "guide/start.md");
                assert_eq!(metadata.content_sha256, canonical_checksum());
                assert_eq!(metadata.file_kind, WikiSourceFileKind::Page);
            }
            (Self::MoveWithin, WikiDriveProjectionMutation::MoveWithin { source_path }) => {
                assert_eq!(source_path, "guide/renamed.md");
            }
            (
                Self::MovedOut,
                WikiDriveProjectionMutation::Revoke {
                    source_state,
                    publication_state,
                    reason_code,
                },
            ) => {
                assert_eq!(*source_state, WikiSourceState::Deleted);
                assert_eq!(*publication_state, WikiPagePublicationState::Archived);
                assert_eq!(reason_code, "moved_outside_source_root");
            }
            (
                Self::Quarantined,
                WikiDriveProjectionMutation::Revoke {
                    source_state,
                    publication_state,
                    reason_code,
                },
            ) => {
                assert_eq!(*source_state, WikiSourceState::Quarantined);
                assert_eq!(*publication_state, WikiPagePublicationState::Unpublished);
                assert_eq!(reason_code, "drive_quarantined");
            }
            (
                Self::Deleted,
                WikiDriveProjectionMutation::Revoke {
                    source_state,
                    publication_state,
                    reason_code,
                },
            ) => {
                assert_eq!(*source_state, WikiSourceState::Deleted);
                assert_eq!(*publication_state, WikiPagePublicationState::Archived);
                assert_eq!(reason_code, "drive_deleted");
            }
            _ => panic!("unexpected Wiki Drive mutation: {mutation:?}"),
        }
    }
}

struct FakePersistence {
    publication: WikiPublication,
    checkpoints: Vec<WikiDriveCheckpoint>,
    state: Mutex<FakeInboxState>,
}

#[derive(Default)]
struct FakeInboxState {
    events: Vec<WikiDriveInboxEvent>,
    received: Vec<ReceiveWikiDriveEventRequest>,
    applied: Vec<ApplyWikiDriveEventRequest>,
    retried: Vec<RetryWikiDriveEventRequest>,
}

impl FakePersistence {
    fn new() -> Self {
        let checkpoint = checkpoint();
        Self {
            publication: publication(),
            checkpoints: vec![checkpoint],
            state: Mutex::new(FakeInboxState::default()),
        }
    }

    fn with_checkpoints(checkpoints: Vec<WikiDriveCheckpoint>) -> Self {
        assert!(
            !checkpoints.is_empty(),
            "at least one checkpoint is required"
        );
        Self {
            publication: publication(),
            checkpoints,
            state: Mutex::new(FakeInboxState::default()),
        }
    }

    fn received(&self) -> Vec<ReceiveWikiDriveEventRequest> {
        self.state.lock().unwrap().received.clone()
    }

    fn applied(&self) -> Vec<ApplyWikiDriveEventRequest> {
        self.state.lock().unwrap().applied.clone()
    }

    fn retried(&self) -> Vec<RetryWikiDriveEventRequest> {
        self.state.lock().unwrap().retried.clone()
    }

    fn tamper_payload_hash(&self) {
        self.state.lock().unwrap().events[0].payload_sha256 = format!("sha256:{}", "f".repeat(64));
    }
}

#[async_trait]
impl WikiPublicationStore for FakePersistence {
    async fn provision_publication(
        &self,
        _request: ProvisionWikiPublicationRequest,
    ) -> Result<WikiPublicationProvisioningResult, WikiPersistenceError> {
        unimplemented!()
    }

    async fn get_publication(
        &self,
        scope: WikiPersistenceScope,
        site_publication_id: u64,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        if scope == self.publication.scope && site_publication_id == self.publication.id {
            Ok(self.publication.clone())
        } else {
            Err(WikiPersistenceError::NotFound {
                resource: "wiki_publication",
                id: site_publication_id,
            })
        }
    }

    async fn get_publication_for_space(
        &self,
        _scope: WikiPersistenceScope,
        _space_id: u64,
    ) -> Result<Option<WikiPublication>, WikiPersistenceError> {
        unimplemented!()
    }

    async fn bind_source_scope(
        &self,
        _request: BindWikiSourceScopeRequest,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        unimplemented!()
    }
}

#[async_trait]
impl WikiDriveCheckpointStore for FakePersistence {
    async fn provision_checkpoint(
        &self,
        _request: ProvisionWikiDriveCheckpointRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        unimplemented!()
    }

    async fn get_checkpoint(
        &self,
        scope: WikiPersistenceScope,
        checkpoint_id: u64,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        self.checkpoints
            .iter()
            .find(|checkpoint| checkpoint.scope == scope && checkpoint.id == checkpoint_id)
            .cloned()
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_drive_checkpoint",
                id: checkpoint_id,
            })
    }

    async fn find_checkpoint_by_drive_scope(
        &self,
        scope: WikiPersistenceScope,
        drive_space_uuid: &str,
        source_scope_uuid: &str,
    ) -> Result<Option<WikiDriveCheckpoint>, WikiPersistenceError> {
        Ok(self
            .checkpoints
            .iter()
            .find(|checkpoint| {
                checkpoint.scope == scope
                    && checkpoint.drive_space_uuid == drive_space_uuid
                    && checkpoint.source_scope_uuid == source_scope_uuid
            })
            .cloned())
    }

    async fn list_checkpoints(
        &self,
        request: ListWikiDriveCheckpointsRequest,
    ) -> Result<WikiDriveCheckpointPage, WikiPersistenceError> {
        let after = request.after_checkpoint_id.unwrap_or(0);
        let mut checkpoints = self
            .checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.scope == request.scope && checkpoint.id > after)
            .cloned()
            .collect::<Vec<_>>();
        let has_more = checkpoints.len() > request.limit as usize;
        if has_more {
            checkpoints.truncate(request.limit as usize);
        }
        Ok(WikiDriveCheckpointPage {
            next_after_checkpoint_id: has_more
                .then(|| checkpoints.last().map(|checkpoint| checkpoint.id))
                .flatten(),
            checkpoints,
        })
    }

    async fn claim_reconciliation(
        &self,
        _request: ClaimWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        unimplemented!()
    }

    async fn advance_reconciliation(
        &self,
        _request: AdvanceWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        unimplemented!()
    }

    async fn complete_reconciliation(
        &self,
        _request: CompleteWikiReconciliationRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        unimplemented!()
    }
}

#[async_trait]
impl WikiDriveEventInboxStore for FakePersistence {
    async fn receive_event(
        &self,
        request: ReceiveWikiDriveEventRequest,
    ) -> Result<WikiDriveEventReceipt, WikiPersistenceError> {
        let mut state = self.state.lock().unwrap();
        state.received.push(request.clone());
        if let Some(event) = state
            .events
            .iter()
            .find(|event| event.source_event_id == request.source_event_id)
            .cloned()
        {
            return Ok(WikiDriveEventReceipt {
                event,
                disposition: WikiDriveEventReceiveDisposition::Duplicate,
            });
        }
        let event = inbox_event(state.events.len() as u64 + 1, request);
        state.events.push(event.clone());
        Ok(WikiDriveEventReceipt {
            event,
            disposition: WikiDriveEventReceiveDisposition::Ready,
        })
    }

    async fn claim_events(
        &self,
        request: ClaimWikiDriveEventsRequest,
    ) -> Result<Vec<WikiDriveInboxEvent>, WikiPersistenceError> {
        let mut state = self.state.lock().unwrap();
        let Some(event) = state.events.iter_mut().find(|event| {
            event.scope == request.scope
                && event.checkpoint_id == request.checkpoint_id
                && matches!(
                    event.processing_state,
                    WikiDriveEventProcessingState::Received | WikiDriveEventProcessingState::Retry
                )
                && event.lease_token.is_none()
        }) else {
            return Ok(Vec::new());
        };
        event.lease_token = Some("lease-1".to_string());
        Ok(vec![event.clone()])
    }

    async fn complete_event(
        &self,
        _request: CompleteWikiDriveEventRequest,
    ) -> Result<WikiDriveInboxEvent, WikiPersistenceError> {
        unimplemented!()
    }

    async fn apply_event(
        &self,
        request: ApplyWikiDriveEventRequest,
    ) -> Result<WikiDriveEventApplicationResult, WikiPersistenceError> {
        let mut state = self.state.lock().unwrap();
        state.applied.push(request.clone());
        let event = state
            .events
            .iter_mut()
            .find(|event| event.id == request.complete.event_id)
            .expect("claimed event must exist");
        event.processing_state = WikiDriveEventProcessingState::Applied;
        event.lease_token = None;
        Ok(WikiDriveEventApplicationResult {
            event: event.clone(),
            projection: None,
            public_route_change: None,
        })
    }

    async fn retry_event(
        &self,
        request: RetryWikiDriveEventRequest,
    ) -> Result<WikiDriveInboxEvent, WikiPersistenceError> {
        let mut state = self.state.lock().unwrap();
        state.retried.push(request.clone());
        let event = state
            .events
            .iter_mut()
            .find(|event| event.id == request.event_id)
            .expect("claimed event must exist");
        event.attempt_count += 1;
        event.processing_state = if event.attempt_count >= request.max_attempts {
            WikiDriveEventProcessingState::DeadLetter
        } else {
            WikiDriveEventProcessingState::Retry
        };
        event.lease_token = None;
        Ok(event.clone())
    }
}

struct FakeDriveSource {
    response: Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError>,
    requests: Mutex<Vec<ResolveKnowledgeWikiSourceRequest>>,
}

impl FakeDriveSource {
    fn success(resource: KnowledgeWikiSourceResource) -> Self {
        Self {
            response: Ok(resource),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn failure(error: KnowledgeWikiDriveSourceError) -> Self {
        Self {
            response: Err(error),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<ResolveKnowledgeWikiSourceRequest> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeWikiDriveScope for FakeDriveSource {
    async fn ensure_raw_scope(
        &self,
        _request: EnsureKnowledgebaseRawScopeRequest,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        unimplemented!()
    }

    async fn retrieve_raw_scope(
        &self,
        _subscription_uuid: &str,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        unimplemented!()
    }
}

#[async_trait]
impl KnowledgeWikiDriveSource for FakeDriveSource {
    async fn resolve_source(
        &self,
        request: ResolveKnowledgeWikiSourceRequest,
    ) -> Result<KnowledgeWikiSourceResource, KnowledgeWikiDriveSourceError> {
        self.requests.lock().unwrap().push(request);
        self.response.clone()
    }

    async fn read_pinned_source(
        &self,
        _request: ReadKnowledgeWikiSourceRequest,
    ) -> Result<Vec<u8>, KnowledgeWikiDriveSourceError> {
        unimplemented!()
    }
}

fn publication() -> WikiPublication {
    WikiPublication {
        id: PUBLICATION_ID,
        uuid: "wiki-publication-501".to_string(),
        scope: SCOPE,
        space_id: 501,
        drive_space_uuid: DRIVE_SPACE_ID.to_string(),
        source_root_node_uuid: Some("raw-node-501".to_string()),
        source_scope_uuid: Some(SOURCE_SCOPE_ID.to_string()),
        wiki_status: WikiPublicationStatus::Active,
        title: "Docs".to_string(),
        homepage_source_path: "index.md".to_string(),
        publication_mode: WikiPublicationMode::AutoPublicAfterChecks,
        default_visibility: WikiVisibility::Public,
        update_policy: WikiUpdatePolicy::KeepLastPublicUntilReady,
        provider_generation: 1,
        navigation_generation: 1,
        search_generation: 1,
        last_projected_drive_checkpoint: 0,
        version: 1,
    }
}

fn checkpoint() -> WikiDriveCheckpoint {
    WikiDriveCheckpoint {
        id: CHECKPOINT_ID,
        uuid: "wiki-checkpoint-701".to_string(),
        scope: SCOPE,
        site_publication_id: PUBLICATION_ID,
        drive_space_uuid: DRIVE_SPACE_ID.to_string(),
        source_scope_uuid: SOURCE_SCOPE_ID.to_string(),
        last_sequence_no: 0,
        last_event_id: None,
        stream_state: WikiDriveStreamState::Healthy,
        gap_from_sequence_no: None,
        gap_to_sequence_no: None,
        reconciliation_cursor: None,
        lease_token: None,
        fence_token: 0,
        version: 1,
    }
}

fn checkpoint_with_id(id: u64) -> WikiDriveCheckpoint {
    WikiDriveCheckpoint {
        id,
        uuid: format!("wiki-checkpoint-{id}"),
        ..checkpoint()
    }
}

fn inbox_event(id: u64, request: ReceiveWikiDriveEventRequest) -> WikiDriveInboxEvent {
    WikiDriveInboxEvent {
        id,
        uuid: format!("inbox-event-{id}"),
        scope: request.scope,
        site_publication_id: request.site_publication_id,
        checkpoint_id: request.checkpoint_id,
        source_event_id: request.source_event_id,
        event_type: request.event_type,
        sequence_no: request.sequence_no,
        drive_node_uuid: request.drive_node_uuid,
        drive_version_uuid: request.drive_version_uuid,
        payload_sha256: request.payload_sha256,
        payload_json: request.payload_json,
        source_event_time: request.source_event_time,
        processing_state: WikiDriveEventProcessingState::Received,
        attempt_count: 0,
        lease_token: None,
        version: 0,
    }
}

fn resolved_source(path: &str, version_id: &str) -> KnowledgeWikiSourceResource {
    KnowledgeWikiSourceResource {
        scope_type: "ROOT_SCOPE_SUBSCRIPTION".to_string(),
        subscription_uuid: SOURCE_SCOPE_ID.to_string(),
        scope_generation: "7".to_string(),
        normalized_relative_path: path.to_string(),
        resource_type: "FILE".to_string(),
        drive_node_id: "node-1".to_string(),
        drive_node_version_id: version_id.to_string(),
        version_no: "2".to_string(),
        checksum_sha256_hex: canonical_checksum(),
        etag: format!("\"{}\"", canonical_checksum()),
        content_type: "text/markdown".to_string(),
        content_length: 128,
        last_modified: "2026-07-21T00:00:00Z".to_string(),
        scope_status: "ACTIVE".to_string(),
        node_status: "ACTIVE".to_string(),
        eligibility: "ELIGIBLE".to_string(),
    }
}

fn version_event(sequence_no: i64, scope_id: &str, path: &str) -> String {
    serde_json::to_string(&DriveEventEnvelope::new(
        format!("event-version-{sequence_no}"),
        "drive.node.version.committed.v1",
        "2026-07-21T00:00:00Z",
        SCOPE.tenant_id.to_string(),
        Some(SCOPE.organization_id.to_string()),
        "drive://spaces/drive-space-501/nodes/node-1",
        "9001",
        sequence_no,
        DriveNodeVersionCommittedV1Data {
            operation_id: "upload-1".to_string(),
            space_id: DRIVE_SPACE_ID.to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/drive-space-501/nodes/node-1".to_string(),
            drive_version_id: "version-1".to_string(),
            version_no: "1".to_string(),
            space_relative_path: format!("sources/raw/{path}"),
            content_type: "text/markdown".to_string(),
            content_length: "128".to_string(),
            checksum_sha256_hex: canonical_checksum(),
            root_scopes: vec![root_effect(scope_id, path)],
        },
    ))
    .unwrap()
}

fn path_event(sequence_no: i64, old: Option<(&str, &str)>, new: Option<(&str, &str)>) -> String {
    serde_json::to_string(&DriveEventEnvelope::new(
        format!("event-path-{sequence_no}"),
        "drive.node.path.changed.v1",
        "2026-07-21T00:00:00Z",
        SCOPE.tenant_id.to_string(),
        Some(SCOPE.organization_id.to_string()),
        "drive://spaces/drive-space-501/nodes/node-1",
        "9001",
        sequence_no,
        DriveNodePathChangedV1Data {
            operation_id: "move-1".to_string(),
            space_id: DRIVE_SPACE_ID.to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/drive-space-501/nodes/node-1".to_string(),
            old_space_relative_path: "sources/raw/guide/start.md".to_string(),
            new_space_relative_path: "sources/raw/guide/renamed.md".to_string(),
            old_root_scopes: old
                .map(|(scope, path)| vec![root_effect(scope, path)])
                .unwrap_or_default(),
            new_root_scopes: new
                .map(|(scope, path)| vec![root_effect(scope, path)])
                .unwrap_or_default(),
        },
    ))
    .unwrap()
}

fn eligibility_event(sequence_no: i64, eligibility: DriveNodeEligibility, reason: &str) -> String {
    serde_json::to_string(&DriveEventEnvelope::new(
        format!("event-eligibility-{sequence_no}"),
        "drive.node.eligibility.changed.v1",
        "2026-07-21T00:00:00Z",
        SCOPE.tenant_id.to_string(),
        Some(SCOPE.organization_id.to_string()),
        "drive://spaces/drive-space-501/nodes/node-1",
        "9001",
        sequence_no,
        DriveNodeEligibilityChangedV1Data {
            operation_id: "eligibility-1".to_string(),
            space_id: DRIVE_SPACE_ID.to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/drive-space-501/nodes/node-1".to_string(),
            drive_version_id: Some("version-2".to_string()),
            version_no: Some("2".to_string()),
            space_relative_path: "sources/raw/guide/start.md".to_string(),
            old_eligibility: if eligibility == DriveNodeEligibility::Eligible {
                DriveNodeEligibility::Ineligible
            } else {
                DriveNodeEligibility::Eligible
            },
            new_eligibility: eligibility,
            reason: reason.to_string(),
            root_scopes: vec![root_effect(SOURCE_SCOPE_ID, "guide/start.md")],
        },
    ))
    .unwrap()
}

fn deleted_event(sequence_no: i64) -> String {
    serde_json::to_string(&DriveEventEnvelope::new(
        format!("event-delete-{sequence_no}"),
        "drive.node.deleted.v1",
        "2026-07-21T00:00:00Z",
        SCOPE.tenant_id.to_string(),
        Some(SCOPE.organization_id.to_string()),
        "drive://spaces/drive-space-501/nodes/node-1",
        "9001",
        sequence_no,
        DriveNodeDeletedV1Data {
            operation_id: "delete-1".to_string(),
            space_id: DRIVE_SPACE_ID.to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/drive-space-501/nodes/node-1".to_string(),
            drive_version_id: Some("version-1".to_string()),
            version_no: Some("1".to_string()),
            last_space_relative_path: "sources/raw/guide/start.md".to_string(),
            deletion_reason: "PERMANENT_DELETE".to_string(),
            root_scopes: vec![root_effect(SOURCE_SCOPE_ID, "guide/start.md")],
        },
    ))
    .unwrap()
}

fn root_effect(scope_id: &str, path: &str) -> DriveRootScopeEffect {
    DriveRootScopeEffect {
        scope_id: scope_id.to_string(),
        scope_kind: DriveRootScopeKind::KnowledgebaseRaw,
        relative_path: path.to_string(),
        root_generation: Some("7".to_string()),
    }
}

fn canonical_checksum() -> String {
    format!("sha256:{}", "a".repeat(64))
}
