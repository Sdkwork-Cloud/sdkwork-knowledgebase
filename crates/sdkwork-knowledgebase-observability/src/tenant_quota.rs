//! Per-tenant business quota limits for commercial SaaS enforcement.

use sdkwork_utils_rust::is_blank;
use std::env;

const DEFAULT_MAX_DOCUMENTS: u64 = 100_000;
const DEFAULT_MAX_CONCURRENT_INGEST_JOBS: u32 = 32;
const DEFAULT_MAX_RETRIEVALS_PER_MINUTE: u32 = 600;
const DEFAULT_MAX_STORAGE_BYTES: u64 = 100 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgebaseTenantQuotaLimits {
    pub max_documents: u64,
    pub max_concurrent_ingest_jobs: u32,
    pub max_retrievals_per_minute: u32,
    pub max_storage_bytes: u64,
}

impl Default for KnowledgebaseTenantQuotaLimits {
    fn default() -> Self {
        Self {
            max_documents: DEFAULT_MAX_DOCUMENTS,
            max_concurrent_ingest_jobs: DEFAULT_MAX_CONCURRENT_INGEST_JOBS,
            max_retrievals_per_minute: DEFAULT_MAX_RETRIEVALS_PER_MINUTE,
            max_storage_bytes: DEFAULT_MAX_STORAGE_BYTES,
        }
    }
}

impl KnowledgebaseTenantQuotaLimits {
    pub fn from_env() -> Self {
        Self {
            max_documents: read_u64_env(
                "SDKWORK_KNOWLEDGEBASE_TENANT_MAX_DOCUMENTS",
                DEFAULT_MAX_DOCUMENTS,
            ),
            max_concurrent_ingest_jobs: read_u32_env(
                "SDKWORK_KNOWLEDGEBASE_TENANT_MAX_CONCURRENT_INGEST_JOBS",
                DEFAULT_MAX_CONCURRENT_INGEST_JOBS,
            ),
            max_retrievals_per_minute: read_u32_env(
                "SDKWORK_KNOWLEDGEBASE_TENANT_MAX_RETRIEVALS_PER_MINUTE",
                DEFAULT_MAX_RETRIEVALS_PER_MINUTE,
            ),
            max_storage_bytes: read_u64_env(
                "SDKWORK_KNOWLEDGEBASE_TENANT_MAX_STORAGE_BYTES",
                DEFAULT_MAX_STORAGE_BYTES,
            ),
        }
    }
}

fn read_u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

fn read_u32_env(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limits_are_positive() {
        let limits = KnowledgebaseTenantQuotaLimits::default();
        assert!(limits.max_documents > 0);
        assert!(limits.max_concurrent_ingest_jobs > 0);
        assert!(limits.max_retrievals_per_minute > 0);
        assert!(limits.max_storage_bytes > 0);
    }
}
