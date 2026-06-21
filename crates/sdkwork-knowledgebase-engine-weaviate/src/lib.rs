//! Weaviate external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/weaviate/engine.manifest.json`
//! Handlers MUST NOT call Weaviate HTTP directly; only this adapter crate may integrate upstream APIs.

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

pub use client::WeaviateApiClient;
pub use config::{
    class_name_from_connector_metadata, WeaviateConnectorConfig, DEFAULT_WEAVIATE_CONTENT_PROPERTY,
    DEFAULT_WEAVIATE_TITLE_PROPERTY, WEAVIATE_API_KEY_ENV, WEAVIATE_BASE_URL_ENV,
    WEAVIATE_CLASS_NAME_ENV, WEAVIATE_CONTENT_PROPERTY_ENV, WEAVIATE_TITLE_PROPERTY_ENV,
};

pub const WEAVIATE_VENDOR_ID: &str = "weaviate";
pub const WEAVIATE_IMPLEMENTATION_ID: &str = "engine.knowledge.external.weaviate";
pub const WEAVIATE_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.weaviate";

pub struct WeaviateKnowledgeEngine {
    config: Option<WeaviateConnectorConfig>,
    client: Option<WeaviateApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl WeaviateKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = WeaviateConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| WeaviateApiClient::new(value.clone()));
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
        config: WeaviateConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = WeaviateApiClient::new(config.clone());
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
            "Weaviate (external adapter)"
        } else {
            "Weaviate (external adapter — unconfigured)"
        };
        descriptor_for_external(WEAVIATE_VENDOR_ID, display_name)
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Weaviate adapter requires {WEAVIATE_BASE_URL_ENV}; optional auth via {WEAVIATE_API_KEY_ENV}; class via {WEAVIATE_CLASS_NAME_ENV} or kb_source connector metadata datasetId"
        )
    }

    async fn resolve_class_name_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_class_name.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Weaviate search requires {WEAVIATE_CLASS_NAME_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            WEAVIATE_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_class_name.clone()),
            &format!("{WEAVIATE_CLASS_NAME_ENV}"),
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for WeaviateKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: WEAVIATE_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        match client.connector_health().await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: WEAVIATE_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: WEAVIATE_IMPLEMENTATION_ID.to_string(),
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

        let class_name = self.resolve_class_name_for_space(request.space_id).await?;
        client
            .near_text_search(request.space_id, &class_name, &request.query, request.top_k)
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

        let (_, object_id) =
            parse_compound_document_ref(&request.document_id).ok_or_else(|| {
                KnowledgeEngineError::Validation(
                    "Weaviate read_document requires title#objectId ids from search hits"
                        .to_string(),
                )
            })?;

        let class_name = self.resolve_class_name_for_space(request.space_id).await?;
        client.get_object(&class_name, &object_id).await
    }

    async fn list_documents(
        &self,
        request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        let class_name = self.resolve_class_name_for_space(request.space_id).await?;

        Ok(KnowledgeEngineDocumentList {
            items: vec![KnowledgeEngineDocumentRef {
                document_id: format!("{}/{}", request.space_id, class_name),
                title: class_name.clone(),
                source_uri: Some(format!("weaviate://class/{class_name}")),
            }],
        })
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for WeaviateKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Weaviate sync_sources is managed via object ingest APIs; adapter exposes search/read/list only"
                .to_string(),
        ))
    }
}
