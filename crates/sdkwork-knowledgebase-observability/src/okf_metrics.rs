//! OKF bundle Prometheus counters and structured audit log lines.

use std::sync::atomic::{AtomicU64, Ordering};

static OKF_CONCEPT_PUBLISH_TOTAL: AtomicU64 = AtomicU64::new(0);
static OKF_CONCEPT_UPSERT_TOTAL: AtomicU64 = AtomicU64::new(0);
static OKF_BUNDLE_LINT_ISSUES_TOTAL: AtomicU64 = AtomicU64::new(0);
static OKF_CONFORMANCE_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);
static OKF_BUNDLE_IMPORT_TOTAL: AtomicU64 = AtomicU64::new(0);
static OKF_BUNDLE_EXPORT_TOTAL: AtomicU64 = AtomicU64::new(0);

pub fn record_okf_concept_upsert(space_id: u64, concept_id: &str, actor: &str) {
    OKF_CONCEPT_UPSERT_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::debug!(
        audit_event = "okf.concept.upserted",
        space_id,
        concept_id,
        actor,
        "okf concept upserted"
    );
}

pub fn record_okf_concept_publish(space_id: u64, concept_id: &str, actor: &str) {
    OKF_CONCEPT_PUBLISH_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "okf.concept.published",
        space_id,
        concept_id,
        actor,
        "okf concept published"
    );
}

pub fn record_okf_bundle_lint_completed(
    space_id: u64,
    issue_count: u64,
    conformance_failures: u64,
) {
    if issue_count > 0 {
        OKF_BUNDLE_LINT_ISSUES_TOTAL.fetch_add(issue_count, Ordering::Relaxed);
    }
    if conformance_failures > 0 {
        OKF_CONFORMANCE_FAILURES_TOTAL.fetch_add(conformance_failures, Ordering::Relaxed);
    }
    tracing::info!(
        audit_event = "okf.bundle.lint.completed",
        space_id,
        issue_count,
        conformance_failures,
        "okf bundle lint completed"
    );
}

pub fn record_okf_bundle_imported(space_id: u64, imported_concept_count: u32, actor: &str) {
    OKF_BUNDLE_IMPORT_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "okf.bundle.imported",
        space_id,
        imported_concept_count,
        actor,
        "okf bundle imported"
    );
}

pub fn record_okf_bundle_exported(space_id: u64, export_type: &str, file_count: u32) {
    OKF_BUNDLE_EXPORT_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "okf.bundle.exported",
        space_id,
        export_type,
        file_count,
        "okf bundle exported"
    );
}

pub fn render_okf_prometheus_metrics() -> String {
    format!(
        "# HELP kb_okf_concept_publish_total Total OKF concepts published to the bundle tree.\n\
         # TYPE kb_okf_concept_publish_total counter\n\
         kb_okf_concept_publish_total {}\n\
         # HELP kb_okf_concept_upsert_total Total OKF concept upserts including candidates.\n\
         # TYPE kb_okf_concept_upsert_total counter\n\
         kb_okf_concept_upsert_total {}\n\
         # HELP kb_okf_bundle_lint_issues_total Total OKF bundle lint issues recorded.\n\
         # TYPE kb_okf_bundle_lint_issues_total counter\n\
         kb_okf_bundle_lint_issues_total {}\n\
         # HELP kb_okf_conformance_failures_total Total OKF conformance failures from lint.\n\
         # TYPE kb_okf_conformance_failures_total counter\n\
         kb_okf_conformance_failures_total {}\n\
         # HELP kb_okf_bundle_import_total Total OKF bundle import operations.\n\
         # TYPE kb_okf_bundle_import_total counter\n\
         kb_okf_bundle_import_total {}\n\
         # HELP kb_okf_bundle_export_total Total OKF bundle export operations.\n\
         # TYPE kb_okf_bundle_export_total counter\n\
         kb_okf_bundle_export_total {}\n",
        OKF_CONCEPT_PUBLISH_TOTAL.load(Ordering::Relaxed),
        OKF_CONCEPT_UPSERT_TOTAL.load(Ordering::Relaxed),
        OKF_BUNDLE_LINT_ISSUES_TOTAL.load(Ordering::Relaxed),
        OKF_CONFORMANCE_FAILURES_TOTAL.load(Ordering::Relaxed),
        OKF_BUNDLE_IMPORT_TOTAL.load(Ordering::Relaxed),
        OKF_BUNDLE_EXPORT_TOTAL.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn okf_metrics_export_prometheus_lines() {
        record_okf_concept_publish(1, "tables/users", "author");
        record_okf_concept_upsert(1, "tables/users", "author");
        record_okf_bundle_lint_completed(1, 2, 1);
        record_okf_bundle_imported(1, 3, "importer");
        record_okf_bundle_exported(1, "okf_strict", 5);

        let body = render_okf_prometheus_metrics();
        assert!(body.contains("kb_okf_concept_publish_total 1"));
        assert!(body.contains("kb_okf_concept_upsert_total 1"));
        assert!(body.contains("kb_okf_bundle_lint_issues_total 2"));
        assert!(body.contains("kb_okf_conformance_failures_total 1"));
        assert!(body.contains("kb_okf_bundle_import_total 1"));
        assert!(body.contains("kb_okf_bundle_export_total 1"));
    }
}
