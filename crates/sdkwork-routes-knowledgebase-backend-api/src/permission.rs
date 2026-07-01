use crate::ports::KnowledgeBackendRequestContext;

pub const KNOWLEDGE_PLATFORM_MANAGE_PERMISSION: &str = "knowledge.platform.manage";

/// Deprecated legacy wire code retained for migration compatibility only.
pub const KNOWLEDGE_ADMIN_PERMISSION: &str = "knowledge.admin";

pub fn can_access_knowledge_admin(context: &KnowledgeBackendRequestContext) -> bool {
    context
        .permission_scope
        .iter()
        .any(|scope| is_knowledge_platform_manage_scope(scope))
}

fn is_knowledge_platform_manage_scope(scope: &str) -> bool {
    scope == KNOWLEDGE_PLATFORM_MANAGE_PERMISSION
        || scope == KNOWLEDGE_ADMIN_PERMISSION
        || scope == "knowledge.*"
        || (scope.starts_with("knowledge.") && scope.ends_with(".admin"))
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
        }
    }

    #[test]
    fn allows_knowledge_platform_manage_permission() {
        let context = context_with_scopes(&[KNOWLEDGE_PLATFORM_MANAGE_PERMISSION]);
        assert!(can_access_knowledge_admin(&context));
    }

    #[test]
    fn allows_legacy_knowledge_admin_permission() {
        let context = context_with_scopes(&[KNOWLEDGE_ADMIN_PERMISSION]);
        assert!(can_access_knowledge_admin(&context));
    }

    #[test]
    fn allows_knowledge_wildcard_permission() {
        let context = context_with_scopes(&["knowledge.*"]);
        assert!(can_access_knowledge_admin(&context));
    }

    #[test]
    fn rejects_missing_admin_permission() {
        let context = context_with_scopes(&["knowledge.spaces.read"]);
        assert!(!can_access_knowledge_admin(&context));
    }
}
