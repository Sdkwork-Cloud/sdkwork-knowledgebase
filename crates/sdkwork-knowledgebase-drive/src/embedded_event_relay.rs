use async_trait::async_trait;
use sdkwork_drive_workspace_service::ports::domain_outbox_embedded_relay::{
    DeliverDriveDomainOutboxEmbeddedEventRequest, DriveDomainOutboxEmbeddedRelay,
    DriveDomainOutboxEmbeddedRelayError, DriveDomainOutboxEmbeddedTarget,
    ResolveDriveDomainOutboxEmbeddedTargetsRequest,
};
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_wiki_drive_source::KnowledgeWikiDriveSource,
        knowledge_wiki_persistence::{
            WikiDriveCheckpointStore, WikiDriveEventInboxStore, WikiPublicationStore,
        },
    },
    wiki_event_consumer::{
        resolve_knowledge_wiki_drive_trusted_event_targets, KnowledgeWikiDriveEventConsumerService,
        ReceiveKnowledgeWikiDriveTrustedEventRequest,
    },
};
use std::sync::Arc;

const EMBEDDED_KNOWLEDGEBASE_RAW_CHANNEL_PREFIX: &str = "embedded:kbraw:";

pub fn embedded_knowledgebase_raw_channel_id(source_scope_uuid: &str) -> String {
    format!("{EMBEDDED_KNOWLEDGEBASE_RAW_CHANNEL_PREFIX}{source_scope_uuid}")
}

#[derive(Clone)]
pub struct KnowledgebaseDriveEmbeddedEventRelay {
    publication_store: Arc<dyn WikiPublicationStore>,
    checkpoint_store: Arc<dyn WikiDriveCheckpointStore>,
    inbox_store: Arc<dyn WikiDriveEventInboxStore>,
    drive_source: Arc<dyn KnowledgeWikiDriveSource>,
}

impl KnowledgebaseDriveEmbeddedEventRelay {
    pub fn new(
        publication_store: Arc<dyn WikiPublicationStore>,
        checkpoint_store: Arc<dyn WikiDriveCheckpointStore>,
        inbox_store: Arc<dyn WikiDriveEventInboxStore>,
        drive_source: Arc<dyn KnowledgeWikiDriveSource>,
    ) -> Self {
        Self {
            publication_store,
            checkpoint_store,
            inbox_store,
            drive_source,
        }
    }

    fn consumer(&self) -> KnowledgeWikiDriveEventConsumerService<'_> {
        KnowledgeWikiDriveEventConsumerService::new(
            self.publication_store.as_ref(),
            self.checkpoint_store.as_ref(),
            self.inbox_store.as_ref(),
            self.drive_source.as_ref(),
        )
    }
}

#[async_trait]
impl DriveDomainOutboxEmbeddedRelay for KnowledgebaseDriveEmbeddedEventRelay {
    async fn resolve_targets(
        &self,
        request: ResolveDriveDomainOutboxEmbeddedTargetsRequest<'_>,
    ) -> Result<Vec<DriveDomainOutboxEmbeddedTarget>, DriveDomainOutboxEmbeddedRelayError> {
        let targets = resolve_knowledge_wiki_drive_trusted_event_targets(request.payload_json)
            .map_err(|error| {
                DriveDomainOutboxEmbeddedRelayError::InvalidEvent(error.code().to_string())
            })?;
        if targets.scope.tenant_id.to_string() != request.tenant_id
            || targets.drive_space_uuid != request.space_id
        {
            return Err(DriveDomainOutboxEmbeddedRelayError::InvalidEvent(
                "event envelope does not match its Drive outbox authority".to_string(),
            ));
        }
        Ok(targets
            .source_scope_uuids
            .into_iter()
            .map(|source_scope_uuid| DriveDomainOutboxEmbeddedTarget {
                channel_id: embedded_knowledgebase_raw_channel_id(&source_scope_uuid),
                source_scope_uuid,
            })
            .collect())
    }

    async fn deliver(
        &self,
        request: DeliverDriveDomainOutboxEmbeddedEventRequest<'_>,
    ) -> Result<(), DriveDomainOutboxEmbeddedRelayError> {
        if request.channel_id != embedded_knowledgebase_raw_channel_id(request.source_scope_uuid) {
            return Err(DriveDomainOutboxEmbeddedRelayError::InvalidEvent(
                "embedded channel does not match its Knowledgebase raw scope".to_string(),
            ));
        }
        let targets = resolve_knowledge_wiki_drive_trusted_event_targets(request.payload_json)
            .map_err(|error| {
                DriveDomainOutboxEmbeddedRelayError::InvalidEvent(error.code().to_string())
            })?;
        if targets.scope.tenant_id.to_string() != request.tenant_id
            || targets.drive_space_uuid != request.space_id
            || !targets
                .source_scope_uuids
                .iter()
                .any(|scope| scope == request.source_scope_uuid)
        {
            return Err(DriveDomainOutboxEmbeddedRelayError::InvalidEvent(
                "embedded delivery target is not authorized by the Drive event".to_string(),
            ));
        }
        self.consumer()
            .receive_trusted(ReceiveKnowledgeWikiDriveTrustedEventRequest {
                scope: targets.scope,
                source_scope_uuid: request.source_scope_uuid.to_string(),
                payload_json: request.payload_json.to_string(),
            })
            .await
            .map_err(|error| {
                DriveDomainOutboxEmbeddedRelayError::Delivery(error.code().to_string())
            })?;
        Ok(())
    }
}
