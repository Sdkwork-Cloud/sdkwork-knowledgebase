//! Shared runtime environment helpers for SDKWork Knowledgebase services.

/// Returns true only when `SDKWORK_KNOWLEDGEBASE_ENVIRONMENT` is explicitly `development`.
pub fn is_development_environment() -> bool {
    knowledgebase_environment()
        .map(|value| value.eq_ignore_ascii_case("development"))
        .unwrap_or(false)
}

/// Production-like environments fail closed on unsafe defaults.
pub fn is_production_like_environment() -> bool {
    matches!(
        knowledgebase_environment()
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("production") | Some("staging") | Some("test")
    )
}

pub fn knowledgebase_environment() -> Option<String> {
    std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT").ok()
}

/// Deployment-bound tenant identifier used for billing events when request context is unavailable.
pub fn deployment_tenant_id() -> u64 {
    std::env::var("SDKWORK_KNOWLEDGEBASE_TENANT_ID")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_like_matches_expected_values() {
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "production");
        assert!(is_production_like_environment());
        std::env::set_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT", "staging");
        assert!(is_production_like_environment());
        std::env::remove_var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT");
        assert!(!is_production_like_environment());
    }
}
