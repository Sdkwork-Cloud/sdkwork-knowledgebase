//! Shared outbound runtime for external Knowledgebase providers.

mod error;
mod policy;
mod runtime;
mod telemetry;

pub use error::ProviderError;
pub use policy::{ProviderOrigin, ProviderRuntimeConfig, ProviderTargetPolicy};
pub use runtime::{
    ProviderExecutionContext, ProviderHttpRequest, ProviderHttpResponse, ProviderRuntime,
};
pub use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineProviderErrorCategory as ProviderErrorCategory,
    KnowledgeEngineProviderOperation as ProviderOperation,
};
pub use telemetry::{
    install_provider_telemetry, NoopProviderTelemetry, ProviderTelemetry, ProviderTelemetryEvent,
};
