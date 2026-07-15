/**
 * IM is the sole caller of this service. Tenant, organization, service identity, request id,
 * trace id, and idempotency are asserted by the internal RPC framework through mTLS and a signed
 * service caller context. They must never be supplied in these business payloads.
 *
 * @generated from service sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService
 */
export declare const GroupKnowledgeSpaceLifecycleService: {
    readonly typeName: "sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService";
    readonly methods: {
        /**
         * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.EnsureGroupKnowledgeSpace
         */
        readonly ensureGroupKnowledgeSpace: {
            readonly name: "EnsureGroupKnowledgeSpace";
            readonly I: any;
            readonly O: any;
            readonly kind: any;
        };
        /**
         * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.SynchronizeGroupKnowledgeSpaceMembers
         */
        readonly synchronizeGroupKnowledgeSpaceMembers: {
            readonly name: "SynchronizeGroupKnowledgeSpaceMembers";
            readonly I: any;
            readonly O: any;
            readonly kind: any;
        };
        /**
         * @generated from rpc sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService.ArchiveGroupKnowledgeSpace
         */
        readonly archiveGroupKnowledgeSpace: {
            readonly name: "ArchiveGroupKnowledgeSpace";
            readonly I: any;
            readonly O: any;
            readonly kind: any;
        };
    };
};
