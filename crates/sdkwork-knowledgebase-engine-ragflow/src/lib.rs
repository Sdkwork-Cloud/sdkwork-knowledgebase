//! RAGFlow external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/ragflow/engine.manifest.json`
//! Handlers MUST NOT call RAGFlow HTTP directly; only this adapter crate may integrate upstream APIs.

mod client;
mod config;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    resolve_connector_dataset_id_for_space, KnowledgeEngine,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_engine::ExternalKnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::KnowledgeSourceStore;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_external, descriptor_for_external_search_read, parse_compound_document_ref,
    KnowledgeEngineDescriptor, KnowledgeEngineDocument, KnowledgeEngineDocumentList,
    KnowledgeEngineError, KnowledgeEngineHealth, KnowledgeEngineHealthStatus,
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
    KnowledgeEngineSearchResult,
};
use std::sync::Arc;

pub use client::RagflowApiClient;
pub use config::{
    dataset_id_from_connector_metadata, RagflowConnectorConfig, RAGFLOW_BASE_URL_ENV,
    RAGFLOW_CREDENTIAL_ENV, RAGFLOW_CREDENTIAL_FILE_ENV, RAGFLOW_DATASET_ID_ENV,
};

pub const RAGFLOW_VENDOR_ID: &str = "ragflow";
pub const RAGFLOW_IMPLEMENTATION_ID: &str = "engine.knowledge.external.ragflow";
pub const RAGFLOW_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.ragflow";

pub struct RagflowKnowledgeEngine {
    config: Option<RagflowConnectorConfig>,
    client: Option<RagflowApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl RagflowKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = RagflowConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| RagflowApiClient::new(value.clone()));
        Self {
            config,
            client,
            source_store: None,
        }
    }

    pub fn from_runtime(source_store: Arc<dyn KnowledgeSourceStore>) -> Self {
        let mut engine = Self::from_env();
        engine.source_store = Some(source_store);
        engine
    }

    pub fn with_config(
        config: RagflowConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = RagflowApiClient::new(config.clone());
        Self {
            config: Some(config),
            client: Some(client),
            source_store,
        }
    }

    pub fn stub() -> Self {
        Self {
            config: None,
            client: None,
            source_store: None,
        }
    }

    fn descriptor_value(&self) -> KnowledgeEngineDescriptor {
        let display_name = if self.config.is_some() {
            "RAGFlow (external adapter)"
        } else {
            "RAGFlow (external adapter — unconfigured)"
        };
        if self.config.is_some() {
            descriptor_for_external_search_read(RAGFLOW_VENDOR_ID, display_name)
        } else {
            descriptor_for_external(RAGFLOW_VENDOR_ID, display_name)
        }
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "RAGFlow adapter requires {RAGFLOW_BASE_URL_ENV} and {RAGFLOW_CREDENTIAL_ENV}; optional default dataset via {RAGFLOW_DATASET_ID_ENV} or kb_source connector metadata datasetId"
        )
    }

    async fn resolve_dataset_id_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_dataset_id.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "RAGFlow search requires {RAGFLOW_DATASET_ID_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            RAGFLOW_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_dataset_id.clone()),
            RAGFLOW_DATASET_ID_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for RagflowKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    fn bind_provider(
        &self,
        binding: &sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineProviderBinding,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        if binding.implementation_id != RAGFLOW_IMPLEMENTATION_ID {
            return Err(KnowledgeEngineError::Validation(
                "RAGFlow cannot bind a different Provider implementation".to_string(),
            ));
        }
        let mut config = self
            .config
            .clone()
            .ok_or_else(|| KnowledgeEngineError::Unsupported(self.unconfigured_message()))?;
        config.default_dataset_id = Some(binding.remote_resource_id.clone());
        Ok(Arc::new(Self::with_config(config, None)))
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: RAGFLOW_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let dataset_id = match self
            .config
            .as_ref()
            .and_then(|config| config.default_dataset_id.clone())
        {
            Some(dataset_id) => dataset_id,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: RAGFLOW_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "RAGFlow connector health requires {RAGFLOW_DATASET_ID_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        match client.connector_health(&dataset_id).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: RAGFLOW_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: RAGFLOW_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(error.to_string()),
            }),
        }
    }

    async fn search(
        &self,
        request: KnowledgeEngineSearchRequest,
    ) -> Result<KnowledgeEngineSearchResult, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Err(KnowledgeEngineError::Unsupported(
                self.unconfigured_message(),
            ));
        };

        let dataset_id = self.resolve_dataset_id_for_space(request.space_id).await?;
        client
            .retrieve(request.space_id, &dataset_id, &request.query, request.top_k)
            .await
    }

    async fn read_document(
        &self,
        request: KnowledgeEngineReadRequest,
    ) -> Result<KnowledgeEngineDocument, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Err(KnowledgeEngineError::Unsupported(
                self.unconfigured_message(),
            ));
        };

        let (document_id, chunk_id) = parse_compound_document_ref(&request.document_id)
            .ok_or_else(|| {
                KnowledgeEngineError::Validation(
                    "RAGFlow read_document requires parentDocument#chunkId ids from search hits"
                        .to_string(),
                )
            })?;

        let dataset_id = self.resolve_dataset_id_for_space(request.space_id).await?;
        client
            .read_chunk(&dataset_id, &document_id, &chunk_id)
            .await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "adapter-tier list_documents is unsupported; use search hits or native ingestion"
                .to_string(),
        ))
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for RagflowKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "RAGFlow sync_sources is unsupported at adapter tier; use RAGFlow console ingestion"
                .to_string(),
        ))
    }
}
