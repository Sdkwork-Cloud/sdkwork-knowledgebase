use crate::serde_int64::{
    deserialize_nonnegative_i64_as_u64_from_string_or_number,
    deserialize_option_positive_i64_as_u64_from_string_or_number,
    deserialize_positive_i64_as_u64_from_string_or_number,
    serialize_option_u64_as_string, serialize_u64_as_string,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Exact opaque format issued by IM for a one-time group Knowledgebase launch.
///
/// The format deliberately excludes URL delimiters and whitespace so the ticket can travel only
/// through a URL fragment or a native deep-link path segment without altering its meaning.
pub const GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PREFIX: &str = "gklt_";
pub const GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PAYLOAD_LENGTH: usize = 43;
pub const GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_LENGTH: usize = GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PREFIX
    .len()
    + GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PAYLOAD_LENGTH;

/// Shared IM <-> Knowledgebase group lifecycle limits. These identifiers are opaque and must
/// remain byte-for-byte stable across the IM outbox, internal RPC, and Drive ACL projection.
pub const GROUP_KNOWLEDGE_SPACE_CONVERSATION_ID_MAX_LENGTH: usize = 256;
pub const GROUP_KNOWLEDGE_SPACE_GROUP_NAME_MAX_LENGTH: usize = 256;
pub const GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH: usize = 256;
pub const GROUP_KNOWLEDGE_SPACE_SOURCE_EVENT_ID_MAX_LENGTH: usize = 512;
pub const GROUP_KNOWLEDGE_SPACE_BINDING_UUID_MAX_LENGTH: usize = 64;
pub const GROUP_KNOWLEDGE_SPACE_SPACE_UUID_MAX_LENGTH: usize = 64;

/// Returns whether an opaque IM group-launch ticket has the canonical wire shape.
///
/// This validates only its non-secret syntax. IM remains the authority for signature, expiry,
/// single-use replay protection, and caller binding.
pub fn is_valid_group_knowledgebase_launch_ticket(ticket: &str) -> bool {
    let ticket_bytes = ticket.as_bytes();
    ticket_bytes.len() == GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_LENGTH
        && ticket_bytes.starts_with(GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PREFIX.as_bytes())
        && ticket_bytes[GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_PREFIX.len()..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_'))
}

/// The authoritative, Knowledgebase-owned association between one IM conversation and one space.
///
/// This is intentionally separate from `KnowledgeSpaceContextBinding`: chat-group ownership has
/// lifecycle, replay protection, and member-epoch requirements that generic contexts do not have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupKnowledgeSpaceBinding {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub id: u64,
    pub uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub organization_id: u64,
    pub conversation_id: String,
    #[serde(
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_positive_i64_as_u64_from_string_or_number"
    )]
    pub space_id: Option<u64>,
    pub space_uuid: Option<String>,
    pub group_name: String,
    pub lifecycle_state: GroupKnowledgeSpaceLifecycleState,
    pub acl_projection_state: GroupKnowledgeSpaceAclProjectionState,
    /// SHA-256 only. The raw idempotency key is never persisted or returned.
    #[serde(skip_serializing, skip_deserializing, default)]
    pub provisioning_idempotency_key_sha256_hex: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub membership_epoch: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub version: u64,
    /// Last accepted IM link generation. It is deliberately independent from this binding's
    /// local optimistic-concurrency version and is never exposed as a public KB version.
    #[serde(skip_serializing, skip_deserializing, default)]
    pub upstream_link_generation: u64,
    /// Durable archive-saga state. It is internal-only and must never be used as caller input.
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_source_event_id: Option<String>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_payload_sha256_hex: Option<String>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_lease_token: Option<String>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_lease_until: Option<String>,
    /// Durable cursor/checkpoint for bounded direct-Drive ACL revocation. Internal only.
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_acl_cursor: Option<String>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_acl_pages_processed: u64,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archive_acl_cleanup_completed_at: Option<String>,
    pub last_source_event_id: Option<String>,
    pub last_error_code: Option<String>,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub archived_by: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgeSpaceLifecycleState {
    Provisioning,
    Active,
    Failed,
    /// A durable, fail-closed archive intent. External ACL cleanup and physical-space archival
    /// can be retried until the binding reaches the terminal `archived` state.
    Archiving,
    Archived,
    Deleted,
}

impl GroupKnowledgeSpaceLifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provisioning => "provisioning",
            Self::Active => "active",
            Self::Failed => "failed",
            Self::Archiving => "archiving",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub fn accepts_membership_updates(self) -> bool {
        matches!(self, Self::Provisioning | Self::Active)
    }
}

impl FromStr for GroupKnowledgeSpaceLifecycleState {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "provisioning" => Ok(Self::Provisioning),
            "active" => Ok(Self::Active),
            "failed" => Ok(Self::Failed),
            "archiving" => Ok(Self::Archiving),
            "archived" => Ok(Self::Archived),
            "deleted" => Ok(Self::Deleted),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgeSpaceAclProjectionState {
    Pending,
    Active,
    Failed,
}

impl GroupKnowledgeSpaceAclProjectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for GroupKnowledgeSpaceAclProjectionState {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "active" => Ok(Self::Active),
            "failed" => Ok(Self::Failed),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgeSpaceMemberRole {
    Owner,
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgeSpacePrincipalKind {
    User,
}

impl GroupKnowledgeSpacePrincipalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
        }
    }
}

impl FromStr for GroupKnowledgeSpacePrincipalKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "user" => Ok(Self::User),
            _ => Err(()),
        }
    }
}

impl GroupKnowledgeSpaceMemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }

    pub fn access_level(self) -> Option<GroupKnowledgeSpaceAccessLevel> {
        match self {
            Self::Owner => Some(GroupKnowledgeSpaceAccessLevel::Owner),
            Self::Admin => Some(GroupKnowledgeSpaceAccessLevel::Writer),
            Self::Member => Some(GroupKnowledgeSpaceAccessLevel::Reader),
            // Guests are intentionally not given an implicit knowledgebase grant.
            Self::Guest => None,
        }
    }
}

impl FromStr for GroupKnowledgeSpaceMemberRole {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            "guest" => Ok(Self::Guest),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgeSpaceAccessLevel {
    Reader,
    Writer,
    Owner,
}

impl GroupKnowledgeSpaceAccessLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reader => "reader",
            Self::Writer => "writer",
            Self::Owner => "owner",
        }
    }
}

impl FromStr for GroupKnowledgeSpaceAccessLevel {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "reader" => Ok(Self::Reader),
            "writer" => Ok(Self::Writer),
            "owner" => Ok(Self::Owner),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupKnowledgeSpaceMember {
    pub principal_kind: GroupKnowledgeSpacePrincipalKind,
    pub actor_id: String,
    /// This is the only role supplied by IM. `writer` and `reader` are deliberately not valid
    /// group-member roles; Knowledgebase derives its local access level from this IM role.
    pub role: GroupKnowledgeSpaceMemberRole,
    /// Persistence/projection detail derived locally from `role`. It must never be accepted from
    /// an IM caller, otherwise a caller could attempt to turn a `guest` or `member` into a more
    /// privileged Drive grant.
    #[serde(default, skip_serializing, skip_deserializing)]
    pub access_level: Option<GroupKnowledgeSpaceAccessLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsureGroupKnowledgeSpaceRequest {
    pub conversation_id: String,
    pub group_name: String,
    pub source_event_id: String,
    /// A caller-provided retry identity. It is stored only as a SHA-256 digest by Knowledgebase.
    pub provisioning_idempotency_key: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub membership_epoch: u64,
    pub members: Vec<GroupKnowledgeSpaceMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizeGroupKnowledgeSpaceMembersRequest {
    pub conversation_id: String,
    pub group_name: String,
    pub source_event_id: String,
    /// Immutable Knowledgebase target fence. It is issued by `ensure` and prevents an old IM
    /// event from being applied to another binding for the same conversation.
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub knowledgebase_binding_id: u64,
    pub knowledgebase_binding_uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub knowledge_space_id: u64,
    pub knowledge_space_uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub membership_epoch: u64,
    /// IM link generation, intentionally distinct from `GroupKnowledgeSpaceBinding.version`.
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub upstream_link_generation: u64,
    pub members: Vec<GroupKnowledgeSpaceMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGroupKnowledgeSpaceRequest {
    pub conversation_id: String,
    pub source_event_id: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub membership_epoch: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub knowledgebase_binding_id: u64,
    pub knowledgebase_binding_uuid: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub knowledge_space_id: u64,
    pub knowledge_space_uuid: String,
    /// IM link generation, intentionally distinct from `GroupKnowledgeSpaceBinding.version`.
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_nonnegative_i64_as_u64_from_string_or_number"
    )]
    pub upstream_link_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ConsumeGroupKnowledgebaseLaunchTicketRequest {
    /// Opaque, one-time ticket carried from a browser fragment or desktop deep link after login.
    /// It is intentionally the only caller-controlled field in this command.
    pub ticket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupKnowledgebaseLaunchTarget {
    pub conversation_id: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_positive_i64_as_u64_from_string_or_number"
    )]
    pub space_id: u64,
    pub space_uuid: String,
    pub group_name: String,
    pub lifecycle_state: GroupKnowledgeSpaceLifecycleState,
}

/// Reports whether this Knowledgebase runtime can consume IM-issued group launch tickets.
/// It deliberately contains no endpoint, certificate, credential, or integration diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKnowledgebaseLaunchCapabilityState {
    Configured,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupKnowledgebaseLaunchCapability {
    pub state: GroupKnowledgebaseLaunchCapabilityState,
}

#[cfg(test)]
mod tests {
    use super::{
        is_valid_group_knowledgebase_launch_ticket, ArchiveGroupKnowledgeSpaceRequest,
        GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_LENGTH,
    };
    use crate::{
        parse_canonical_nonnegative_signed_i64, parse_canonical_positive_signed_i64,
    };
    use serde_json::json;

    fn valid_ticket() -> String {
        format!("gklt_{}", "a".repeat(43))
    }

    #[test]
    fn accepts_only_the_exact_opaque_group_launch_ticket_shape() {
        let ticket = valid_ticket();

        assert_eq!(ticket.len(), GROUP_KNOWLEDGEBASE_LAUNCH_TICKET_LENGTH);
        assert!(is_valid_group_knowledgebase_launch_ticket(ticket.as_str()));
    }

    #[test]
    fn rejects_malformed_group_launch_tickets_before_im_consumption() {
        let mut ticket_with_delimiter = valid_ticket();
        ticket_with_delimiter.replace_range(5..6, "/");

        for ticket in [
            String::new(),
            "gklt_".to_string(),
            format!("gklt_{}", "a".repeat(42)),
            format!("gklt_{}", "a".repeat(44)),
            ticket_with_delimiter,
            format!("gklt_{}", " ".repeat(43)),
            format!("GKLT_{}", "a".repeat(43)),
        ] {
            assert!(
                !is_valid_group_knowledgebase_launch_ticket(ticket.as_str()),
                "unexpectedly accepted malformed ticket"
            );
        }
    }

    #[test]
    fn group_lifecycle_resource_ids_reject_unsigned_bigint_overflow() {
        let boundary = i64::MAX.to_string();
        let overflow = (i64::MAX as u64 + 1).to_string();
        let valid: ArchiveGroupKnowledgeSpaceRequest = serde_json::from_value(json!({
            "conversationId": "conversation-1",
            "sourceEventId": "archive-1",
            "membershipEpoch": boundary.clone(),
            "knowledgebaseBindingId": boundary.clone(),
            "knowledgebaseBindingUuid": "binding-uuid",
            "knowledgeSpaceId": boundary,
            "knowledgeSpaceUuid": "space-uuid",
            "upstreamLinkGeneration": "0"
        }))
        .expect("signed BIGINT boundary is valid");
        assert_eq!(valid.knowledgebase_binding_id, i64::MAX as u64);

        for field in [
            "membershipEpoch",
            "knowledgebaseBindingId",
            "knowledgeSpaceId",
            "upstreamLinkGeneration",
        ] {
            let mut request = json!({
                "conversationId": "conversation-1",
                "sourceEventId": "archive-1",
                "membershipEpoch": "1",
                "knowledgebaseBindingId": "1",
                "knowledgebaseBindingUuid": "binding-uuid",
                "knowledgeSpaceId": "1",
                "knowledgeSpaceUuid": "space-uuid",
                "upstreamLinkGeneration": "0"
            });
            request[field] = json!(overflow.clone());
            assert!(
                serde_json::from_value::<ArchiveGroupKnowledgeSpaceRequest>(request).is_err(),
                "{field} must reject values above signed BIGINT"
            );
        }
    }

    #[test]
    fn group_lifecycle_scope_ids_require_canonical_signed_bigint_text() {
        for invalid in ["", "0", "01", "+1", " 1", "1 ", "-1", "9223372036854775808"] {
            assert!(
                parse_canonical_positive_signed_i64(invalid).is_err(),
                "{invalid:?} must not be accepted as a canonical positive scope id"
            );
        }
        assert_eq!(
            parse_canonical_positive_signed_i64("9223372036854775807"),
            Ok(i64::MAX as u64)
        );
        assert_eq!(parse_canonical_nonnegative_signed_i64("0"), Ok(0));
        assert!(parse_canonical_nonnegative_signed_i64("00").is_err());
    }
}
