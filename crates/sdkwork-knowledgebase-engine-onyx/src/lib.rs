//! Onyx external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/onyx/engine.manifest.json`
//! Handlers MUST NOT call Onyx HTTP directly; only this adapter crate may integrate upstream APIs.

mod client;
mod config;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_engine::ExternalKnowledgeEngine;
use sdkwork_knowledgebase_contract::knowledge_engine::{
    descriptor_for_external, KnowledgeEngineDescriptor, KnowledgeEngineDocument,
    KnowledgeEngineDocumentList, KnowledgeEngineDocumentRef, KnowledgeEngineError,
    KnowledgeEngineHealth, KnowledgeEngineHealthStatus, KnowledgeEngineListRequest,
    KnowledgeEngineReadRequest, KnowledgeEngineSearchRequest, KnowledgeEngineSearchResult,
};

pub use client::{decode_url_document_id, encode_url_document_id, OnyxApiClient};
pub use config::{OnyxConnectorConfig, ONYX_API_KEY_ENV, ONYX_BASE_URL_ENV};

pub const ONYX_VENDOR_ID: &str = "onyx";
pub const ONYX_IMPLEMENTATION_ID: &str = "engine.knowledge.external.onyx";
pub const ONYX_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.onyx";

pub struct OnyxKnowledgeEngine {
    config: Option<OnyxConnectorConfig>,
    client: Option<OnyxApiClient>,
}

impl OnyxKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = OnyxConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| OnyxApiClient::new(value.clone()));
        Self { config, client }
    }

    pub fn with_config(config: OnyxConnectorConfig) -> Self {
        let client = OnyxApiClient::new(config.clone());
        Self {
            config: Some(config),
            client: Some(client),
        }
    }

    pub fn stub() -> Self {
        Self {
            config: None,
            client: None,
        }
    }

    fn descriptor_value(&self) -> KnowledgeEngineDescriptor {
        let display_name = if self.config.is_some() {
            "Onyx (external adapter)"
        } else {
            "Onyx (external adapter — unconfigured)"
        };
        descriptor_for_external(ONYX_VENDOR_ID, display_name)
    }

    fn unconfigured_message(&self) -> String {
        format!("Onyx adapter requires {ONYX_BASE_URL_ENV} and {ONYX_API_KEY_ENV}")
    }
}

#[async_trait]
impl KnowledgeEngine for OnyxKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: ONYX_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        match client.connector_health().await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: ONYX_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: ONYX_IMPLEMENTATION_ID.to_string(),
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

        client.search(request.space_id, &request.query).await
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

        let Some(url) = decode_url_document_id(&request.document_id) else {
            return Err(KnowledgeEngineError::Validation(
                "Onyx read_document requires url:{url} document ids from search hits".to_string(),
            ));
        };

        client.read_url_document(&url).await
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
impl ExternalKnowledgeEngine for OnyxKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Onyx sync_sources is managed via Onyx connector admin; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
