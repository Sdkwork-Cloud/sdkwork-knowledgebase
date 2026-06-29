use serde::{Deserialize, Serialize};

// ============================================================================
// Tenant contract types
// ============================================================================
//
// DESIGN PRINCIPLES (from SDKWork tenant isolation spec):
// 1. Tenant identity is NEVER accepted from request parameters — always derived
//    from the authenticated access token via KnowledgeBackendRequestContext.
// 2. Tenant creation/management is handled by the IAM layer — knowledgebase only
//    reports tenant-level statistics (space count, document count, status).
// 3. All tenant-scoped operations derive tenant info from token claims.
// ============================================================================

/// Summary representation of the **caller's own tenant** knowledgebase status.
///
/// **Security constraint**: Only the caller's own tenant is returned.
/// No `tenant_id` is exposed — the token already encodes the tenant identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeTenantStatus {
    /// Human-readable tenant name (derived from IAM context if available)
    pub tenant_name: Option<String>,
    /// Current lifecycle status (derived from IAM context)
    pub status: KnowledgeTenantStatusEnum,
    /// Number of knowledge spaces owned by this tenant
    pub space_count: u64,
    /// Number of documents across all spaces
    pub document_count: u64,
    /// ISO 8601 creation timestamp (first space created)
    pub created_at: Option<String>,
}

/// Tenant lifecycle status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KnowledgeTenantStatusEnum {
    Active = 1,
    Suspended = 2,
    Archived = 3,
}
