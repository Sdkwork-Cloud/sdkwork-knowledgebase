use std::time::Duration;

use thiserror::Error;

use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineError, KnowledgeEngineProviderFailure,
};

use crate::{ProviderErrorCategory, ProviderOperation};

#[derive(Debug, Clone, Error)]
#[error("provider {operation} failed ({category:?}): {safe_message}")]
pub struct ProviderError {
    pub category: ProviderErrorCategory,
    pub operation: ProviderOperation,
    pub implementation_id: String,
    pub binding_id: Option<String>,
    pub status_code: Option<u16>,
    pub retryable: bool,
    pub retry_after: Option<Duration>,
    pub safe_message: String,
}

impl ProviderError {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        category: ProviderErrorCategory,
        operation: ProviderOperation,
        implementation_id: impl Into<String>,
        binding_id: Option<String>,
        status_code: Option<u16>,
        retryable: bool,
        retry_after: Option<Duration>,
        safe_message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            operation,
            implementation_id: implementation_id.into(),
            binding_id,
            status_code,
            retryable,
            retry_after,
            safe_message: safe_message.into(),
        }
    }
}

impl From<ProviderError> for KnowledgeEngineError {
    fn from(error: ProviderError) -> Self {
        Self::Provider(KnowledgeEngineProviderFailure {
            category: error.category,
            operation: error.operation,
            implementation_id: error.implementation_id,
            binding_id: error.binding_id,
            status_code: error.status_code,
            retryable: error.retryable,
            retry_after_ms: error
                .retry_after
                .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)),
            safe_message: error.safe_message,
        })
    }
}
