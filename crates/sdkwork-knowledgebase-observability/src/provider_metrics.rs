use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use sdkwork_knowledgebase_provider_runtime::{
    install_provider_telemetry, ProviderErrorCategory, ProviderOperation, ProviderTelemetry,
    ProviderTelemetryEvent,
};

const IMPLEMENTATION_LABELS: [&str; 11] = [
    "engine.knowledge.external.dify",
    "engine.knowledge.external.ragflow",
    "engine.knowledge.external.onyx",
    "engine.knowledge.external.anythingllm",
    "engine.knowledge.external.open-webui",
    "engine.knowledge.external.flowise",
    "engine.knowledge.external.haystack",
    "engine.knowledge.external.chroma",
    "engine.knowledge.external.qdrant",
    "engine.knowledge.external.weaviate",
    "unknown",
];
const OPERATION_LABELS: [&str; 6] = ["health", "search", "read", "list", "ingest", "sync"];
const CATEGORY_LABELS: [&str; 14] = [
    "authentication",
    "permission_denied",
    "rate_limited",
    "timeout",
    "unavailable",
    "circuit_open",
    "bulkhead_saturated",
    "invalid_response",
    "response_too_large",
    "invalid_target",
    "not_found",
    "validation",
    "unsupported",
    "internal",
];
const STATUS_LABELS: [&str; 6] = ["none", "2xx", "3xx", "4xx", "5xx", "other"];
const DURATION_BUCKET_MICROSECONDS: [u64; 12] = [
    5_000, 10_000, 25_000, 50_000, 100_000, 250_000, 500_000, 1_000_000, 2_500_000, 5_000_000,
    10_000_000, 30_000_000,
];

const ERROR_DIMENSIONS: usize = CATEGORY_LABELS.len() * STATUS_LABELS.len();

struct ProviderMetricSlot {
    operations_success: AtomicU64,
    operations_error: AtomicU64,
    errors: [AtomicU64; ERROR_DIMENSIONS],
    duration_microseconds_total: AtomicU64,
    duration_count: AtomicU64,
    duration_buckets: [AtomicU64; DURATION_BUCKET_MICROSECONDS.len()],
    retries_total: AtomicU64,
    response_bytes_total: AtomicU64,
    circuit_open_total: AtomicU64,
    bulkhead_saturated_total: AtomicU64,
}

impl Default for ProviderMetricSlot {
    fn default() -> Self {
        Self {
            operations_success: AtomicU64::new(0),
            operations_error: AtomicU64::new(0),
            errors: std::array::from_fn(|_| AtomicU64::new(0)),
            duration_microseconds_total: AtomicU64::new(0),
            duration_count: AtomicU64::new(0),
            duration_buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            retries_total: AtomicU64::new(0),
            response_bytes_total: AtomicU64::new(0),
            circuit_open_total: AtomicU64::new(0),
            bulkhead_saturated_total: AtomicU64::new(0),
        }
    }
}

static PROVIDER_METRICS: LazyLock<Vec<ProviderMetricSlot>> = LazyLock::new(|| {
    (0..IMPLEMENTATION_LABELS.len() * OPERATION_LABELS.len())
        .map(|_| ProviderMetricSlot::default())
        .collect()
});

#[derive(Debug, Default)]
pub struct KnowledgebaseProviderMetrics;

impl ProviderTelemetry for KnowledgebaseProviderMetrics {
    fn record(&self, event: ProviderTelemetryEvent) {
        let implementation = implementation_index(&event.implementation_id);
        let operation = operation_index(event.operation);
        let slot = &PROVIDER_METRICS[implementation * OPERATION_LABELS.len() + operation];
        let duration_microseconds = u64::try_from(event.duration.as_micros()).unwrap_or(u64::MAX);

        slot.duration_microseconds_total
            .fetch_add(duration_microseconds, Ordering::Relaxed);
        slot.duration_count.fetch_add(1, Ordering::Relaxed);
        for (index, upper_bound) in DURATION_BUCKET_MICROSECONDS.iter().enumerate() {
            if duration_microseconds <= *upper_bound {
                slot.duration_buckets[index].fetch_add(1, Ordering::Relaxed);
            }
        }
        slot.retries_total.fetch_add(
            u64::from(event.attempts.saturating_sub(1)),
            Ordering::Relaxed,
        );
        slot.response_bytes_total.fetch_add(
            u64::try_from(event.response_bytes).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );

        match event.error_category {
            Some(category) => {
                slot.operations_error.fetch_add(1, Ordering::Relaxed);
                let error_index = category_index(category) * STATUS_LABELS.len()
                    + status_index(event.status_code);
                slot.errors[error_index].fetch_add(1, Ordering::Relaxed);
                if category == ProviderErrorCategory::CircuitOpen {
                    slot.circuit_open_total.fetch_add(1, Ordering::Relaxed);
                }
                if category == ProviderErrorCategory::BulkheadSaturated {
                    slot.bulkhead_saturated_total
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            None => {
                slot.operations_success.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

pub fn install_provider_metrics() {
    install_provider_telemetry(Arc::new(KnowledgebaseProviderMetrics));
}

pub(crate) fn render_provider_prometheus_metrics() -> String {
    let mut output = String::from(
        "# HELP knowledge_provider_operations_total Total external knowledge Provider operations.\n\
         # TYPE knowledge_provider_operations_total counter\n\
         # HELP knowledge_provider_errors_total External knowledge Provider errors by bounded category and HTTP status class.\n\
         # TYPE knowledge_provider_errors_total counter\n\
         # HELP knowledge_provider_operation_duration_seconds External knowledge Provider operation latency.\n\
         # TYPE knowledge_provider_operation_duration_seconds histogram\n\
         # HELP knowledge_provider_retries_total External knowledge Provider retry attempts.\n\
         # TYPE knowledge_provider_retries_total counter\n\
         # HELP knowledge_provider_response_bytes_total Successful external knowledge Provider response bytes.\n\
         # TYPE knowledge_provider_response_bytes_total counter\n\
         # HELP knowledge_provider_circuit_open_total External knowledge Provider operations rejected by an open circuit.\n\
         # TYPE knowledge_provider_circuit_open_total counter\n\
         # HELP knowledge_provider_bulkhead_saturated_total External knowledge Provider operations rejected by the concurrency bulkhead.\n\
         # TYPE knowledge_provider_bulkhead_saturated_total counter\n",
    );

    for (implementation_index, implementation_id) in IMPLEMENTATION_LABELS.iter().enumerate() {
        for (operation_index, operation) in OPERATION_LABELS.iter().enumerate() {
            let slot =
                &PROVIDER_METRICS[implementation_index * OPERATION_LABELS.len() + operation_index];
            let labels =
                format!("implementation_id=\"{implementation_id}\",operation=\"{operation}\"");
            let success = slot.operations_success.load(Ordering::Relaxed);
            let errors = slot.operations_error.load(Ordering::Relaxed);
            let duration_count = slot.duration_count.load(Ordering::Relaxed);

            let _ = writeln!(
                output,
                "knowledge_provider_operations_total{{{labels},outcome=\"success\"}} {success}"
            );
            let _ = writeln!(
                output,
                "knowledge_provider_operations_total{{{labels},outcome=\"error\"}} {errors}"
            );

            for (category_index, category) in CATEGORY_LABELS.iter().enumerate() {
                for (status_index, status) in STATUS_LABELS.iter().enumerate() {
                    let value = slot.errors[category_index * STATUS_LABELS.len() + status_index]
                        .load(Ordering::Relaxed);
                    let _ = writeln!(
                        output,
                        "knowledge_provider_errors_total{{{labels},category=\"{category}\",status=\"{status}\"}} {value}"
                    );
                }
            }

            for (bucket_index, upper_bound) in DURATION_BUCKET_MICROSECONDS.iter().enumerate() {
                let value = slot.duration_buckets[bucket_index].load(Ordering::Relaxed);
                let seconds = *upper_bound as f64 / 1_000_000.0;
                let _ = writeln!(
                    output,
                    "knowledge_provider_operation_duration_seconds_bucket{{{labels},le=\"{seconds}\"}} {value}"
                );
            }
            let _ = writeln!(
                output,
                "knowledge_provider_operation_duration_seconds_bucket{{{labels},le=\"+Inf\"}} {duration_count}"
            );
            let duration_seconds =
                slot.duration_microseconds_total.load(Ordering::Relaxed) as f64 / 1_000_000.0;
            let _ = writeln!(
                output,
                "knowledge_provider_operation_duration_seconds_sum{{{labels}}} {duration_seconds}"
            );
            let _ = writeln!(
                output,
                "knowledge_provider_operation_duration_seconds_count{{{labels}}} {duration_count}"
            );
            let _ = writeln!(
                output,
                "knowledge_provider_retries_total{{{labels}}} {}",
                slot.retries_total.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                output,
                "knowledge_provider_response_bytes_total{{{labels}}} {}",
                slot.response_bytes_total.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                output,
                "knowledge_provider_circuit_open_total{{{labels}}} {}",
                slot.circuit_open_total.load(Ordering::Relaxed)
            );
            let _ = writeln!(
                output,
                "knowledge_provider_bulkhead_saturated_total{{{labels}}} {}",
                slot.bulkhead_saturated_total.load(Ordering::Relaxed)
            );
        }
    }

    output
}

fn implementation_index(implementation_id: &str) -> usize {
    IMPLEMENTATION_LABELS
        .iter()
        .position(|candidate| *candidate == implementation_id)
        .unwrap_or(IMPLEMENTATION_LABELS.len() - 1)
}

fn operation_index(operation: ProviderOperation) -> usize {
    match operation {
        ProviderOperation::Health => 0,
        ProviderOperation::Search => 1,
        ProviderOperation::Read => 2,
        ProviderOperation::List => 3,
        ProviderOperation::Ingest => 4,
        ProviderOperation::Sync => 5,
    }
}

fn category_index(category: ProviderErrorCategory) -> usize {
    match category {
        ProviderErrorCategory::Authentication => 0,
        ProviderErrorCategory::PermissionDenied => 1,
        ProviderErrorCategory::RateLimited => 2,
        ProviderErrorCategory::Timeout => 3,
        ProviderErrorCategory::Unavailable => 4,
        ProviderErrorCategory::CircuitOpen => 5,
        ProviderErrorCategory::BulkheadSaturated => 6,
        ProviderErrorCategory::InvalidResponse => 7,
        ProviderErrorCategory::ResponseTooLarge => 8,
        ProviderErrorCategory::InvalidTarget => 9,
        ProviderErrorCategory::NotFound => 10,
        ProviderErrorCategory::Validation => 11,
        ProviderErrorCategory::Unsupported => 12,
        ProviderErrorCategory::Internal => 13,
    }
}

fn status_index(status: Option<u16>) -> usize {
    match status {
        None => 0,
        Some(200..=299) => 1,
        Some(300..=399) => 2,
        Some(400..=499) => 3,
        Some(500..=599) => 4,
        Some(_) => 5,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn provider_metrics_use_bounded_labels_and_export_runtime_signals() {
        KnowledgebaseProviderMetrics.record(ProviderTelemetryEvent {
            implementation_id: "engine.knowledge.external.dify".to_string(),
            operation: ProviderOperation::Search,
            error_category: Some(ProviderErrorCategory::Unavailable),
            status_code: Some(503),
            attempts: 3,
            duration: Duration::from_millis(25),
            response_bytes: 128,
        });
        KnowledgebaseProviderMetrics.record(ProviderTelemetryEvent {
            implementation_id: "unbounded-user-controlled-value".to_string(),
            operation: ProviderOperation::Read,
            error_category: None,
            status_code: Some(200),
            attempts: 1,
            duration: Duration::from_millis(5),
            response_bytes: 64,
        });

        let output = render_provider_prometheus_metrics();
        assert!(output
            .contains("implementation_id=\"engine.knowledge.external.dify\",operation=\"search\""));
        assert!(output.contains("category=\"unavailable\",status=\"5xx\""));
        assert!(output.contains("knowledge_provider_retries_total"));
        assert!(output.contains("knowledge_provider_response_bytes_total"));
        assert!(output.contains("implementation_id=\"unknown\",operation=\"read\""));
        assert!(!output.contains("unbounded-user-controlled-value"));
    }
}
