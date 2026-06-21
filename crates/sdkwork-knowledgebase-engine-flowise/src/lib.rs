//! Flowise external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/flowise/engine.manifest.json`
//! Handlers MUST NOT call Flowise HTTP directly; only this adapter crate may integrate upstream APIs.

mod client;
mod config;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    resolve_connector_dataset_id_for_space, KnowledgeEngine,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_engine::ExternalKnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::KnowledgeSourceStore;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_external, parse_compound_document_ref, KnowledgeEngineDescriptor,
    KnowledgeEngineDocument, KnowledgeEngineDocumentList, KnowledgeEngineDocumentRef,
    KnowledgeEngineError, KnowledgeEngineHealth, KnowledgeEngineHealthStatus,
    KnowledgeEngineListRequest, KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest,
    KnowledgeEngineSearchResult,
};
use std::sync::Arc;

pub use client::{chunk_id_from_content, FlowiseApiClient};
pub use config::{
    store_id_from_connector_metadata, FlowiseConnectorConfig, FLOWISE_API_KEY_ENV,
    FLOWISE_BASE_URL_ENV, FLOWISE_STORE_ID_ENV,
};

pub const FLOWISE_VENDOR_ID: &str = "flowise";
pub const FLOWISE_IMPLEMENTATION_ID: &str = "engine.knowledge.external.flowise";
pub const FLOWISE_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.flowise";

pub struct FlowiseKnowledgeEngine {
    config: Option<FlowiseConnectorConfig>,
    client: Option<FlowiseApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl FlowiseKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = FlowiseConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| FlowiseApiClient::new(value.clone()));
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
        config: FlowiseConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = FlowiseApiClient::new(config.clone());
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
            "Flowise (external adapter)"
        } else {
            "Flowise (external adapter — unconfigured)"
        };
        descriptor_for_external(FLOWISE_VENDOR_ID, display_name)
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Flowise adapter requires {FLOWISE_BASE_URL_ENV} and {FLOWISE_API_KEY_ENV}; optional default document store via {FLOWISE_STORE_ID_ENV} or kb_source connector metadata datasetId"
        )
    }

    async fn resolve_store_id_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_store_id.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Flowise search requires {FLOWISE_STORE_ID_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            FLOWISE_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_store_id.clone()),
            &format!("{FLOWISE_STORE_ID_ENV}"),
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for FlowiseKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: FLOWISE_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let store_id = match self
            .config
            .as_ref()
            .and_then(|config| config.default_store_id.clone())
        {
            Some(store_id) => store_id,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: FLOWISE_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "Flowise connector health requires {FLOWISE_STORE_ID_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        match client.connector_health(&store_id).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: FLOWISE_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: FLOWISE_IMPLEMENTATION_ID.to_string(),
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

        let store_id = self.resolve_store_id_for_space(request.space_id).await?;
        client
            .query_vector_store(request.space_id, &store_id, &request.query, request.top_k)
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

        let (document_hint, chunk_id) = parse_compound_document_ref(&request.document_id)
            .ok_or_else(|| {
                KnowledgeEngineError::Validation(
                    "Flowise read_document requires source#chunkId ids from search hits"
                        .to_string(),
                )
            })?;

        let store_id = self.resolve_store_id_for_space(request.space_id).await?;
        client
            .read_chunk(request.space_id, &store_id, &document_hint, &chunk_id)
            .await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Ok(KnowledgeEngineDocumentList {
            items: Vec::<KnowledgeEngineDocumentRef>::new(),
        })
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for FlowiseKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Flowise sync_sources is managed via document store UI; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
