//! Tenant business quota evaluation (documents, ingest concurrency).

use sdkwork_knowledgebase_contract::KnowledgeTenantQuotaStatus;
use sdkwork_knowledgebase_observability::KnowledgebaseTenantQuotaLimits;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantQuotaKind {
    Documents,
    IngestConcurrency,
    RetrievalRate,
    StorageBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("tenant quota exceeded: {kind:?} usage={usage} limit={limit}")]
pub struct TenantQuotaExceeded {
    pub kind: TenantQuotaKind,
    pub usage: u64,
    pub limit: u64,
}

pub fn ensure_document_capacity(
    document_count: u64,
    limits: &KnowledgebaseTenantQuotaLimits,
) -> Result<(), TenantQuotaExceeded> {
    if document_count >= limits.max_documents {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::Documents,
            usage: document_count,
            limit: limits.max_documents,
        });
    }
    Ok(())
}

pub fn ensure_ingest_concurrency(
    inflight_jobs: u32,
    limits: &KnowledgebaseTenantQuotaLimits,
) -> Result<(), TenantQuotaExceeded> {
    if inflight_jobs >= limits.max_concurrent_ingest_jobs {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::IngestConcurrency,
            usage: u64::from(inflight_jobs),
            limit: u64::from(limits.max_concurrent_ingest_jobs),
        });
    }
    Ok(())
}

pub fn ensure_storage_capacity(
    storage_bytes_used: u64,
    additional_bytes: u64,
    limits: &KnowledgebaseTenantQuotaLimits,
) -> Result<(), TenantQuotaExceeded> {
    let projected = storage_bytes_used.saturating_add(additional_bytes);
    if projected > limits.max_storage_bytes {
        return Err(TenantQuotaExceeded {
            kind: TenantQuotaKind::StorageBytes,
            usage: projected,
            limit: limits.max_storage_bytes,
        });
    }
    Ok(())
}

pub fn build_quota_status(
    limits: KnowledgebaseTenantQuotaLimits,
    document_count: u64,
    inflight_ingest_jobs: u32,
    storage_bytes_used: u64,
) -> KnowledgeTenantQuotaStatus {
    KnowledgeTenantQuotaStatus {
        max_documents: limits.max_documents,
        document_count,
        max_concurrent_ingest_jobs: limits.max_concurrent_ingest_jobs,
        inflight_ingest_jobs,
        max_retrievals_per_minute: limits.max_retrievals_per_minute,
        max_storage_bytes: limits.max_storage_bytes,
        storage_bytes_used,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_quota_blocks_at_limit() {
        let limits = KnowledgebaseTenantQuotaLimits {
            max_documents: 10,
            ..KnowledgebaseTenantQuotaLimits::default()
        };
        assert!(ensure_document_capacity(9, &limits).is_ok());
        assert!(ensure_document_capacity(10, &limits).is_err());
    }

    #[test]
    fn ingest_concurrency_quota_blocks_at_limit() {
        let limits = KnowledgebaseTenantQuotaLimits {
            max_concurrent_ingest_jobs: 2,
            ..KnowledgebaseTenantQuotaLimits::default()
        };
        assert!(ensure_ingest_concurrency(1, &limits).is_ok());
        assert!(ensure_ingest_concurrency(2, &limits).is_err());
    }

    #[test]
    fn storage_quota_blocks_when_projected_usage_exceeds_limit() {
        let limits = KnowledgebaseTenantQuotaLimits {
            max_storage_bytes: 1_000,
            ..KnowledgebaseTenantQuotaLimits::default()
        };
        assert!(ensure_storage_capacity(900, 50, &limits).is_ok());
        assert!(ensure_storage_capacity(900, 101, &limits).is_err());
    }
}
