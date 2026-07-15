use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::{
    group_space::{GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError},
    ports::knowledge_group_space_binding_store::{
        GroupKnowledgeSpaceMembershipChange, GroupKnowledgeSpaceScope,
    },
};
use sdkwork_knowledgebase_contract::group_space::{
    ArchiveGroupKnowledgeSpaceRequest, EnsureGroupKnowledgeSpaceRequest,
    GroupKnowledgeSpaceBinding, SynchronizeGroupKnowledgeSpaceMembersRequest,
};

/// Runtime port consumed by the generated gRPC adapter.
///
/// The RPC crate does not know how Knowledgebase storage or Drive adapters are assembled. A
/// process host injects this port and is solely responsible for concrete dependency wiring.
#[async_trait]
pub trait GroupKnowledgeSpaceLifecycleRuntime: Send + Sync {
    async fn ensure_group_knowledge_space(
        &self,
        scope: GroupKnowledgeSpaceScope,
        actor_id: &str,
        request: EnsureGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError>;

    async fn synchronize_group_knowledge_space_members(
        &self,
        scope: GroupKnowledgeSpaceScope,
        actor_id: &str,
        request: SynchronizeGroupKnowledgeSpaceMembersRequest,
    ) -> Result<GroupKnowledgeSpaceMembershipChange, KnowledgeGroupKnowledgeSpaceServiceError>;

    async fn archive_group_knowledge_space(
        &self,
        scope: GroupKnowledgeSpaceScope,
        service_actor_id: &str,
        archived_by: &str,
        request: ArchiveGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceBinding, KnowledgeGroupKnowledgeSpaceServiceError>;
}
