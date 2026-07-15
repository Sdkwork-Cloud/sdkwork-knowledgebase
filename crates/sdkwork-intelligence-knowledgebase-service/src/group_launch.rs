use crate::{
    group_space_access::{
        GroupKnowledgeSpaceAccessAuthorizer, GroupKnowledgeSpaceAccessAuthorizerError,
    },
    ports::{
        group_launch_ticket_consumer::{
            ConsumeGroupLaunchTicketCommand, GroupLaunchTicketCallerContext,
            GroupLaunchTicketConsumer, GroupLaunchTicketConsumerError,
        },
        knowledge_access_control::KnowledgeAccessRole,
        knowledge_group_space_binding_store::{
            GroupKnowledgeSpaceScope, KnowledgeGroupSpaceBindingStore,
            KnowledgeGroupSpaceBindingStoreError,
        },
    },
};
use sdkwork_knowledgebase_contract::group_space::{
    is_valid_group_knowledgebase_launch_ticket, ConsumeGroupKnowledgebaseLaunchTicketRequest,
    GroupKnowledgeSpaceLifecycleState, GroupKnowledgeSpaceMemberRole,
    GroupKnowledgebaseLaunchTarget,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

/// Resolves an opaque, one-time IM launch ticket to exactly one synchronized group KB space.
/// There is intentionally no default/personal-space fallback on any validation failure.
pub struct GroupKnowledgebaseLaunchResolver<'a> {
    ticket_consumer: &'a dyn GroupLaunchTicketConsumer,
    binding_store: &'a dyn KnowledgeGroupSpaceBindingStore,
}

impl<'a> GroupKnowledgebaseLaunchResolver<'a> {
    pub fn new(
        ticket_consumer: &'a dyn GroupLaunchTicketConsumer,
        binding_store: &'a dyn KnowledgeGroupSpaceBindingStore,
    ) -> Self {
        Self {
            ticket_consumer,
            binding_store,
        }
    }

    pub async fn consume(
        &self,
        caller: GroupLaunchTicketCallerContext,
        request: ConsumeGroupKnowledgebaseLaunchTicketRequest,
    ) -> Result<GroupKnowledgebaseLaunchTarget, GroupKnowledgebaseLaunchResolverError> {
        if is_blank(Some(request.ticket.as_str())) {
            return Err(GroupKnowledgebaseLaunchResolverError::InvalidRequest(
                "group launch ticket is required".to_string(),
            ));
        }
        if !is_valid_group_knowledgebase_launch_ticket(request.ticket.as_str()) {
            return Err(GroupKnowledgebaseLaunchResolverError::InvalidRequest(
                "group launch ticket has an invalid format".to_string(),
            ));
        }
        let consumed = self
            .ticket_consumer
            .consume_group_launch_ticket(ConsumeGroupLaunchTicketCommand {
                ticket: request.ticket,
                caller: caller.clone(),
            })
            .await?;

        if consumed.tenant_id != caller.tenant_id
            || consumed.organization_id != caller.organization_id
            || consumed.principal_kind != caller.principal_kind
            || consumed.actor_id != caller.actor_id
        {
            return Err(GroupKnowledgebaseLaunchResolverError::Denied(
                "launch ticket identity does not match the authenticated session".to_string(),
            ));
        }
        if consumed.membership_role == GroupKnowledgeSpaceMemberRole::Guest {
            return Err(GroupKnowledgebaseLaunchResolverError::Denied(
                "group guests cannot launch the group knowledgebase".to_string(),
            ));
        }

        let scope = GroupKnowledgeSpaceScope {
            tenant_id: caller.tenant_id,
            organization_id: caller.organization_id,
        };
        let binding = self
            .binding_store
            .get_group_space(scope, &consumed.conversation_id)
            .await?;
        if binding.lifecycle_state != GroupKnowledgeSpaceLifecycleState::Active
            || binding.id != consumed.knowledgebase_binding_id
            || binding.uuid != consumed.knowledgebase_binding_uuid
            || binding.space_id != Some(consumed.space_id)
            || binding.space_uuid.as_deref() != Some(consumed.space_uuid.as_str())
            || binding.membership_epoch != consumed.membership_epoch
            || binding.upstream_link_generation != consumed.upstream_link_generation
        {
            return Err(GroupKnowledgebaseLaunchResolverError::Denied(
                "group launch ticket no longer resolves to the current knowledgebase binding"
                    .to_string(),
            ));
        }

        let authorizer = GroupKnowledgeSpaceAccessAuthorizer::new(self.binding_store);
        authorizer
            .authorize(
                scope,
                consumed.space_id,
                &caller.actor_id,
                KnowledgeAccessRole::Reader,
            )
            .await?;
        let members = self
            .binding_store
            .list_active_group_members(scope, binding.id)
            .await?;
        let current_role = members
            .into_iter()
            .find(|member| member.actor_id == caller.actor_id)
            .map(|member| member.role)
            .ok_or_else(|| {
                GroupKnowledgebaseLaunchResolverError::Denied(
                    "authenticated actor is no longer in the group knowledgebase snapshot"
                        .to_string(),
                )
            })?;
        if current_role == GroupKnowledgeSpaceMemberRole::Guest
            || current_role != consumed.membership_role
        {
            return Err(GroupKnowledgebaseLaunchResolverError::Denied(
                "group launch ticket membership role is stale".to_string(),
            ));
        }

        Ok(GroupKnowledgebaseLaunchTarget {
            conversation_id: binding.conversation_id,
            space_id: consumed.space_id,
            space_uuid: binding.space_uuid.ok_or_else(|| {
                GroupKnowledgebaseLaunchResolverError::InvalidBinding(
                    "active group binding has no space UUID".to_string(),
                )
            })?,
            group_name: binding.group_name,
            lifecycle_state: binding.lifecycle_state,
        })
    }
}

#[derive(Debug, Error)]
pub enum GroupKnowledgebaseLaunchResolverError {
    #[error("group launch request is invalid: {0}")]
    InvalidRequest(String),
    #[error("group launch ticket is denied: {0}")]
    Denied(String),
    #[error("group launch binding is invalid: {0}")]
    InvalidBinding(String),
    #[error(transparent)]
    Ticket(#[from] GroupLaunchTicketConsumerError),
    #[error(transparent)]
    Binding(#[from] KnowledgeGroupSpaceBindingStoreError),
    #[error(transparent)]
    Authorization(#[from] GroupKnowledgeSpaceAccessAuthorizerError),
}
