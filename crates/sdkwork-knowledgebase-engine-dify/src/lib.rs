//! Dify external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/dify/engine.manifest.json`
//! Handlers MUST NOT call Dify HTTP directly; only this adapter crate may integrate upstream APIs.

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

pub use client::DifyApiClient;
pub use config::{
    dataset_id_from_connector_metadata, DifyConnectorConfig, DIFY_BASE_URL_ENV,
    DIFY_CREDENTIAL_ENV, DIFY_CREDENTIAL_FILE_ENV, DIFY_DATASET_ID_ENV,
};

pub const DIFY_VENDOR_ID: &str = "dify";
pub const DIFY_IMPLEMENTATION_ID: &str = "engine.knowledge.external.dify";
pub const DIFY_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.dify";

pub struct DifyKnowledgeEngine {
    config: Option<DifyConnectorConfig>,
    client: Option<DifyApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl DifyKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = DifyConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| DifyApiClient::new(value.clone()));
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
        config: DifyConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = DifyApiClient::new(config.clone());
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
            "Dify (external adapter)"
        } else {
            "Dify (external adapter — unconfigured)"
        };
        descriptor_for_external(DIFY_VENDOR_ID, display_name)
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Dify adapter requires {DIFY_BASE_URL_ENV} and {DIFY_CREDENTIAL_ENV}; optional default dataset via {DIFY_DATASET_ID_ENV} or kb_source connector metadata datasetId"
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
                        "Dify search requires {DIFY_DATASET_ID_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            DIFY_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_dataset_id.clone()),
            DIFY_DATASET_ID_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for DifyKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: DIFY_IMPLEMENTATION_ID.to_string(),
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
                    implementation_id: DIFY_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "Dify connector health requires {DIFY_DATASET_ID_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        match client.connector_health(&dataset_id).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: DIFY_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: DIFY_IMPLEMENTATION_ID.to_string(),
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

        let (document_id, segment_id) = parse_compound_document_ref(&request.document_id)
            .ok_or_else(|| {
                KnowledgeEngineError::Validation(
                    "Dify read_document requires parentDocument#segmentId ids from search hits"
                        .to_string(),
                )
            })?;

        let dataset_id = self.resolve_dataset_id_for_space(request.space_id).await?;
        client
            .read_segment(&dataset_id, &document_id, &segment_id)
            .await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "adapter-tier list_documents is not implemented; use search hits or native ingestion"
                .to_string(),
        ))
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for DifyKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Dify sync_sources is not implemented at adapter tier; use Dify console ingestion"
                .to_string(),
        ))
    }
}
