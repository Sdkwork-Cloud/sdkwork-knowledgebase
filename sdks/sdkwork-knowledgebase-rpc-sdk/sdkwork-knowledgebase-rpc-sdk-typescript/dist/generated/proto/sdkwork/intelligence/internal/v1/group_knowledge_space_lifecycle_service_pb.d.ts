import type { GenEnum, GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import type { RequestMetadata, ResponseMetadata } from "../../../common/v1/context_pb";
import type { Message } from "@bufbuild/protobuf";
/**
 * Describes the file sdkwork/intelligence/internal/v1/group_knowledge_space_lifecycle_service.proto.
 */
export declare const file_sdkwork_intelligence_internal_v1_group_knowledge_space_lifecycle_service: GenFile;
/**
 * @generated from message sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMember
 */
export type GroupKnowledgeSpaceMember = Message<"sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMember"> & {
    /**
     * @generated from field: string actor_id = 1;
     */
    actorId: string;
    /**
     * @generated from field: sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMemberRole role = 2;
     */
    role: GroupKnowledgeSpaceMemberRole;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMember.
 * Use `create(GroupKnowledgeSpaceMemberSchema)` to create a new message.
 */
export declare const GroupKnowledgeSpaceMemberSchema: GenMessage<GroupKnowledgeSpaceMember>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceRequest
 */
export type EnsureGroupKnowledgeSpaceRequest = Message<"sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceRequest"> & {
    /**
     * @generated from field: string conversation_id = 1;
     */
    conversationId: string;
    /**
     * @generated from field: string group_name = 2;
     */
    groupName: string;
    /**
     * @generated from field: string source_event_id = 3;
     */
    sourceEventId: string;
    /**
     * Retained only as a SHA-256 digest by Knowledgebase.
     *
     * @generated from field: string provisioning_idempotency_key = 4;
     */
    provisioningIdempotencyKey: string;
    /**
     * Decimal signed-BIGINT text preserves precision across generated clients.
     *
     * @generated from field: string membership_epoch = 5;
     */
    membershipEpoch: string;
    /**
     * @generated from field: repeated sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMember members = 6;
     */
    members: GroupKnowledgeSpaceMember[];
    /**
     * @generated from field: sdkwork.common.v1.RequestMetadata metadata = 15;
     */
    metadata?: RequestMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceRequest.
 * Use `create(EnsureGroupKnowledgeSpaceRequestSchema)` to create a new message.
 */
export declare const EnsureGroupKnowledgeSpaceRequestSchema: GenMessage<EnsureGroupKnowledgeSpaceRequest>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersRequest
 */
export type SynchronizeGroupKnowledgeSpaceMembersRequest = Message<"sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersRequest"> & {
    /**
     * @generated from field: string conversation_id = 1;
     */
    conversationId: string;
    /**
     * @generated from field: string group_name = 2;
     */
    groupName: string;
    /**
     * @generated from field: string source_event_id = 3;
     */
    sourceEventId: string;
    /**
     * @generated from field: string knowledgebase_binding_id = 4;
     */
    knowledgebaseBindingId: string;
    /**
     * @generated from field: string knowledgebase_binding_uuid = 5;
     */
    knowledgebaseBindingUuid: string;
    /**
     * @generated from field: string knowledge_space_id = 6;
     */
    knowledgeSpaceId: string;
    /**
     * @generated from field: string knowledge_space_uuid = 7;
     */
    knowledgeSpaceUuid: string;
    /**
     * @generated from field: string membership_epoch = 8;
     */
    membershipEpoch: string;
    /**
     * IM-owned link generation, deliberately distinct from Knowledgebase binding.version.
     *
     * @generated from field: string upstream_link_generation = 9;
     */
    upstreamLinkGeneration: string;
    /**
     * @generated from field: repeated sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMember members = 10;
     */
    members: GroupKnowledgeSpaceMember[];
    /**
     * @generated from field: sdkwork.common.v1.RequestMetadata metadata = 15;
     */
    metadata?: RequestMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersRequest.
 * Use `create(SynchronizeGroupKnowledgeSpaceMembersRequestSchema)` to create a new message.
 */
export declare const SynchronizeGroupKnowledgeSpaceMembersRequestSchema: GenMessage<SynchronizeGroupKnowledgeSpaceMembersRequest>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceRequest
 */
export type ArchiveGroupKnowledgeSpaceRequest = Message<"sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceRequest"> & {
    /**
     * @generated from field: string conversation_id = 1;
     */
    conversationId: string;
    /**
     * @generated from field: string source_event_id = 2;
     */
    sourceEventId: string;
    /**
     * @generated from field: string knowledgebase_binding_id = 3;
     */
    knowledgebaseBindingId: string;
    /**
     * @generated from field: string knowledgebase_binding_uuid = 4;
     */
    knowledgebaseBindingUuid: string;
    /**
     * @generated from field: string knowledge_space_id = 5;
     */
    knowledgeSpaceId: string;
    /**
     * @generated from field: string knowledge_space_uuid = 6;
     */
    knowledgeSpaceUuid: string;
    /**
     * @generated from field: string membership_epoch = 7;
     */
    membershipEpoch: string;
    /**
     * @generated from field: string upstream_link_generation = 8;
     */
    upstreamLinkGeneration: string;
    /**
     * IM-provided audit attribution only. It has no authorization effect in Knowledgebase.
     *
     * @generated from field: string archived_by = 9;
     */
    archivedBy: string;
    /**
     * @generated from field: sdkwork.common.v1.RequestMetadata metadata = 15;
     */
    metadata?: RequestMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceRequest.
 * Use `create(ArchiveGroupKnowledgeSpaceRequestSchema)` to create a new message.
 */
export declare const ArchiveGroupKnowledgeSpaceRequestSchema: GenMessage<ArchiveGroupKnowledgeSpaceRequest>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle
 */
export type GroupKnowledgeSpaceLifecycle = Message<"sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle"> & {
    /**
     * @generated from field: string knowledgebase_binding_id = 1;
     */
    knowledgebaseBindingId: string;
    /**
     * @generated from field: string knowledgebase_binding_uuid = 2;
     */
    knowledgebaseBindingUuid: string;
    /**
     * @generated from field: string knowledge_space_id = 3;
     */
    knowledgeSpaceId: string;
    /**
     * @generated from field: string knowledge_space_uuid = 4;
     */
    knowledgeSpaceUuid: string;
    /**
     * @generated from field: sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleState lifecycle_state = 5;
     */
    lifecycleState: GroupKnowledgeSpaceLifecycleState;
    /**
     * @generated from field: string membership_epoch = 6;
     */
    membershipEpoch: string;
    /**
     * @generated from field: string upstream_link_generation = 7;
     */
    upstreamLinkGeneration: string;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle.
 * Use `create(GroupKnowledgeSpaceLifecycleSchema)` to create a new message.
 */
export declare const GroupKnowledgeSpaceLifecycleSchema: GenMessage<GroupKnowledgeSpaceLifecycle>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceResponse
 */
export type EnsureGroupKnowledgeSpaceResponse = Message<"sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceResponse"> & {
    /**
     * @generated from field: sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle lifecycle = 1;
     */
    lifecycle?: GroupKnowledgeSpaceLifecycle | undefined;
    /**
     * @generated from field: sdkwork.common.v1.ResponseMetadata metadata = 15;
     */
    metadata?: ResponseMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.EnsureGroupKnowledgeSpaceResponse.
 * Use `create(EnsureGroupKnowledgeSpaceResponseSchema)` to create a new message.
 */
export declare const EnsureGroupKnowledgeSpaceResponseSchema: GenMessage<EnsureGroupKnowledgeSpaceResponse>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersResponse
 */
export type SynchronizeGroupKnowledgeSpaceMembersResponse = Message<"sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersResponse"> & {
    /**
     * @generated from field: sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle lifecycle = 1;
     */
    lifecycle?: GroupKnowledgeSpaceLifecycle | undefined;
    /**
     * @generated from field: sdkwork.common.v1.ResponseMetadata metadata = 15;
     */
    metadata?: ResponseMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.SynchronizeGroupKnowledgeSpaceMembersResponse.
 * Use `create(SynchronizeGroupKnowledgeSpaceMembersResponseSchema)` to create a new message.
 */
export declare const SynchronizeGroupKnowledgeSpaceMembersResponseSchema: GenMessage<SynchronizeGroupKnowledgeSpaceMembersResponse>;
/**
 * @generated from message sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceResponse
 */
export type ArchiveGroupKnowledgeSpaceResponse = Message<"sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceResponse"> & {
    /**
     * @generated from field: sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycle lifecycle = 1;
     */
    lifecycle?: GroupKnowledgeSpaceLifecycle | undefined;
    /**
     * @generated from field: sdkwork.common.v1.ResponseMetadata metadata = 15;
     */
    metadata?: ResponseMetadata | undefined;
};
/**
 * Describes the message sdkwork.intelligence.internal.v1.ArchiveGroupKnowledgeSpaceResponse.
 * Use `create(ArchiveGroupKnowledgeSpaceResponseSchema)` to create a new message.
 */
export declare const ArchiveGroupKnowledgeSpaceResponseSchema: GenMessage<ArchiveGroupKnowledgeSpaceResponse>;
/**
 * @generated from enum sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMemberRole
 */
export declare enum GroupKnowledgeSpaceMemberRole {
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_MEMBER_ROLE_UNSPECIFIED = 0;
     */
    UNSPECIFIED = 0,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_MEMBER_ROLE_OWNER = 1;
     */
    OWNER = 1,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_MEMBER_ROLE_ADMIN = 2;
     */
    ADMIN = 2,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_MEMBER_ROLE_MEMBER = 3;
     */
    MEMBER = 3,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_MEMBER_ROLE_GUEST = 4;
     */
    GUEST = 4
}
/**
 * Describes the enum sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceMemberRole.
 */
export declare const GroupKnowledgeSpaceMemberRoleSchema: GenEnum<GroupKnowledgeSpaceMemberRole>;
/**
 * @generated from enum sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleState
 */
export declare enum GroupKnowledgeSpaceLifecycleState {
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_UNSPECIFIED = 0;
     */
    UNSPECIFIED = 0,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_PROVISIONING = 1;
     */
    PROVISIONING = 1,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_ACTIVE = 2;
     */
    ACTIVE = 2,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_FAILED = 3;
     */
    FAILED = 3,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_ARCHIVING = 4;
     */
    ARCHIVING = 4,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_ARCHIVED = 5;
     */
    ARCHIVED = 5,
    /**
     * @generated from enum value: GROUP_KNOWLEDGE_SPACE_LIFECYCLE_STATE_DELETED = 6;
     */
    DELETED = 6
}
/**
 * Describes the enum sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleState.
 */
export declare const GroupKnowledgeSpaceLifecycleStateSchema: GenEnum<GroupKnowledgeSpaceLifecycleState>;
/**
 * IM is the sole caller of this service. Tenant, organization, service identity, request id,
 * trace id, and idempotency are asserted by the internal RPC framework through mTLS and a signed
 * service caller context. They must never be supplied in these business payloads.
 *
 * @generated from service sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService
 */
export declare const GroupKnowledgeSpaceLifecycleService: GenService<{
    /**
     * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.EnsureGroupKnowledgeSpace
     */
    ensureGroupKnowledgeSpace: {
        methodKind: "unary";
        input: typeof EnsureGroupKnowledgeSpaceRequestSchema;
        output: typeof EnsureGroupKnowledgeSpaceResponseSchema;
    };
    /**
     * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.SynchronizeGroupKnowledgeSpaceMembers
     */
    synchronizeGroupKnowledgeSpaceMembers: {
        methodKind: "unary";
        input: typeof SynchronizeGroupKnowledgeSpaceMembersRequestSchema;
        output: typeof SynchronizeGroupKnowledgeSpaceMembersResponseSchema;
    };
    /**
     * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.ArchiveGroupKnowledgeSpace
     */
    archiveGroupKnowledgeSpace: {
        methodKind: "unary";
        input: typeof ArchiveGroupKnowledgeSpaceRequestSchema;
        output: typeof ArchiveGroupKnowledgeSpaceResponseSchema;
    };
}>;
