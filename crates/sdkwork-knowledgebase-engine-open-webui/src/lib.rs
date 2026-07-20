//! Open WebUI external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/open-webui/engine.manifest.json`
//! Handlers MUST NOT call Open WebUI HTTP directly; only this adapter crate may integrate upstream APIs.

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

pub use client::{chunk_id_from_content, OpenWebuiApiClient};
pub use config::{
    knowledge_id_from_connector_metadata, OpenWebuiConnectorConfig, OPEN_WEBUI_BASE_URL_ENV,
    OPEN_WEBUI_CREDENTIAL_ENV, OPEN_WEBUI_CREDENTIAL_FILE_ENV, OPEN_WEBUI_KNOWLEDGE_ID_ENV,
};

pub const OPEN_WEBUI_VENDOR_ID: &str = "open-webui";
pub const OPEN_WEBUI_IMPLEMENTATION_ID: &str = "engine.knowledge.external.open-webui";
pub const OPEN_WEBUI_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.open-webui";

pub struct OpenWebuiKnowledgeEngine {
    config: Option<OpenWebuiConnectorConfig>,
    client: Option<OpenWebuiApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl OpenWebuiKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = OpenWebuiConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| OpenWebuiApiClient::new(value.clone()));
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
        config: OpenWebuiConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = OpenWebuiApiClient::new(config.clone());
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
            "Open WebUI (external adapter)"
        } else {
            "Open WebUI (external adapter — unconfigured)"
        };
        if self.config.is_some() {
            descriptor_for_external_search_read(OPEN_WEBUI_VENDOR_ID, display_name)
        } else {
            descriptor_for_external(OPEN_WEBUI_VENDOR_ID, display_name)
        }
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Open WebUI adapter requires {OPEN_WEBUI_BASE_URL_ENV} and {OPEN_WEBUI_CREDENTIAL_ENV}; optional default knowledge collection via {OPEN_WEBUI_KNOWLEDGE_ID_ENV} or kb_source connector metadata datasetId"
        )
    }

    async fn resolve_knowledge_id_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_knowledge_id.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Open WebUI search requires {OPEN_WEBUI_KNOWLEDGE_ID_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            OPEN_WEBUI_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_knowledge_id.clone()),
            OPEN_WEBUI_KNOWLEDGE_ID_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for OpenWebuiKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    fn bind_provider(
        &self,
        binding: &sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineProviderBinding,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        if binding.implementation_id != OPEN_WEBUI_IMPLEMENTATION_ID {
            return Err(KnowledgeEngineError::Validation(
                "Open WebUI cannot bind a different Provider implementation".to_string(),
            ));
        }
        let mut config = self
            .config
            .clone()
            .ok_or_else(|| KnowledgeEngineError::Unsupported(self.unconfigured_message()))?;
        config.default_knowledge_id = Some(binding.remote_resource_id.clone());
        Ok(Arc::new(Self::with_config(config, None)))
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: OPEN_WEBUI_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let knowledge_id = match self
            .config
            .as_ref()
            .and_then(|config| config.default_knowledge_id.clone())
        {
            Some(knowledge_id) => knowledge_id,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: OPEN_WEBUI_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "Open WebUI connector health requires {OPEN_WEBUI_KNOWLEDGE_ID_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        match client.connector_health(&knowledge_id).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: OPEN_WEBUI_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: OPEN_WEBUI_IMPLEMENTATION_ID.to_string(),
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

        let knowledge_id = self
            .resolve_knowledge_id_for_space(request.space_id)
            .await?;
        client
            .query_collection(
                request.space_id,
                &knowledge_id,
                &request.query,
                request.top_k,
            )
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
                    "Open WebUI read_document requires source#chunkId ids from search hits"
                        .to_string(),
                )
            })?;

        let knowledge_id = self
            .resolve_knowledge_id_for_space(request.space_id)
            .await?;
        client
            .read_chunk(request.space_id, &knowledge_id, &document_hint, &chunk_id)
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
impl ExternalKnowledgeEngine for OpenWebuiKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Open WebUI sync_sources is managed via knowledge collections UI; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
