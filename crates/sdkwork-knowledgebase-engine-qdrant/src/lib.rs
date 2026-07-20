//! Qdrant external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/qdrant/engine.manifest.json`
//! Handlers MUST NOT call Qdrant HTTP directly; only this adapter crate may integrate upstream APIs.

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

pub use client::QdrantApiClient;
pub use config::{
    collection_name_from_connector_metadata, QdrantConnectorConfig, QDRANT_BASE_URL_ENV,
    QDRANT_COLLECTION_NAME_ENV, QDRANT_CREDENTIAL_ENV, QDRANT_CREDENTIAL_FILE_ENV,
    QDRANT_QUERY_MODEL_ENV, QDRANT_USING_VECTOR_ENV,
};

pub const QDRANT_VENDOR_ID: &str = "qdrant";
pub const QDRANT_IMPLEMENTATION_ID: &str = "engine.knowledge.external.qdrant";
pub const QDRANT_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.qdrant";

pub struct QdrantKnowledgeEngine {
    config: Option<QdrantConnectorConfig>,
    client: Option<QdrantApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl QdrantKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = QdrantConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| QdrantApiClient::new(value.clone()));
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
        config: QdrantConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = QdrantApiClient::new(config.clone());
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
            "Qdrant (external adapter)"
        } else {
            "Qdrant (external adapter — unconfigured)"
        };
        if self.config.is_some() {
            descriptor_for_external_search_read(QDRANT_VENDOR_ID, display_name)
        } else {
            descriptor_for_external(QDRANT_VENDOR_ID, display_name)
        }
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Qdrant adapter requires {QDRANT_BASE_URL_ENV}; optional auth via {QDRANT_CREDENTIAL_ENV} or {QDRANT_CREDENTIAL_FILE_ENV}; collection via {QDRANT_COLLECTION_NAME_ENV} or kb_source connector metadata datasetId; text search via {QDRANT_QUERY_MODEL_ENV}"
        )
    }

    async fn resolve_collection_name_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_collection_name.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Qdrant search requires {QDRANT_COLLECTION_NAME_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            QDRANT_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_collection_name.clone()),
            QDRANT_COLLECTION_NAME_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for QdrantKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: QDRANT_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let collection_name = match self
            .config
            .as_ref()
            .and_then(|config| config.default_collection_name.clone())
        {
            Some(collection_name) => collection_name,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: QDRANT_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "Qdrant connector health requires {QDRANT_COLLECTION_NAME_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        match client.connector_health(&collection_name).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: QDRANT_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: QDRANT_IMPLEMENTATION_ID.to_string(),
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

        let collection_name = self
            .resolve_collection_name_for_space(request.space_id)
            .await?;
        client
            .query_points(
                request.space_id,
                &collection_name,
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

        let (_, point_id) = parse_compound_document_ref(&request.document_id).ok_or_else(|| {
            KnowledgeEngineError::Validation(
                "Qdrant read_document requires title#pointId ids from search hits".to_string(),
            )
        })?;

        let collection_name = self
            .resolve_collection_name_for_space(request.space_id)
            .await?;
        client.get_point(&collection_name, &point_id).await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Qdrant adapter does not expose a document enumeration API".to_string(),
        ))
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for QdrantKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Qdrant sync_sources is managed via point upsert APIs; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
