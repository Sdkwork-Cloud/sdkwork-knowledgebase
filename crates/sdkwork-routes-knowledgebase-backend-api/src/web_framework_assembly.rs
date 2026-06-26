//! Production-oriented SDKWork web-framework wiring for Knowledgebase HTTP surfaces.

use std::sync::Arc;

use sdkwork_knowledgebase_observability::environment::is_production_like_environment;
use sdkwork_web_axum::WebFrameworkLayer;
use sdkwork_web_core::{
    EnforcePrincipalTenantIsolationPolicy, SecurityPolicy, WebRequestContextResolver,
};

/// Applies tenant isolation and production security defaults for Knowledgebase HTTP surfaces.
pub fn apply_knowledgebase_web_framework<R>(layer: WebFrameworkLayer<R>) -> WebFrameworkLayer<R>
where
    R: WebRequestContextResolver + Clone,
{
    let layer = layer.with_tenant_isolation_policy(Arc::new(EnforcePrincipalTenantIsolationPolicy));
    if is_production_like_environment() {
        layer.with_security_policy(SecurityPolicy::production())
    } else {
        layer
    }
}
