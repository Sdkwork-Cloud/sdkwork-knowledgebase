use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;

use crate::{ProviderErrorCategory, ProviderOperation};

#[derive(Debug, Clone)]
pub struct ProviderTelemetryEvent {
    pub implementation_id: String,
    pub operation: ProviderOperation,
    pub error_category: Option<ProviderErrorCategory>,
    pub status_code: Option<u16>,
    pub attempts: u32,
    pub duration: Duration,
    pub response_bytes: usize,
}

pub trait ProviderTelemetry: Send + Sync {
    fn record(&self, event: ProviderTelemetryEvent);
}

#[derive(Debug, Default)]
pub struct NoopProviderTelemetry;

impl ProviderTelemetry for NoopProviderTelemetry {
    fn record(&self, _event: ProviderTelemetryEvent) {}
}

struct GlobalProviderTelemetry;

impl ProviderTelemetry for GlobalProviderTelemetry {
    fn record(&self, event: ProviderTelemetryEvent) {
        let telemetry = global_telemetry_slot()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        telemetry.record(event);
    }
}

fn global_telemetry_slot() -> &'static RwLock<Arc<dyn ProviderTelemetry>> {
    static GLOBAL_TELEMETRY: OnceLock<RwLock<Arc<dyn ProviderTelemetry>>> = OnceLock::new();
    GLOBAL_TELEMETRY.get_or_init(|| RwLock::new(Arc::new(NoopProviderTelemetry)))
}

pub fn install_provider_telemetry(telemetry: Arc<dyn ProviderTelemetry>) {
    *global_telemetry_slot()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = telemetry;
}

pub(crate) fn default_telemetry() -> Arc<dyn ProviderTelemetry> {
    Arc::new(GlobalProviderTelemetry)
}
