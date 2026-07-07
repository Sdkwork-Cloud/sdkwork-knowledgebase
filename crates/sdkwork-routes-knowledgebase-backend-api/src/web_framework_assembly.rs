//! Production-oriented SDKWork web-framework wiring for Knowledgebase HTTP surfaces.

use std::sync::Arc;

use sdkwork_knowledgebase_observability::environment::is_production_like_environment;
use sdkwork_web_axum::WebFrameworkLayer;
use sdkwork_web_core::{
    CorsPolicy, EnforcePrincipalTenantIsolationPolicy, SecurityPolicy, WebRequestContextResolver,
};

/// Standalone knowledgebase PC browser dev server defaults.
const KNOWLEDGEBASE_STANDALONE_DEV_ORIGINS: &[&str] =
    &["http://127.0.0.1:5184", "http://localhost:5184"];

/// IM host-embedded browser surfaces (PC/H5/desktop) when knowledgebase routes are mounted in IM.
const IM_HOST_EMBEDDED_DEV_ORIGINS: &[&str] = &[
    "http://127.0.0.1:4176",
    "http://localhost:4176",
    "http://127.0.0.1:1620",
    "http://localhost:1620",
    "tauri://localhost",
];

const DEV_BROWSER_ORIGIN_ENV_KEYS: &[&str] = &[
    "SDKWORK_KNOWLEDGEBASE_DEV_ALLOWED_ORIGINS",
    "SDKWORK_IM_BROWSER_ORIGINS",
];

const PRODUCTION_BROWSER_ORIGIN_ENV_KEYS: &[&str] = &[
    "SDKWORK_IM_BROWSER_ORIGINS",
    "SDKWORK_KNOWLEDGEBASE_BROWSER_ORIGINS",
];

fn parse_browser_origin_list(raw: &str) -> Vec<String> {
    let mut origins = Vec::new();
    for value in raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let normalized = value.trim_end_matches('/').to_owned();
        if !origins.contains(&normalized) {
            origins.push(normalized);
        }
    }
    origins
}

fn merge_browser_origins_from_env(keys: &[&str]) -> Vec<String> {
    let mut origins = Vec::new();
    for key in keys {
        let Ok(raw) = std::env::var(key) else {
            continue;
        };
        for origin in parse_browser_origin_list(&raw) {
            if !origins.contains(&origin) {
                origins.push(origin);
            }
        }
    }
    origins
}

fn default_dev_allowed_origins() -> Vec<String> {
    KNOWLEDGEBASE_STANDALONE_DEV_ORIGINS
        .iter()
        .chain(IM_HOST_EMBEDDED_DEV_ORIGINS.iter())
        .map(|value| (*value).to_owned())
        .collect()
}

fn resolve_dev_allowed_origins() -> Vec<String> {
    let configured = merge_browser_origins_from_env(DEV_BROWSER_ORIGIN_ENV_KEYS);
    if configured.is_empty() {
        return default_dev_allowed_origins();
    }
    configured
}

fn resolve_production_allowed_origins() -> Vec<String> {
    merge_browser_origins_from_env(PRODUCTION_BROWSER_ORIGIN_ENV_KEYS)
}

fn knowledgebase_security_policy() -> SecurityPolicy {
    if is_production_like_environment() {
        let mut security_policy = SecurityPolicy::production();
        let allowed_origins = resolve_production_allowed_origins();
        if !allowed_origins.is_empty() {
            security_policy.cors.allowed_origins = allowed_origins;
        }
        return security_policy;
    }

    // Dev CORS remains explicit (SECURITY_SPEC §4 / §5.1): environment-specific allowlist only.
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

#[cfg(test)]
mod tests {
    use super::{
        default_dev_allowed_origins, parse_browser_origin_list, resolve_dev_allowed_origins,
    };

    #[test]
    fn parse_browser_origin_list_normalizes_trailing_slashes() {
        assert_eq!(
            parse_browser_origin_list("http://localhost:4176/, http://127.0.0.1:4176"),
            vec![
                "http://localhost:4176".to_owned(),
                "http://127.0.0.1:4176".to_owned(),
            ]
        );
    }

    #[test]
    fn default_dev_allowed_origins_include_im_host_embedded_surfaces() {
        let origins = default_dev_allowed_origins();
        assert!(origins.contains(&"http://127.0.0.1:4176".to_owned()));
        assert!(origins.contains(&"http://127.0.0.1:5184".to_owned()));
    }

    #[test]
    fn resolve_dev_allowed_origins_prefers_im_browser_origins_env() {
        unsafe {
            std::env::set_var(
                "SDKWORK_IM_BROWSER_ORIGINS",
                "http://127.0.0.1:4188,http://localhost:4188",
            );
        }
        let origins = resolve_dev_allowed_origins();
        assert!(origins.contains(&"http://127.0.0.1:4188".to_owned()));
        assert!(origins.contains(&"http://localhost:4188".to_owned()));
        unsafe {
            std::env::remove_var("SDKWORK_IM_BROWSER_ORIGINS");
        }
    }
}
