use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_wiki_drive_source::{
            EnsureKnowledgebaseRawScopeRequest, KnowledgeWikiDriveEventDeliveryMode,
            KnowledgeWikiDriveScope, KnowledgeWikiDriveSourceError, KnowledgebaseRawScope,
            KnowledgebaseRawScopeEventDelivery, RenewKnowledgebaseRawScopeEventDeliveryRequest,
        },
        knowledge_wiki_persistence::*,
    },
    wiki_backfill::{
        KnowledgeWikiBackfillService, RunWikiPublicationBackfillRequest,
        WikiPublicationBackfillDisposition,
    },
    wiki_initialization::{InitializeKnowledgeWikiRequest, KnowledgeWikiInitializationService},
};

const SCOPE: WikiPersistenceScope = WikiPersistenceScope {
    tenant_id: 101,
    organization_id: 202,
};

#[tokio::test]
async fn initialization_ensures_canonical_raw_scope_and_checkpoint() {
    let persistence = MemoryWikiPersistence::with_publication(publication(501, "drive-501"));
    let drive_scope = FixedDriveScope::default();
    let initializer =
        KnowledgeWikiInitializationService::new(&persistence, &persistence, &drive_scope);

    let result = initializer
        .initialize(InitializeKnowledgeWikiRequest {
            scope: SCOPE,
            space_id: 501,
            knowledgebase_uuid: "knowledgebase-501".to_string(),
            drive_space_uuid: "drive-501".to_string(),
            actor_id: 9001,
        })
        .await
        .expect("initialize Wiki publication");

    assert_eq!(
        result.publication.source_root_node_uuid.as_deref(),
        Some("raw-node-drive-501")
    );
    assert_eq!(
        result.publication.source_scope_uuid.as_deref(),
        Some("scope-drive-501")
    );
    assert_eq!(result.checkpoint.source_scope_uuid, "scope-drive-501");
    assert_eq!(result.publication.wiki_status, WikiPublicationStatus::Ready);
    assert_eq!(result.event_delivery.channel_id, "kbraw:scope-drive-501");
}

#[tokio::test]
async fn backfill_stops_before_advancing_past_a_failed_candidate() {
    let persistence = MemoryWikiPersistence::default();
    let drive_scope = FixedDriveScope {
        fail_for_drive: Some("drive-502".to_string()),
    };
    let initializer =
        KnowledgeWikiInitializationService::new(&persistence, &persistence, &drive_scope);
    let candidates = FixedBackfillStore {
        candidates: vec![candidate(501), candidate(502), candidate(503)],
    };
    let service = KnowledgeWikiBackfillService::new(&candidates, &persistence, &initializer);

    let result = service
        .run_page(RunWikiPublicationBackfillRequest {
            scope: SCOPE,
            after_space_id: Some(500),
            page_size: 10,
            actor_id: 9001,
            dry_run: false,
        })
        .await
        .expect("run bounded backfill page");

    assert!(result.stopped_on_failure);
    assert_eq!(result.next_after_space_id, Some(501));
    assert_eq!(result.outcomes.len(), 2);
    assert_eq!(
        result.outcomes[0].disposition,
        WikiPublicationBackfillDisposition::Initialized
    );
    assert_eq!(
        result.outcomes[1].disposition,
        WikiPublicationBackfillDisposition::Failed
    );
    assert_eq!(persistence.provision_count(), 2);
}

#[tokio::test]
async fn dry_run_reports_without_mutating_publication_or_drive() {
    let persistence = MemoryWikiPersistence::default();
    let drive_scope = FixedDriveScope::default();
    let initializer =
        KnowledgeWikiInitializationService::new(&persistence, &persistence, &drive_scope);
    let candidates = FixedBackfillStore {
        candidates: vec![candidate(501), candidate(502)],
    };
    let service = KnowledgeWikiBackfillService::new(&candidates, &persistence, &initializer);

    let result = service
        .run_page(RunWikiPublicationBackfillRequest {
            scope: SCOPE,
            after_space_id: None,
            page_size: 10,
            actor_id: 9001,
            dry_run: true,
        })
        .await
        .expect("run dry backfill page");

    assert!(!result.stopped_on_failure);
    assert_eq!(result.outcomes.len(), 2);
    assert!(result
        .outcomes
        .iter()
        .all(|outcome| outcome.disposition == WikiPublicationBackfillDisposition::Planned));
    assert_eq!(persistence.provision_count(), 0);
}

#[derive(Default)]
struct MemoryWikiPersistence {
    publications: Mutex<HashMap<u64, WikiPublication>>,
    provision_count: Mutex<u32>,
}

impl MemoryWikiPersistence {
    fn with_publication(value: WikiPublication) -> Self {
        Self {
            publications: Mutex::new(HashMap::from([(value.space_id, value)])),
            provision_count: Mutex::new(0),
        }
    }

    fn provision_count(&self) -> u32 {
        *self.provision_count.lock().unwrap()
    }
}

#[async_trait]
impl WikiPublicationStore for MemoryWikiPersistence {
    async fn provision_publication(
        &self,
        request: ProvisionWikiPublicationRequest,
    ) -> Result<WikiPublicationProvisioningResult, WikiPersistenceError> {
        *self.provision_count.lock().unwrap() += 1;
        let mut publications = self.publications.lock().unwrap();
        let created = !publications.contains_key(&request.space_id);
        let value = publications
            .entry(request.space_id)
            .or_insert_with(|| publication(request.space_id, &request.drive_space_uuid))
            .clone();
        Ok(WikiPublicationProvisioningResult {
            publication: value,
            created,
        })
    }

    async fn get_publication(
        &self,
        _scope: WikiPersistenceScope,
        site_publication_id: u64,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        self.publications
            .lock()
            .unwrap()
            .values()
            .find(|publication| publication.id == site_publication_id)
            .cloned()
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_publication",
                id: site_publication_id,
            })
    }

    async fn get_publication_for_space(
        &self,
        _scope: WikiPersistenceScope,
        space_id: u64,
    ) -> Result<Option<WikiPublication>, WikiPersistenceError> {
        Ok(self.publications.lock().unwrap().get(&space_id).cloned())
    }

    async fn bind_source_scope(
        &self,
        request: BindWikiSourceScopeRequest,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        let mut publications = self.publications.lock().unwrap();
        let value = publications
            .values_mut()
            .find(|publication| publication.id == request.site_publication_id)
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_publication",
                id: request.site_publication_id,
            })?;
        value.source_root_node_uuid = Some(request.source_root_node_uuid);
        value.source_scope_uuid = Some(request.source_scope_uuid);
        value.wiki_status = WikiPublicationStatus::Validating;
        value.version += 1;
        Ok(value.clone())
    }

    async fn mark_publication_ready(
        &self,
        request: MarkWikiPublicationReadyRequest,
    ) -> Result<WikiPublication, WikiPersistenceError> {
        let mut publications = self.publications.lock().unwrap();
        let value = publications
            .values_mut()
            .find(|publication| publication.id == request.site_publication_id)
            .ok_or(WikiPersistenceError::NotFound {
                resource: "wiki_publication",
                id: request.site_publication_id,
            })?;
        assert_eq!(value.version, request.expected_version);
        value.wiki_status = WikiPublicationStatus::Ready;
        value.version += 1;
        Ok(value.clone())
    }
}

#[async_trait]
impl WikiDriveCheckpointStore for MemoryWikiPersistence {
    async fn provision_checkpoint(
        &self,
        request: ProvisionWikiDriveCheckpointRequest,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        Ok(checkpoint(request))
    }

    async fn get_checkpoint(
        &self,
        _scope: WikiPersistenceScope,
        _checkpoint_id: u64,
    ) -> Result<WikiDriveCheckpoint, WikiPersistenceError> {
        unimplemented!()
    }

    async fn find_checkpoint_by_drive_scope(
        &self,
        _scope: WikiPersistenceScope,
        _drive_space_uuid: &str,
        _source_scope_uuid: &str,
    ) -> Result<Option<WikiDriveCheckpoint>, WikiPersistenceError> {
        Ok(None)
    }

    async fn list_checkpoints(
        &self,
        _request: ListWikiDriveCheckpointsRequest,
    ) -> Result<WikiDriveCheckpointPage, WikiPersistenceError> {
        Ok(WikiDriveCheckpointPage {
            checkpoints: Vec::new(),
            next_after_checkpoint_id: None,
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

#[derive(Default)]
struct FixedDriveScope {
    fail_for_drive: Option<String>,
}

#[async_trait]
impl KnowledgeWikiDriveScope for FixedDriveScope {
    async fn ensure_raw_scope(
        &self,
        request: EnsureKnowledgebaseRawScopeRequest,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        let knowledgebase_uuid =
            if self.fail_for_drive.as_deref() == Some(request.drive_space_id.as_str()) {
                "wrong-knowledgebase".to_string()
            } else {
                request.knowledgebase_uuid
            };
        Ok(KnowledgebaseRawScope {
            subscription_uuid: format!("scope-{}", request.drive_space_id),
            drive_space_id: request.drive_space_id.clone(),
            consumer_kind: "knowledgebase_raw".to_string(),
            knowledgebase_uuid,
            raw_folder_node_id: format!("raw-node-{}", request.drive_space_id),
            scope_status: "ACTIVE".to_string(),
            version: "0".to_string(),
            created_at: "2026-07-21T00:00:00Z".to_string(),
            updated_at: "2026-07-21T00:00:00Z".to_string(),
        })
    }

    async fn retrieve_raw_scope(
        &self,
        _subscription_uuid: &str,
    ) -> Result<KnowledgebaseRawScope, KnowledgeWikiDriveSourceError> {
        unimplemented!()
    }

    async fn renew_raw_scope_event_delivery(
        &self,
        request: RenewKnowledgebaseRawScopeEventDeliveryRequest,
    ) -> Result<KnowledgebaseRawScopeEventDelivery, KnowledgeWikiDriveSourceError> {
        Ok(KnowledgebaseRawScopeEventDelivery {
            channel_id: format!("kbraw:{}", request.subscription_uuid),
            subscription_uuid: request.subscription_uuid,
            expiration_epoch_ms: Some(4_102_444_800_000),
            mode: KnowledgeWikiDriveEventDeliveryMode::CloudWebhook,
        })
    }
}

struct FixedBackfillStore {
    candidates: Vec<WikiPublicationBackfillCandidate>,
}

#[async_trait]
impl WikiPublicationBackfillStore for FixedBackfillStore {
    async fn list_backfill_candidates(
        &self,
        _request: ListWikiPublicationBackfillCandidatesRequest,
    ) -> Result<WikiPublicationBackfillCandidatePage, WikiPersistenceError> {
        Ok(WikiPublicationBackfillCandidatePage {
            candidates: self.candidates.clone(),
            next_after_space_id: None,
        })
    }
}

fn publication(space_id: u64, drive_space_uuid: &str) -> WikiPublication {
    WikiPublication {
        id: 10_000 + space_id,
        uuid: format!("publication-{space_id}"),
        scope: SCOPE,
        space_id,
        drive_space_uuid: drive_space_uuid.to_string(),
        source_root_node_uuid: None,
        source_scope_uuid: None,
        wiki_status: WikiPublicationStatus::Draft,
        title: format!("Knowledgebase {space_id}"),
        homepage_source_path: "index.md".to_string(),
        publication_mode: WikiPublicationMode::ReviewRequired,
        default_visibility: WikiVisibility::Private,
        update_policy: WikiUpdatePolicy::KeepLastPublicUntilReady,
        provider_generation: 1,
        navigation_generation: 1,
        search_generation: 1,
        last_projected_drive_checkpoint: 0,
        version: 0,
    }
}

fn checkpoint(request: ProvisionWikiDriveCheckpointRequest) -> WikiDriveCheckpoint {
    WikiDriveCheckpoint {
        id: 20_000 + request.site_publication_id,
        uuid: format!("checkpoint-{}", request.site_publication_id),
        scope: request.scope,
        site_publication_id: request.site_publication_id,
        drive_space_uuid: request.drive_space_uuid,
        source_scope_uuid: request.source_scope_uuid,
        last_sequence_no: 0,
        last_event_id: None,
        stream_state: WikiDriveStreamState::Healthy,
        gap_from_sequence_no: None,
        gap_to_sequence_no: None,
        reconciliation_cursor: None,
        lease_token: None,
        fence_token: 0,
        version: 0,
    }
}

fn candidate(space_id: u64) -> WikiPublicationBackfillCandidate {
    WikiPublicationBackfillCandidate {
        space_id,
        knowledgebase_uuid: format!("knowledgebase-{space_id}"),
        title: format!("Knowledgebase {space_id}"),
        drive_space_uuid: format!("drive-{space_id}"),
        publication_missing: true,
        source_scope_missing: true,
        checkpoint_missing: true,
    }
}
