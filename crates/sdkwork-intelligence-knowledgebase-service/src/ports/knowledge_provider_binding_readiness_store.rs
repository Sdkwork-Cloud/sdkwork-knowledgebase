use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::knowledge_provider_binding_store::KnowledgeEngineProviderScope;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListKnowledgeEngineProviderBindingReadinessGapsRequest {
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineProviderBindingReadinessGap {
    #[serde(with = "sdkwork_utils_rust::serde_uint64")]
    pub space_id: u64,
    pub space_uuid: String,
    #[serde(with = "sdkwork_utils_rust::serde_uint64")]
    pub non_active_binding_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineProviderBindingReadinessGapPage {
    pub items: Vec<KnowledgeEngineProviderBindingReadinessGap>,
    pub next_cursor: Option<String>,
}

#[async_trait]
pub trait KnowledgeEngineProviderBindingReadinessStore: Send + Sync {
    async fn list_spaces_missing_active_binding(
        &self,
        scope: KnowledgeEngineProviderScope,
        request: ListKnowledgeEngineProviderBindingReadinessGapsRequest,
    ) -> Result<
        KnowledgeEngineProviderBindingReadinessGapPage,
        KnowledgeEngineProviderBindingReadinessStoreError,
    >;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineProviderBindingReadinessStoreError {
    #[error("knowledge engine Provider Binding readiness request is invalid: {0}")]
    InvalidRequest(String),
    #[error("knowledge engine Provider Binding readiness query failed")]
    QueryFailed,
}
