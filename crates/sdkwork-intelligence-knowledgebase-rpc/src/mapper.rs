use std::collections::BTreeSet;

use sdkwork_knowledgebase_contract::{
    group_space::{
        ArchiveGroupKnowledgeSpaceRequest, EnsureGroupKnowledgeSpaceRequest,
        GroupKnowledgeSpaceBinding, GroupKnowledgeSpaceLifecycleState, GroupKnowledgeSpaceMember,
        GroupKnowledgeSpaceMemberRole, GroupKnowledgeSpacePrincipalKind,
        SynchronizeGroupKnowledgeSpaceMembersRequest, GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH,
        GROUP_KNOWLEDGE_SPACE_BINDING_UUID_MAX_LENGTH,
        GROUP_KNOWLEDGE_SPACE_CONVERSATION_ID_MAX_LENGTH,
        GROUP_KNOWLEDGE_SPACE_GROUP_NAME_MAX_LENGTH,
        GROUP_KNOWLEDGE_SPACE_SOURCE_EVENT_ID_MAX_LENGTH,
        GROUP_KNOWLEDGE_SPACE_SPACE_UUID_MAX_LENGTH,
    },
    parse_canonical_nonnegative_signed_i64, parse_canonical_positive_signed_i64,
};
use sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::{
    common::v1::{RequestMetadata, ResponseMetadata},
    intelligence::internal::v1 as proto,
};
use sdkwork_utils_rust::is_blank;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tonic::Status;

use crate::context::GroupKnowledgeSpaceLifecycleCaller;

pub fn ensure_request_from_proto(
    request: proto::EnsureGroupKnowledgeSpaceRequest,
    caller: &GroupKnowledgeSpaceLifecycleCaller,
) -> Result<EnsureGroupKnowledgeSpaceRequest, Status> {
    validate_request_metadata(request.metadata.as_ref(), caller)?;
    validate_required_text(
        "conversation_id",
        &request.conversation_id,
        GROUP_KNOWLEDGE_SPACE_CONVERSATION_ID_MAX_LENGTH,
    )?;
    validate_required_text(
        "group_name",
        &request.group_name,
        GROUP_KNOWLEDGE_SPACE_GROUP_NAME_MAX_LENGTH,
    )?;
    validate_required_text(
        "source_event_id",
        &request.source_event_id,
        GROUP_KNOWLEDGE_SPACE_SOURCE_EVENT_ID_MAX_LENGTH,
    )?;
    validate_source_event_correlation(&request.source_event_id, caller)?;
    validate_required_text(
        "provisioning_idempotency_key",
        &request.provisioning_idempotency_key,
        512,
    )?;
    let membership_epoch = parse_nonnegative("membership_epoch", &request.membership_epoch)?;
    // IM has already authorized the originating group owner or administrator before writing its
    // durable outbox event. The service actor is intentionally `sdkwork-im`, never a user copied
    // into a background-delivery context. The Knowledgebase service validates the snapshot and
    // performs the explicit trusted-IM lifecycle transition.
    let members = group_members_from_proto(&request.members)?;

    Ok(EnsureGroupKnowledgeSpaceRequest {
        conversation_id: request.conversation_id,
        group_name: request.group_name,
        source_event_id: request.source_event_id,
        provisioning_idempotency_key: request.provisioning_idempotency_key,
        membership_epoch,
        members,
    })
}

pub fn synchronize_members_request_from_proto(
    request: proto::SynchronizeGroupKnowledgeSpaceMembersRequest,
    caller: &GroupKnowledgeSpaceLifecycleCaller,
) -> Result<SynchronizeGroupKnowledgeSpaceMembersRequest, Status> {
    validate_request_metadata(request.metadata.as_ref(), caller)?;
    validate_required_text(
        "conversation_id",
        &request.conversation_id,
        GROUP_KNOWLEDGE_SPACE_CONVERSATION_ID_MAX_LENGTH,
    )?;
    validate_required_text(
        "group_name",
        &request.group_name,
        GROUP_KNOWLEDGE_SPACE_GROUP_NAME_MAX_LENGTH,
    )?;
    validate_required_text(
        "source_event_id",
        &request.source_event_id,
        GROUP_KNOWLEDGE_SPACE_SOURCE_EVENT_ID_MAX_LENGTH,
    )?;
    validate_source_event_correlation(&request.source_event_id, caller)?;
    validate_required_text(
        "knowledgebase_binding_uuid",
        &request.knowledgebase_binding_uuid,
        GROUP_KNOWLEDGE_SPACE_BINDING_UUID_MAX_LENGTH,
    )?;
    validate_required_text(
        "knowledge_space_uuid",
        &request.knowledge_space_uuid,
        GROUP_KNOWLEDGE_SPACE_SPACE_UUID_MAX_LENGTH,
    )?;

    Ok(SynchronizeGroupKnowledgeSpaceMembersRequest {
        conversation_id: request.conversation_id,
        group_name: request.group_name,
        source_event_id: request.source_event_id,
        knowledgebase_binding_id: parse_positive(
            "knowledgebase_binding_id",
            &request.knowledgebase_binding_id,
        )?,
        knowledgebase_binding_uuid: request.knowledgebase_binding_uuid,
        knowledge_space_id: parse_positive("knowledge_space_id", &request.knowledge_space_id)?,
        knowledge_space_uuid: request.knowledge_space_uuid,
        membership_epoch: parse_nonnegative("membership_epoch", &request.membership_epoch)?,
        upstream_link_generation: parse_nonnegative(
            "upstream_link_generation",
            &request.upstream_link_generation,
        )?,
        members: group_members_from_proto(&request.members)?,
    })
}

#[derive(Debug)]
pub struct ArchiveGroupKnowledgeSpaceCommand {
    pub archived_by: String,
    pub request: ArchiveGroupKnowledgeSpaceRequest,
}

pub fn archive_request_from_proto(
    request: proto::ArchiveGroupKnowledgeSpaceRequest,
    caller: &GroupKnowledgeSpaceLifecycleCaller,
) -> Result<ArchiveGroupKnowledgeSpaceCommand, Status> {
    validate_request_metadata(request.metadata.as_ref(), caller)?;
    validate_required_text(
        "conversation_id",
        &request.conversation_id,
        GROUP_KNOWLEDGE_SPACE_CONVERSATION_ID_MAX_LENGTH,
    )?;
    validate_required_text(
        "source_event_id",
        &request.source_event_id,
        GROUP_KNOWLEDGE_SPACE_SOURCE_EVENT_ID_MAX_LENGTH,
    )?;
    validate_source_event_correlation(&request.source_event_id, caller)?;
    validate_required_text(
        "knowledgebase_binding_uuid",
        &request.knowledgebase_binding_uuid,
        GROUP_KNOWLEDGE_SPACE_BINDING_UUID_MAX_LENGTH,
    )?;
    validate_required_text(
        "knowledge_space_uuid",
        &request.knowledge_space_uuid,
        GROUP_KNOWLEDGE_SPACE_SPACE_UUID_MAX_LENGTH,
    )?;
    validate_required_text(
        "archived_by",
        &request.archived_by,
        GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH,
    )?;

    Ok(ArchiveGroupKnowledgeSpaceCommand {
        archived_by: request.archived_by,
        request: ArchiveGroupKnowledgeSpaceRequest {
            conversation_id: request.conversation_id,
            source_event_id: request.source_event_id,
            membership_epoch: parse_nonnegative("membership_epoch", &request.membership_epoch)?,
            knowledgebase_binding_id: parse_positive(
                "knowledgebase_binding_id",
                &request.knowledgebase_binding_id,
            )?,
            knowledgebase_binding_uuid: request.knowledgebase_binding_uuid,
            knowledge_space_id: parse_positive("knowledge_space_id", &request.knowledge_space_id)?,
            knowledge_space_uuid: request.knowledge_space_uuid,
            upstream_link_generation: parse_nonnegative(
                "upstream_link_generation",
                &request.upstream_link_generation,
            )?,
        },
    })
}

pub fn lifecycle_from_binding(
    binding: &GroupKnowledgeSpaceBinding,
) -> Result<proto::GroupKnowledgeSpaceLifecycle, Status> {
    let knowledge_space_id = binding.space_id.ok_or_else(|| {
        Status::failed_precondition("group knowledge-space binding has no knowledge space")
    })?;
    let knowledge_space_uuid = binding
        .space_uuid
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
        .ok_or_else(|| {
            Status::failed_precondition("group knowledge-space binding has no knowledge space")
        })?;

    Ok(proto::GroupKnowledgeSpaceLifecycle {
        knowledgebase_binding_id: binding.id.to_string(),
        knowledgebase_binding_uuid: binding.uuid.clone(),
        knowledge_space_id: knowledge_space_id.to_string(),
        knowledge_space_uuid: knowledge_space_uuid.to_string(),
        lifecycle_state: lifecycle_state_to_proto(binding.lifecycle_state) as i32,
        membership_epoch: binding.membership_epoch.to_string(),
        upstream_link_generation: binding.upstream_link_generation.to_string(),
    })
}

pub fn response_metadata(caller: &GroupKnowledgeSpaceLifecycleCaller) -> ResponseMetadata {
    ResponseMetadata {
        trace_id: caller
            .trace_id
            .clone()
            .unwrap_or_else(|| caller.request_id.clone()),
        traceparent: String::new(),
        server_time: OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_default(),
        warnings: Vec::new(),
        deprecation_notices: Vec::new(),
    }
}

fn validate_request_metadata(
    metadata: Option<&RequestMetadata>,
    caller: &GroupKnowledgeSpaceLifecycleCaller,
) -> Result<(), Status> {
    let metadata = metadata.ok_or_else(|| {
        Status::invalid_argument(
            "request metadata is required for internal group lifecycle commands",
        )
    })?;
    if !metadata.trace_id.is_empty()
        && caller.trace_id.as_deref() != Some(metadata.trace_id.as_str())
    {
        return Err(Status::invalid_argument(
            "request metadata trace_id must agree with the signed caller context",
        ));
    }
    if !metadata.idempotency_key.is_empty() && metadata.idempotency_key != caller.idempotency_key {
        return Err(Status::invalid_argument(
            "request metadata idempotency_key must agree with signed metadata",
        ));
    }
    for (field, value) in [
        ("traceparent", metadata.traceparent.as_str()),
        ("request_hash", metadata.request_hash.as_str()),
        ("client_version", metadata.client_version.as_str()),
    ] {
        validate_optional_metadata_text(field, value)?;
    }
    Ok(())
}

fn validate_source_event_correlation(
    source_event_id: &str,
    caller: &GroupKnowledgeSpaceLifecycleCaller,
) -> Result<(), Status> {
    let expected = format!(
        "gkb-{}",
        sdkwork_utils_rust::sha256_hash(source_event_id.as_bytes())
    );
    if caller.request_id != expected
        || caller.trace_id.as_deref() != Some(expected.as_str())
        || caller.idempotency_key != expected
    {
        return Err(Status::unauthenticated(
            "signed caller correlation does not match the source event",
        ));
    }
    Ok(())
}

fn group_members_from_proto(
    members: &[proto::GroupKnowledgeSpaceMember],
) -> Result<Vec<GroupKnowledgeSpaceMember>, Status> {
    if members.is_empty() {
        return Err(Status::invalid_argument(
            "at least one group member is required",
        ));
    }

    let mut actor_ids = BTreeSet::new();
    let mut owner_count = 0usize;
    let mut mapped = Vec::with_capacity(members.len());
    for member in members {
        validate_required_text(
            "member.actor_id",
            &member.actor_id,
            GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH,
        )?;
        if !actor_ids.insert(member.actor_id.as_str()) {
            return Err(Status::invalid_argument(
                "group member actor ids must be unique",
            ));
        }
        let role = proto_member_role_to_domain(member.role)?;
        if role == GroupKnowledgeSpaceMemberRole::Owner {
            owner_count += 1;
        }
        mapped.push(GroupKnowledgeSpaceMember {
            principal_kind: GroupKnowledgeSpacePrincipalKind::User,
            actor_id: member.actor_id.clone(),
            role,
            access_level: None,
        });
    }
    if owner_count != 1 {
        return Err(Status::invalid_argument(
            "a group membership snapshot must contain exactly one owner",
        ));
    }
    Ok(mapped)
}

fn proto_member_role_to_domain(role: i32) -> Result<GroupKnowledgeSpaceMemberRole, Status> {
    match proto::GroupKnowledgeSpaceMemberRole::try_from(role).ok() {
        Some(proto::GroupKnowledgeSpaceMemberRole::Owner) => {
            Ok(GroupKnowledgeSpaceMemberRole::Owner)
        }
        Some(proto::GroupKnowledgeSpaceMemberRole::Admin) => {
            Ok(GroupKnowledgeSpaceMemberRole::Admin)
        }
        Some(proto::GroupKnowledgeSpaceMemberRole::Member) => {
            Ok(GroupKnowledgeSpaceMemberRole::Member)
        }
        Some(proto::GroupKnowledgeSpaceMemberRole::Guest) => {
            Ok(GroupKnowledgeSpaceMemberRole::Guest)
        }
        Some(proto::GroupKnowledgeSpaceMemberRole::Unspecified) | None => {
            Err(Status::invalid_argument("group member role is required"))
        }
    }
}

fn lifecycle_state_to_proto(
    state: GroupKnowledgeSpaceLifecycleState,
) -> proto::GroupKnowledgeSpaceLifecycleState {
    match state {
        GroupKnowledgeSpaceLifecycleState::Provisioning => {
            proto::GroupKnowledgeSpaceLifecycleState::Provisioning
        }
        GroupKnowledgeSpaceLifecycleState::Active => {
            proto::GroupKnowledgeSpaceLifecycleState::Active
        }
        GroupKnowledgeSpaceLifecycleState::Failed => {
            proto::GroupKnowledgeSpaceLifecycleState::Failed
        }
        GroupKnowledgeSpaceLifecycleState::Archiving => {
            proto::GroupKnowledgeSpaceLifecycleState::Archiving
        }
        GroupKnowledgeSpaceLifecycleState::Archived => {
            proto::GroupKnowledgeSpaceLifecycleState::Archived
        }
        GroupKnowledgeSpaceLifecycleState::Deleted => {
            proto::GroupKnowledgeSpaceLifecycleState::Deleted
        }
    }
}

fn parse_positive(field: &str, value: &str) -> Result<u64, Status> {
    parse_canonical_positive_signed_i64(value).map_err(|_| {
        Status::invalid_argument(format!(
            "{field} must be a canonical positive signed BIGINT"
        ))
    })
}

fn parse_nonnegative(field: &str, value: &str) -> Result<u64, Status> {
    parse_canonical_nonnegative_signed_i64(value).map_err(|_| {
        Status::invalid_argument(format!(
            "{field} must be a canonical nonnegative signed BIGINT"
        ))
    })
}

fn validate_required_text(field: &str, value: &str, maximum_length: usize) -> Result<(), Status> {
    if is_blank(Some(value)) || value.len() > maximum_length || contains_control_character(value) {
        return Err(Status::invalid_argument(format!("{field} is invalid")));
    }
    Ok(())
}

fn validate_optional_metadata_text(field: &str, value: &str) -> Result<(), Status> {
    if value.len() > 512 || contains_control_character(value) {
        return Err(Status::invalid_argument(format!(
            "request metadata {field} is invalid"
        )));
    }
    Ok(())
}

fn contains_control_character(value: &str) -> bool {
    value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::GroupKnowledgeSpaceLifecycleCaller;
    use sdkwork_intelligence_knowledgebase_service::ports::knowledge_group_space_binding_store::GroupKnowledgeSpaceScope;

    fn caller() -> GroupKnowledgeSpaceLifecycleCaller {
        let correlation = format!("gkb-{}", sdkwork_utils_rust::sha256_hash(b"event-1"));
        GroupKnowledgeSpaceLifecycleCaller {
            scope: GroupKnowledgeSpaceScope {
                tenant_id: 1,
                organization_id: 2,
            },
            actor_id: "owner-1".to_string(),
            request_id: correlation.clone(),
            trace_id: Some(correlation.clone()),
            idempotency_key: correlation,
        }
    }

    fn owner() -> proto::GroupKnowledgeSpaceMember {
        proto::GroupKnowledgeSpaceMember {
            actor_id: "owner-1".to_string(),
            role: proto::GroupKnowledgeSpaceMemberRole::Owner as i32,
        }
    }

    fn request_metadata() -> RequestMetadata {
        let correlation = format!("gkb-{}", sdkwork_utils_rust::sha256_hash(b"event-1"));
        RequestMetadata {
            trace_id: correlation.clone(),
            traceparent: String::new(),
            idempotency_key: correlation,
            request_hash: String::new(),
            client_version: "sdkwork-im".to_string(),
        }
    }

    #[test]
    fn ensure_keeps_the_durable_provisioning_identity_separate_from_transport_correlation() {
        let request = proto::EnsureGroupKnowledgeSpaceRequest {
            conversation_id: "conversation-1".to_string(),
            group_name: "Group".to_string(),
            source_event_id: "event-1".to_string(),
            provisioning_idempotency_key: "different".to_string(),
            membership_epoch: "0".to_string(),
            members: vec![owner()],
            metadata: Some(request_metadata()),
        };
        assert_eq!(
            ensure_request_from_proto(request, &caller())
                .expect("durable provisioning key is distinct from transport correlation")
                .provisioning_idempotency_key,
            "different"
        );
    }

    #[test]
    fn lifecycle_requests_require_metadata_that_matches_the_signed_source_event_correlation() {
        let request = proto::EnsureGroupKnowledgeSpaceRequest {
            conversation_id: "conversation-1".to_string(),
            group_name: "Group".to_string(),
            source_event_id: "event-1".to_string(),
            provisioning_idempotency_key: "business-key".to_string(),
            membership_epoch: "0".to_string(),
            members: vec![owner()],
            metadata: None,
        };
        assert_eq!(
            ensure_request_from_proto(request, &caller())
                .expect_err("metadata is mandatory for internal commands")
                .code(),
            tonic::Code::InvalidArgument
        );
    }

    #[test]
    fn membership_mapping_rejects_duplicate_members_and_unsigned_integer_spellings() {
        let mut duplicate = owner();
        duplicate.role = proto::GroupKnowledgeSpaceMemberRole::Admin as i32;
        assert!(group_members_from_proto(&[owner(), duplicate]).is_err());
        assert!(parse_nonnegative("membership_epoch", "01").is_err());
        assert!(parse_positive("knowledge_space_id", "0").is_err());
    }

    #[test]
    fn archive_preserves_user_audit_attribution_without_using_it_as_service_authority() {
        let request = proto::ArchiveGroupKnowledgeSpaceRequest {
            conversation_id: "conversation-1".to_string(),
            source_event_id: "event-1".to_string(),
            knowledgebase_binding_id: "11".to_string(),
            knowledgebase_binding_uuid: "binding-uuid".to_string(),
            knowledge_space_id: "12".to_string(),
            knowledge_space_uuid: "space-uuid".to_string(),
            membership_epoch: "3".to_string(),
            upstream_link_generation: "4".to_string(),
            archived_by: "user-7".to_string(),
            metadata: Some(request_metadata()),
        };
        let command = archive_request_from_proto(request, &caller())
            .expect("user audit attribution is valid beside the IM service caller");
        assert_eq!(command.archived_by, "user-7");

        let invalid = proto::ArchiveGroupKnowledgeSpaceRequest {
            archived_by: String::new(),
            ..proto::ArchiveGroupKnowledgeSpaceRequest {
                conversation_id: "conversation-1".to_string(),
                source_event_id: "event-1".to_string(),
                knowledgebase_binding_id: "11".to_string(),
                knowledgebase_binding_uuid: "binding-uuid".to_string(),
                knowledge_space_id: "12".to_string(),
                knowledge_space_uuid: "space-uuid".to_string(),
                membership_epoch: "3".to_string(),
                upstream_link_generation: "4".to_string(),
                archived_by: "ignored".to_string(),
                metadata: Some(request_metadata()),
            }
        };
        assert_eq!(
            archive_request_from_proto(invalid, &caller())
                .expect_err("empty audit attribution must be rejected")
                .code(),
            tonic::Code::InvalidArgument
        );
    }
}
