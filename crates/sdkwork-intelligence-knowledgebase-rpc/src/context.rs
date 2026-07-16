use sdkwork_intelligence_knowledgebase_service::ports::knowledge_group_space_binding_store::GroupKnowledgeSpaceScope;
use sdkwork_knowledgebase_contract::{
    group_space::GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH, parse_canonical_nonnegative_signed_i64,
    parse_canonical_positive_signed_i64,
};
use sdkwork_rpc_framework_core::RpcCallerActorKind;
use sdkwork_rpc_server::{
    require_verified_rpc_caller_context, require_verified_rpc_service_identity,
};
use sdkwork_utils_rust::is_blank;
use tonic::{Request, Status};

pub const IM_SERVICE_ID: &str = "sdkwork-im";

/// Authority extracted only from framework-verified mTLS and signed caller context extensions.
/// No proto request field can replace or widen these values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupKnowledgeSpaceLifecycleCaller {
    pub scope: GroupKnowledgeSpaceScope,
    pub actor_id: String,
    pub request_id: String,
    pub trace_id: Option<String>,
    pub idempotency_key: String,
}

pub fn require_group_knowledge_space_lifecycle_caller<T>(
    request: &Request<T>,
) -> Result<GroupKnowledgeSpaceLifecycleCaller, Status> {
    let service_identity = require_verified_rpc_service_identity(request)?;
    if service_identity.service_id != IM_SERVICE_ID {
        return Err(Status::permission_denied(
            "internal group lifecycle caller is not permitted",
        ));
    }

    let caller_context = require_verified_rpc_caller_context(request)?;
    if caller_context.issuer_service_id != IM_SERVICE_ID
        || caller_context.audience_service_id != "sdkwork-knowledgebase"
        || caller_context.actor_kind != RpcCallerActorKind::Service
        || caller_context.actor_id != IM_SERVICE_ID
        || caller_context.session_id.is_some()
    {
        return Err(Status::unauthenticated(
            "internal group lifecycle caller context is invalid",
        ));
    }

    let tenant_id = parse_canonical_positive_signed_i64(&caller_context.tenant_id)
        .map_err(|_| Status::unauthenticated("internal group lifecycle scope is invalid"))?;
    let organization_id =
        parse_canonical_nonnegative_signed_i64(&caller_context.organization_id)
            .map_err(|_| Status::unauthenticated("internal group lifecycle scope is invalid"))?;
    if caller_context.actor_id.len() > GROUP_KNOWLEDGE_SPACE_ACTOR_ID_MAX_LENGTH
        || is_blank(Some(caller_context.request_id.as_str()))
    {
        return Err(Status::unauthenticated(
            "internal group lifecycle caller context is invalid",
        ));
    }

    let idempotency_key = caller_context
        .idempotency_key
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
        .ok_or_else(|| {
            Status::invalid_argument(
                "idempotency-key metadata is required for group lifecycle commands",
            )
        })?
        .to_string();

    Ok(GroupKnowledgeSpaceLifecycleCaller {
        scope: GroupKnowledgeSpaceScope {
            tenant_id,
            organization_id,
        },
        actor_id: caller_context.actor_id.clone(),
        request_id: caller_context.request_id.clone(),
        trace_id: caller_context.trace_id.clone(),
        idempotency_key,
    })
}

#[cfg(test)]
mod tests {
    use sdkwork_rpc_framework_core::{
        RpcCallerActorKind, VerifiedRpcCallerContext, VerifiedRpcServiceIdentity,
    };

    use super::*;

    fn request() -> Request<()> {
        let mut request = Request::new(());
        request.extensions_mut().insert(VerifiedRpcServiceIdentity {
            service_id: IM_SERVICE_ID.to_string(),
            trust_domain: "sdkwork.internal".to_string(),
            spiffe_uri: "spiffe://sdkwork.internal/sdkwork/service/sdkwork-im".to_string(),
            certificate_sha256: "a".repeat(64),
        });
        request.extensions_mut().insert(VerifiedRpcCallerContext {
            issuer_service_id: IM_SERVICE_ID.to_string(),
            audience_service_id: "sdkwork-knowledgebase".to_string(),
            tenant_id: "1".to_string(),
            organization_id: "2".to_string(),
            actor_id: IM_SERVICE_ID.to_string(),
            actor_kind: RpcCallerActorKind::Service,
            session_id: None,
            request_id: "request-1".to_string(),
            trace_id: Some("trace-1".to_string()),
            idempotency_key: Some("idem-1".to_string()),
            issued_at_unix_seconds: 1,
            expires_at_unix_seconds: 2,
            nonce: "nonce-1".to_string(),
        });
        request
    }

    #[test]
    fn accepts_only_the_verified_im_service_outbox_context() {
        let caller = require_group_knowledge_space_lifecycle_caller(&request())
            .expect("verified IM service context");
        assert_eq!(caller.actor_id, IM_SERVICE_ID);
        assert_eq!(caller.scope.organization_id, 2);

        let mut tenant_request = request();
        tenant_request
            .extensions_mut()
            .get_mut::<VerifiedRpcCallerContext>()
            .expect("caller context")
            .organization_id = "0".to_string();
        let tenant_caller = require_group_knowledge_space_lifecycle_caller(&tenant_request)
            .expect("verified tenant-scoped IM service context");
        assert_eq!(tenant_caller.scope.organization_id, 0);
    }

    #[test]
    fn rejects_user_sessions_and_non_im_service_identities() {
        let mut user_request = request();
        let caller = user_request
            .extensions_mut()
            .get_mut::<VerifiedRpcCallerContext>()
            .expect("caller context");
        caller.actor_kind = RpcCallerActorKind::User;
        caller.actor_id = "owner-1".to_string();
        caller.session_id = Some("session-1".to_string());
        assert_eq!(
            require_group_knowledge_space_lifecycle_caller(&user_request)
                .expect_err("user context is not an IM outbox context")
                .code(),
            tonic::Code::Unauthenticated
        );

        let mut non_im_request = request();
        non_im_request
            .extensions_mut()
            .get_mut::<VerifiedRpcServiceIdentity>()
            .expect("service identity")
            .service_id = "sdkwork-untrusted".to_string();
        assert_eq!(
            require_group_knowledge_space_lifecycle_caller(&non_im_request)
                .expect_err("other service identity must be rejected")
                .code(),
            tonic::Code::PermissionDenied
        );
    }
}
