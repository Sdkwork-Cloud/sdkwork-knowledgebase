//! Chroma external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/chroma/engine.manifest.json`
//! Handlers MUST NOT call Chroma HTTP directly; only this adapter crate may integrate upstream APIs.

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

pub use client::ChromaApiClient;
pub use config::{
    collection_id_from_connector_metadata, ChromaConnectorConfig, CHROMA_BASE_URL_ENV,
    CHROMA_COLLECTION_ID_ENV, CHROMA_CREDENTIAL_ENV, CHROMA_CREDENTIAL_FILE_ENV,
    CHROMA_DATABASE_ENV, CHROMA_TENANT_ENV, DEFAULT_CHROMA_DATABASE, DEFAULT_CHROMA_TENANT,
};

pub const CHROMA_VENDOR_ID: &str = "chroma";
pub const CHROMA_IMPLEMENTATION_ID: &str = "engine.knowledge.external.chroma";
pub const CHROMA_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.chroma";

pub struct ChromaKnowledgeEngine {
    config: Option<ChromaConnectorConfig>,
    client: Option<ChromaApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl ChromaKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = ChromaConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| ChromaApiClient::new(value.clone()));
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
        config: ChromaConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = ChromaApiClient::new(config.clone());
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
            "Chroma (external adapter)"
        } else {
            "Chroma (external adapter — unconfigured)"
        };
        if self.config.is_some() {
            descriptor_for_external_search_read(CHROMA_VENDOR_ID, display_name)
        } else {
            descriptor_for_external(CHROMA_VENDOR_ID, display_name)
        }
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Chroma adapter requires {CHROMA_BASE_URL_ENV}; optional auth via {CHROMA_CREDENTIAL_ENV} or {CHROMA_CREDENTIAL_FILE_ENV}; collection via {CHROMA_COLLECTION_ID_ENV} or kb_source connector metadata datasetId"
        )
    }

    async fn resolve_collection_id_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_collection_id.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Chroma search requires {CHROMA_COLLECTION_ID_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            CHROMA_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_collection_id.clone()),
            CHROMA_COLLECTION_ID_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for ChromaKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    fn bind_provider(
        &self,
        binding: &sdkwork_knowledgebase_contract::provider_binding::KnowledgeEngineProviderBinding,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        if binding.implementation_id != CHROMA_IMPLEMENTATION_ID {
            return Err(KnowledgeEngineError::Validation(
                "Chroma cannot bind a different Provider implementation".to_string(),
            ));
        }
        let mut config = self
            .config
            .clone()
            .ok_or_else(|| KnowledgeEngineError::Unsupported(self.unconfigured_message()))?;
        config.default_collection_id = Some(binding.remote_resource_id.clone());
        Ok(Arc::new(Self::with_config(config, None)))
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: CHROMA_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        match client.connector_health().await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: CHROMA_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: CHROMA_IMPLEMENTATION_ID.to_string(),
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

        let collection_id = self
            .resolve_collection_id_for_space(request.space_id)
            .await?;
        client
            .query_collection(
                request.space_id,
                &collection_id,
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

        let (_, record_id) =
            parse_compound_document_ref(&request.document_id).ok_or_else(|| {
                KnowledgeEngineError::Validation(
                    "Chroma read_document requires title#recordId ids from search hits".to_string(),
                )
            })?;

        let collection_id = self
            .resolve_collection_id_for_space(request.space_id)
            .await?;
        client.get_record(&collection_id, &record_id).await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Chroma adapter does not expose a document enumeration API".to_string(),
        ))
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for ChromaKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Chroma sync_sources is managed via collection ingest APIs; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
