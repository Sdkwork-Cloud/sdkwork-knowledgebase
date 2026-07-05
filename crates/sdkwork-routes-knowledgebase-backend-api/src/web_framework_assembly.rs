//! Production-oriented SDKWork web-framework wiring for Knowledgebase HTTP surfaces.

use std::sync::Arc;

use sdkwork_knowledgebase_observability::environment::is_production_like_environment;
use sdkwork_web_axum::WebFrameworkLayer;
use sdkwork_web_core::{
    CorsPolicy, EnforcePrincipalTenantIsolationPolicy, SecurityPolicy, WebRequestContextResolver,
};

const DEFAULT_DEV_ALLOWED_ORIGINS: &[&str] = &["http://127.0.0.1:5184", "http://localhost:5184"];

fn resolve_dev_allowed_origins() -> Vec<String> {
    let configured = std::env::var("SDKWORK_KNOWLEDGEBASE_DEV_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if configured.is_empty() {
        return DEFAULT_DEV_ALLOWED_ORIGINS
            .iter()
            .map(|value| (*value).to_owned())
            .collect();
    }
    configured
}

fn knowledgebase_security_policy() -> SecurityPolicy {
    if is_production_like_environment() {
        return SecurityPolicy::production();
    }

    // Dev CORS remains explicit (SECURITY_SPEC): allow only configured local origins.
    SecurityPolicy {
        cors: CorsPolicy {
            allow_all_origins: false,
            allowed_origins: resolve_dev_allowed_origins(),
            ..CorsPolicy::default()
        },
        ..SecurityPolicy::default()
    }
}

/// Applies tenant isolation and production security defaults for Knowledgebase HTTP surfaces.
pub fn apply_knowledgebase_web_framework<R>(layer: WebFrameworkLayer<R>) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    layer
        .with_tenant_isolation_policy(Arc::new(EnforcePrincipalTenantIsolationPolicy))
        .with_security_policy(knowledgebase_security_policy())
}
