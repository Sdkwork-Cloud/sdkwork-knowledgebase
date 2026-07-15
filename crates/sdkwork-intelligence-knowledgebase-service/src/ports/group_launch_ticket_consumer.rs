use async_trait::async_trait;
use sdkwork_knowledgebase_contract::group_space::GroupKnowledgeSpaceMemberRole;
use sdkwork_knowledgebase_contract::group_space::GroupKnowledgeSpacePrincipalKind;
use thiserror::Error;

/// Caller context is translated by the injected generated IM RPC adapter into signed forwarding
/// metadata. It is never serialized into the opaque ticket-consumption request body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupLaunchTicketCallerContext {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub principal_kind: GroupKnowledgeSpacePrincipalKind,
    pub actor_id: String,
    pub session_id: Option<String>,
    /// Server-owned correlation values propagated from the framework-authenticated HTTP context.
    /// They are signed by the outbound RPC adapter and never accepted from ticket payload fields.
    pub request_id: String,
    /// A nonblank trace identifier produced by the server-side web framework.
    pub trace_id: String,
    /// The nonblank, framework-normalized idempotency key for this exact ticket consumption.
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumeGroupLaunchTicketCommand {
    pub ticket: String,
    pub caller: GroupLaunchTicketCallerContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedGroupLaunchTicket {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub principal_kind: GroupKnowledgeSpacePrincipalKind,
    pub actor_id: String,
    pub conversation_id: String,
    pub knowledgebase_binding_id: u64,
    pub knowledgebase_binding_uuid: String,
    pub space_id: u64,
    pub space_uuid: String,
    pub membership_epoch: u64,
    /// IM-owned link generation. This is a ticket freshness fence, not a Knowledgebase
    /// optimistic-concurrency version.
    pub upstream_link_generation: u64,
    pub membership_role: GroupKnowledgeSpaceMemberRole,
    pub expires_at: String,
}

#[async_trait]
pub trait GroupLaunchTicketConsumer: Send + Sync {
    async fn consume_group_launch_ticket(
        &self,
        command: ConsumeGroupLaunchTicketCommand,
    ) -> Result<ConsumedGroupLaunchTicket, GroupLaunchTicketConsumerError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GroupLaunchTicketConsumerError {
    #[error("group launch ticket is invalid or expired")]
    InvalidOrExpired,
    #[error("group launch ticket was already consumed")]
    Replayed,
    #[error("group launch ticket caller is not authorized")]
    Unauthorized,
    #[error("IM launch ticket consumer is unavailable")]
    Unavailable,
    #[error("IM launch ticket consumer failed: {0}")]
    Upstream(String),
}
