use crate::ports::KnowledgeBackendRequestContext;

pub const KNOWLEDGE_PLATFORM_MANAGE_PERMISSION: &str = "knowledge.platform.manage";

pub fn can_access_knowledge_admin(context: &KnowledgeBackendRequestContext) -> bool {
    context
        .permission_scope
        .iter()
        .any(|scope| is_knowledge_platform_manage_scope(scope))
}

fn is_knowledge_platform_manage_scope(scope: &str) -> bool {
    // Exact permission only: wildcard scopes would grant every backend administration
    // capability (including credential rotation/revocation and audit export) to any
    // principal holding a domain-level grant.
    scope == KNOWLEDGE_PLATFORM_MANAGE_PERMISSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_with_scopes(scopes: &[&str]) -> KnowledgeBackendRequestContext {
        KnowledgeBackendRequestContext {
            tenant_id: 100_001,
            operator_id: Some(99),
            organization_id: None,
            permission_scope: scopes.iter().map(|scope| (*scope).to_string()).collect(),
            trace_id: "trace-permission-test".to_string(),
        }
    }

    #[test]
    fn allows_knowledge_platform_manage_permission() {
        let context = context_with_scopes(&[KNOWLEDGE_PLATFORM_MANAGE_PERMISSION]);
        assert!(can_access_knowledge_admin(&context));
    }

    #[test]
    fn rejects_retired_knowledge_admin_permission() {
        let context = context_with_scopes(&["knowledge.admin"]);
        assert!(!can_access_knowledge_admin(&context));
    }

    #[test]
    fn rejects_knowledge_wildcard_permission() {
        // Least-privilege: a wildcard domain grant must not unlock backend administration.
        let context = context_with_scopes(&["knowledge.*"]);
        assert!(!can_access_knowledge_admin(&context));
    }

    #[test]
    fn rejects_granular_domain_admin_scope() {
        let context = context_with_scopes(&["knowledge.spaces.admin"]);
        assert!(!can_access_knowledge_admin(&context));
    }

    #[test]
    fn rejects_missing_admin_permission() {
        let context = context_with_scopes(&["knowledge.spaces.read"]);
        assert!(!can_access_knowledge_admin(&context));
    }
}
