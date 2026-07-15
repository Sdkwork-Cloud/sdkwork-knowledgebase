use std::sync::Arc;

use sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::{
    group_knowledge_space_lifecycle_service_server::GroupKnowledgeSpaceLifecycleService,
    ArchiveGroupKnowledgeSpaceRequest, ArchiveGroupKnowledgeSpaceResponse,
    EnsureGroupKnowledgeSpaceRequest, EnsureGroupKnowledgeSpaceResponse,
    SynchronizeGroupKnowledgeSpaceMembersRequest, SynchronizeGroupKnowledgeSpaceMembersResponse,
};
use tonic::{Request, Response, Status};

use crate::{
    context::require_group_knowledge_space_lifecycle_caller,
    error::map_group_knowledge_space_service_error,
    mapper::{
        archive_request_from_proto, ensure_request_from_proto, lifecycle_from_binding,
        response_metadata, synchronize_members_request_from_proto,
    },
    runtime::GroupKnowledgeSpaceLifecycleRuntime,
};

/// Generated-service implementation for the IM-only lifecycle contract.
#[derive(Clone)]
pub struct GroupKnowledgeSpaceLifecycleRpcService {
    runtime: Arc<dyn GroupKnowledgeSpaceLifecycleRuntime>,
}

impl GroupKnowledgeSpaceLifecycleRpcService {
    pub fn new(runtime: Arc<dyn GroupKnowledgeSpaceLifecycleRuntime>) -> Self {
        Self { runtime }
    }
}

#[tonic::async_trait]
impl GroupKnowledgeSpaceLifecycleService for GroupKnowledgeSpaceLifecycleRpcService {
    async fn ensure_group_knowledge_space(
        &self,
        request: Request<EnsureGroupKnowledgeSpaceRequest>,
    ) -> Result<Response<EnsureGroupKnowledgeSpaceResponse>, Status> {
        let caller = require_group_knowledge_space_lifecycle_caller(&request)?;
        let command = ensure_request_from_proto(request.into_inner(), &caller)?;
        let operation = self
            .runtime
            .ensure_group_knowledge_space(caller.scope, &caller.actor_id, command)
            .await
            .map_err(map_group_knowledge_space_service_error)?;
        let lifecycle = lifecycle_from_binding(&operation.binding)?;
        Ok(Response::new(EnsureGroupKnowledgeSpaceResponse {
            lifecycle: Some(lifecycle),
            metadata: Some(response_metadata(&caller)),
        }))
    }

    async fn synchronize_group_knowledge_space_members(
        &self,
        request: Request<SynchronizeGroupKnowledgeSpaceMembersRequest>,
    ) -> Result<Response<SynchronizeGroupKnowledgeSpaceMembersResponse>, Status> {
        let caller = require_group_knowledge_space_lifecycle_caller(&request)?;
        let command = synchronize_members_request_from_proto(request.into_inner(), &caller)?;
        let change = self
            .runtime
            .synchronize_group_knowledge_space_members(caller.scope, &caller.actor_id, command)
            .await
            .map_err(map_group_knowledge_space_service_error)?;
        let lifecycle = lifecycle_from_binding(&change.binding)?;
        Ok(Response::new(
            SynchronizeGroupKnowledgeSpaceMembersResponse {
                lifecycle: Some(lifecycle),
                metadata: Some(response_metadata(&caller)),
            },
        ))
    }

    async fn archive_group_knowledge_space(
        &self,
        request: Request<ArchiveGroupKnowledgeSpaceRequest>,
    ) -> Result<Response<ArchiveGroupKnowledgeSpaceResponse>, Status> {
        let caller = require_group_knowledge_space_lifecycle_caller(&request)?;
        let command = archive_request_from_proto(request.into_inner(), &caller)?;
        let binding = self
            .runtime
            .archive_group_knowledge_space(
                caller.scope,
                &caller.actor_id,
                &command.archived_by,
                command.request,
            )
            .await
            .map_err(map_group_knowledge_space_service_error)?;
        let lifecycle = lifecycle_from_binding(&binding)?;
        Ok(Response::new(ArchiveGroupKnowledgeSpaceResponse {
            lifecycle: Some(lifecycle),
            metadata: Some(response_metadata(&caller)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    };

    use async_trait::async_trait;
    use sdkwork_intelligence_knowledgebase_service::{
        group_space::{GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError},
        ports::knowledge_group_space_binding_store::{
            GroupKnowledgeSpaceMembershipChange, GroupKnowledgeSpaceScope,
        },
    };
    use sdkwork_knowledgebase_contract::group_space::{
        ArchiveGroupKnowledgeSpaceRequest as DomainArchiveRequest,
        EnsureGroupKnowledgeSpaceRequest as DomainEnsureRequest,
        GroupKnowledgeSpaceAclProjectionState, GroupKnowledgeSpaceBinding,
        GroupKnowledgeSpaceLifecycleState,
        SynchronizeGroupKnowledgeSpaceMembersRequest as DomainSyncRequest,
    };
    use sdkwork_rpc_framework_core::{
        RpcCallerActorKind, VerifiedRpcCallerContext, VerifiedRpcServiceIdentity,
    };

    use super::*;

    #[derive(Default)]
    struct FakeRuntime {
        ensure_actor: Mutex<Option<String>>,
        synchronize_dispatched: AtomicBool,
    }

    #[async_trait]
    impl GroupKnowledgeSpaceLifecycleRuntime for FakeRuntime {
        async fn ensure_group_knowledge_space(
            &self,
            _scope: GroupKnowledgeSpaceScope,
            actor_id: &str,
            _request: DomainEnsureRequest,
        ) -> Result<GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError>
        {
            *self.ensure_actor.lock().expect("ensure actor mutex") = Some(actor_id.to_string());
            Ok(GroupKnowledgeSpaceOperation {
                binding: binding(GroupKnowledgeSpaceLifecycleState::Active),
                space: None,
            })
        }

        async fn synchronize_group_knowledge_space_members(
            &self,
            _scope: GroupKnowledgeSpaceScope,
            _actor_id: &str,
            _request: DomainSyncRequest,
        ) -> Result<GroupKnowledgeSpaceMembershipChange, KnowledgeGroupKnowledgeSpaceServiceError>
        {
            self.synchronize_dispatched.store(true, Ordering::SeqCst);
            Ok(GroupKnowledgeSpaceMembershipChange {
                binding: binding(GroupKnowledgeSpaceLifecycleState::Active),
                previous_members: Vec::new(),
                current_members: Vec::new(),
                requires_acl_projection: false,
            })
        }

        async fn archive_group_knowledge_space(
            &self,
            _scope: GroupKnowledgeSpaceScope,
            _actor_id: &str,
            _archived_by: &str,
            _request: DomainArchiveRequest,
        ) -> Result<GroupKnowledgeSpaceBinding, KnowledgeGroupKnowledgeSpaceServiceError> {
            unreachable!("archive is not used by this adapter smoke test")
        }
    }

    fn binding(lifecycle_state: GroupKnowledgeSpaceLifecycleState) -> GroupKnowledgeSpaceBinding {
        GroupKnowledgeSpaceBinding {
            id: 11,
            uuid: "binding-uuid".to_string(),
            tenant_id: 1,
            organization_id: 2,
            conversation_id: "conversation-1".to_string(),
            space_id: Some(12),
            space_uuid: Some("space-uuid".to_string()),
            group_name: "Group".to_string(),
            lifecycle_state,
            acl_projection_state: GroupKnowledgeSpaceAclProjectionState::Active,
            provisioning_idempotency_key_sha256_hex: String::new(),
            membership_epoch: 3,
            version: 1,
            upstream_link_generation: 4,
            archive_source_event_id: None,
            archive_payload_sha256_hex: None,
            archive_lease_token: None,
            archive_lease_until: None,
            archive_acl_cursor: None,
            archive_acl_pages_processed: 0,
            archive_acl_cleanup_completed_at: None,
            last_source_event_id: None,
            last_error_code: None,
            created_by: "sdkwork-im".to_string(),
            updated_by: "sdkwork-im".to_string(),
            created_at: "2026-07-14T00:00:00Z".to_string(),
            updated_at: "2026-07-14T00:00:00Z".to_string(),
            archived_at: None,
            archived_by: None,
            deleted_at: None,
        }
    }

    fn verified_request<T>(message: T) -> Request<T> {
        let correlation = format!("gkb-{}", sdkwork_utils_rust::sha256_hash(b"event-1"));
        let mut request = Request::new(message);
        request.extensions_mut().insert(VerifiedRpcServiceIdentity {
            service_id: "sdkwork-im".to_string(),
            trust_domain: "sdkwork.internal".to_string(),
            spiffe_uri: "spiffe://sdkwork.internal/sdkwork/service/sdkwork-im".to_string(),
            certificate_sha256: "a".repeat(64),
        });
        request.extensions_mut().insert(VerifiedRpcCallerContext {
            issuer_service_id: "sdkwork-im".to_string(),
            audience_service_id: "sdkwork-knowledgebase".to_string(),
            tenant_id: "1".to_string(),
            organization_id: "2".to_string(),
            actor_id: "sdkwork-im".to_string(),
            actor_kind: RpcCallerActorKind::Service,
            session_id: None,
            request_id: correlation.clone(),
            trace_id: Some(correlation.clone()),
            idempotency_key: Some(correlation),
            issued_at_unix_seconds: 1,
            expires_at_unix_seconds: 2,
            nonce: "nonce-1".to_string(),
        });
        request
    }

    fn synchronize_request() -> SynchronizeGroupKnowledgeSpaceMembersRequest {
        SynchronizeGroupKnowledgeSpaceMembersRequest {
            conversation_id: "conversation-1".to_string(),
            group_name: "Group".to_string(),
            source_event_id: "event-1".to_string(),
            knowledgebase_binding_id: "11".to_string(),
            knowledgebase_binding_uuid: "binding-uuid".to_string(),
            knowledge_space_id: "12".to_string(),
            knowledge_space_uuid: "space-uuid".to_string(),
            membership_epoch: "3".to_string(),
            upstream_link_generation: "4".to_string(),
            members: vec![
                sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::GroupKnowledgeSpaceMember {
                    actor_id: "owner-1".to_string(),
                    role: sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::GroupKnowledgeSpaceMemberRole::Owner as i32,
                },
            ],
            metadata: Some(
                sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::common::v1::RequestMetadata {
                    trace_id: format!(
                        "gkb-{}",
                        sdkwork_utils_rust::sha256_hash(b"event-1")
                    ),
                    traceparent: String::new(),
                    idempotency_key: format!(
                        "gkb-{}",
                        sdkwork_utils_rust::sha256_hash(b"event-1")
                    ),
                    request_hash: String::new(),
                    client_version: "sdkwork-im".to_string(),
                },
            ),
        }
    }

    #[tokio::test]
    async fn unary_ensure_smoke_uses_verified_context_and_returns_an_active_target() {
        let runtime = Arc::new(FakeRuntime::default());
        let service = GroupKnowledgeSpaceLifecycleRpcService::new(runtime.clone());
        let request = EnsureGroupKnowledgeSpaceRequest {
            conversation_id: "conversation-1".to_string(),
            group_name: "Group".to_string(),
            source_event_id: "event-1".to_string(),
            provisioning_idempotency_key: "idem-1".to_string(),
            membership_epoch: "3".to_string(),
            members: vec![
                sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::GroupKnowledgeSpaceMember {
                    actor_id: "owner-1".to_string(),
                    role: sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::GroupKnowledgeSpaceMemberRole::Owner as i32,
                },
            ],
            metadata: Some(
                sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::common::v1::RequestMetadata {
                    trace_id: format!(
                        "gkb-{}",
                        sdkwork_utils_rust::sha256_hash(b"event-1")
                    ),
                    traceparent: String::new(),
                    idempotency_key: format!(
                        "gkb-{}",
                        sdkwork_utils_rust::sha256_hash(b"event-1")
                    ),
                    request_hash: String::new(),
                    client_version: "sdkwork-im".to_string(),
                },
            ),
        };

        let response = service
            .ensure_group_knowledge_space(verified_request(request))
            .await
            .expect("verified unary ensure");
        let lifecycle = response.into_inner().lifecycle.expect("lifecycle response");
        assert_eq!(lifecycle.knowledgebase_binding_id, "11");
        assert_eq!(lifecycle.knowledge_space_id, "12");
        assert_eq!(
            lifecycle.lifecycle_state,
            sdkwork_knowledgebase_rpc_sdk_rust::sdkwork::intelligence::internal::v1::GroupKnowledgeSpaceLifecycleState::Active as i32
        );
        assert_eq!(
            runtime
                .ensure_actor
                .lock()
                .expect("ensure actor mutex")
                .as_deref(),
            Some("sdkwork-im")
        );
    }

    #[tokio::test]
    async fn unary_ensure_rejects_an_unverified_request_before_runtime_dispatch() {
        let runtime = Arc::new(FakeRuntime::default());
        let service = GroupKnowledgeSpaceLifecycleRpcService::new(runtime);
        let error = service
            .ensure_group_knowledge_space(Request::new(EnsureGroupKnowledgeSpaceRequest {
                conversation_id: "conversation-1".to_string(),
                group_name: "Group".to_string(),
                source_event_id: "event-1".to_string(),
                provisioning_idempotency_key: "idem-1".to_string(),
                membership_epoch: "0".to_string(),
                members: Vec::new(),
                metadata: None,
            }))
            .await
            .expect_err("missing framework extensions must fail closed");
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn unary_synchronize_rejects_a_user_context_before_runtime_dispatch() {
        let runtime = Arc::new(FakeRuntime::default());
        let service = GroupKnowledgeSpaceLifecycleRpcService::new(runtime.clone());
        let mut request = verified_request(synchronize_request());
        let caller = request
            .extensions_mut()
            .get_mut::<VerifiedRpcCallerContext>()
            .expect("verified caller context");
        caller.actor_kind = RpcCallerActorKind::User;
        caller.actor_id = "owner-1".to_string();
        caller.session_id = Some("session-1".to_string());

        let error = service
            .synchronize_group_knowledge_space_members(request)
            .await
            .expect_err("user context must not invoke the IM lifecycle runtime");
        assert_eq!(error.code(), tonic::Code::Unauthenticated);
        assert!(
            !runtime.synchronize_dispatched.load(Ordering::SeqCst),
            "untrusted callers must be rejected before runtime dispatch"
        );
    }
}
