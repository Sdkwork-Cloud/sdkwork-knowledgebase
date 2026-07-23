use crate::ports::{
    knowledge_wiki_drive_source::{
        KnowledgeWikiDriveEventDeliveryMode, KnowledgeWikiDriveScope,
        RenewKnowledgebaseRawScopeEventDeliveryRequest,
    },
    knowledge_wiki_persistence::{
        ListWikiDriveCheckpointsRequest, WikiDriveCheckpoint, WikiDriveCheckpointPage,
        WikiDriveCheckpointStore, WikiPersistenceError, WikiPersistenceScope,
    },
};
use sdkwork_utils_rust::is_blank;

pub const MAX_EVENT_DELIVERY_RENEWAL_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenewWikiDriveEventDeliveryPageRequest {
    pub scope: WikiPersistenceScope,
    pub after_checkpoint_id: Option<u64>,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiDriveEventDeliveryRenewalFailure {
    pub checkpoint_id: u64,
    pub source_scope_uuid: String,
    pub error_code: String,
    pub error_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiDriveEventDeliveryRenewalPageResult {
    pub checkpoints_scanned: usize,
    pub cloud_deliveries_renewed: usize,
    pub embedded_relays_verified: usize,
    pub failures: Vec<WikiDriveEventDeliveryRenewalFailure>,
    pub next_after_checkpoint_id: Option<u64>,
}

pub struct KnowledgeWikiDriveEventDeliveryRenewalService<'a> {
    checkpoint_store: &'a dyn WikiDriveCheckpointStore,
    drive_scope: &'a dyn KnowledgeWikiDriveScope,
}

impl<'a> KnowledgeWikiDriveEventDeliveryRenewalService<'a> {
    pub fn new(
        checkpoint_store: &'a dyn WikiDriveCheckpointStore,
        drive_scope: &'a dyn KnowledgeWikiDriveScope,
    ) -> Self {
        Self {
            checkpoint_store,
            drive_scope,
        }
    }

    pub async fn renew_page(
        &self,
        request: RenewWikiDriveEventDeliveryPageRequest,
    ) -> Result<WikiDriveEventDeliveryRenewalPageResult, WikiPersistenceError> {
        validate_request(&request)?;
        let page = self
            .checkpoint_store
            .list_checkpoints(ListWikiDriveCheckpointsRequest {
                scope: request.scope,
                after_checkpoint_id: request.after_checkpoint_id,
                limit: request.limit,
            })
            .await?;
        renew_checkpoints(self.drive_scope, page).await
    }
}

async fn renew_checkpoints(
    drive_scope: &dyn KnowledgeWikiDriveScope,
    page: WikiDriveCheckpointPage,
) -> Result<WikiDriveEventDeliveryRenewalPageResult, WikiPersistenceError> {
    let checkpoints_scanned = page.checkpoints.len();
    let mut result = WikiDriveEventDeliveryRenewalPageResult {
        checkpoints_scanned,
        cloud_deliveries_renewed: 0,
        embedded_relays_verified: 0,
        failures: Vec::new(),
        next_after_checkpoint_id: page.next_after_checkpoint_id,
    };

    for checkpoint in page.checkpoints {
        match renew_checkpoint(drive_scope, &checkpoint).await {
            Ok(KnowledgeWikiDriveEventDeliveryMode::CloudWebhook) => {
                result.cloud_deliveries_renewed += 1;
            }
            Ok(KnowledgeWikiDriveEventDeliveryMode::EmbeddedTrustedRelay) => {
                result.embedded_relays_verified += 1;
            }
            Err(error) => result.failures.push(WikiDriveEventDeliveryRenewalFailure {
                checkpoint_id: checkpoint.id,
                source_scope_uuid: checkpoint.source_scope_uuid,
                error_code: error.code().to_string(),
                error_summary: error.to_string(),
            }),
        }
    }
    Ok(result)
}

async fn renew_checkpoint(
    drive_scope: &dyn KnowledgeWikiDriveScope,
    checkpoint: &WikiDriveCheckpoint,
) -> Result<
    KnowledgeWikiDriveEventDeliveryMode,
    crate::ports::knowledge_wiki_drive_source::KnowledgeWikiDriveSourceError,
> {
    let delivery = drive_scope
        .renew_raw_scope_event_delivery(RenewKnowledgebaseRawScopeEventDeliveryRequest {
            subscription_uuid: checkpoint.source_scope_uuid.clone(),
        })
        .await?;
    if delivery.subscription_uuid != checkpoint.source_scope_uuid
        || is_blank(Some(&delivery.channel_id))
    {
        return Err(crate::ports::knowledge_wiki_drive_source::KnowledgeWikiDriveSourceError::IntegrityFailed(
            "Drive event delivery renewal response does not match the checkpoint source scope".to_string(),
        ));
    }
    Ok(delivery.mode)
}

fn validate_request(
    request: &RenewWikiDriveEventDeliveryPageRequest,
) -> Result<(), WikiPersistenceError> {
    if request.scope.tenant_id == 0
        || request.limit == 0
        || request.limit > MAX_EVENT_DELIVERY_RENEWAL_PAGE_SIZE
    {
        return Err(WikiPersistenceError::InvalidRequest(
            "event delivery renewal scope and page limit are invalid".to_string(),
        ));
    }
    Ok(())
}
