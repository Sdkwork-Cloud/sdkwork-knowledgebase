//! Billable usage counters and structured billing events for commercial metering.

use std::sync::atomic::{AtomicU64, Ordering};

static RETRIEVALS_TOTAL: AtomicU64 = AtomicU64::new(0);
static CONTEXT_PACKS_TOTAL: AtomicU64 = AtomicU64::new(0);
static INGEST_JOBS_SUCCEEDED_TOTAL: AtomicU64 = AtomicU64::new(0);
static INGEST_JOBS_FAILED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Records a completed knowledge retrieval suitable for usage-based billing.
pub fn record_retrieval_completed(tenant_id: u64, result_count: u32, latency_ms: u64) {
    RETRIEVALS_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        billing_event = "knowledge.retrieval.completed",
        tenant_id,
        result_count,
        latency_ms,
        "knowledge retrieval completed"
    );
}

/// Records a completed context pack assembly (Open API billable unit).
pub fn record_context_pack_completed(tenant_id: u64, estimated_tokens: u32, truncated: bool) {
    CONTEXT_PACKS_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        billing_event = "knowledge.context_pack.completed",
        tenant_id,
        estimated_tokens,
        truncated,
        "knowledge context pack completed"
    );
}

/// Records a successful ingest job completion.
pub fn record_ingest_job_succeeded(tenant_id: u64, job_id: u64, space_id: u64) {
    INGEST_JOBS_SUCCEEDED_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        billing_event = "knowledge.ingest.succeeded",
        tenant_id,
        job_id,
        space_id,
        "ingest job succeeded"
    );
}

/// Records a failed ingest job terminal state.
pub fn record_ingest_job_failed(tenant_id: u64, job_id: u64, space_id: u64) {
    INGEST_JOBS_FAILED_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        billing_event = "knowledge.ingest.failed",
        tenant_id,
        job_id,
        space_id,
        "ingest job failed"
    );
}

pub fn render_billing_prometheus_metrics() -> String {
    format!(
        "# HELP knowledge_retrievals_total Completed knowledge retrieval operations.\n\
         # TYPE knowledge_retrievals_total counter\n\
         knowledge_retrievals_total {}\n\
         # HELP knowledge_context_packs_total Completed knowledge context pack operations.\n\
         # TYPE knowledge_context_packs_total counter\n\
         knowledge_context_packs_total {}\n\
         # HELP knowledge_ingest_jobs_succeeded_total Successful ingest job completions.\n\
         # TYPE knowledge_ingest_jobs_succeeded_total counter\n\
         knowledge_ingest_jobs_succeeded_total {}\n\
         # HELP knowledge_ingest_jobs_failed_total Failed ingest job terminal states.\n\
         # TYPE knowledge_ingest_jobs_failed_total counter\n\
         knowledge_ingest_jobs_failed_total {}\n",
        RETRIEVALS_TOTAL.load(Ordering::Relaxed),
        CONTEXT_PACKS_TOTAL.load(Ordering::Relaxed),
        INGEST_JOBS_SUCCEEDED_TOTAL.load(Ordering::Relaxed),
        INGEST_JOBS_FAILED_TOTAL.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_billing_counters_for_prometheus_scrape() {
        record_retrieval_completed(9001, 3, 42);
        record_context_pack_completed(9001, 512, false);
        record_ingest_job_succeeded(9001, 10, 20);
        let body = render_billing_prometheus_metrics();
        assert!(body.contains("knowledge_retrievals_total"));
        assert!(body.contains("knowledge_context_packs_total"));
        assert!(body.contains("knowledge_ingest_jobs_succeeded_total"));
        assert!(body.contains("knowledge_ingest_jobs_failed_total"));
    }
}
