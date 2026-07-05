//! AnythingLLM external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/anythingllm/engine.manifest.json`
//! Handlers MUST NOT call AnythingLLM HTTP directly; only this adapter crate may integrate upstream APIs.

mod client;
mod config;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    resolve_connector_workspace_slug_for_space, KnowledgeEngine,
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

pub use client::AnythingLlmApiClient;
pub use config::{
    workspace_slug_from_connector_metadata, AnythingLlmConnectorConfig, ANYTHINGLLM_BASE_URL_ENV,
    ANYTHINGLLM_CREDENTIAL_ENV, ANYTHINGLLM_CREDENTIAL_FILE_ENV, ANYTHINGLLM_WORKSPACE_SLUG_ENV,
};

pub const ANYTHINGLLM_VENDOR_ID: &str = "anythingllm";
pub const ANYTHINGLLM_IMPLEMENTATION_ID: &str = "engine.knowledge.external.anythingllm";
pub const ANYTHINGLLM_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.anythingllm";

pub struct AnythingLlmKnowledgeEngine {
    config: Option<AnythingLlmConnectorConfig>,
    client: Option<AnythingLlmApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl AnythingLlmKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = AnythingLlmConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| AnythingLlmApiClient::new(value.clone()));
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
        config: AnythingLlmConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = AnythingLlmApiClient::new(config.clone());
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
            "AnythingLLM (external adapter)"
        } else {
            "AnythingLLM (external adapter — unconfigured)"
        };
        descriptor_for_external(ANYTHINGLLM_VENDOR_ID, display_name)
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "AnythingLLM adapter requires {ANYTHINGLLM_BASE_URL_ENV} and {ANYTHINGLLM_CREDENTIAL_ENV} or {ANYTHINGLLM_CREDENTIAL_FILE_ENV}; optional default workspace via {ANYTHINGLLM_WORKSPACE_SLUG_ENV} or kb_source connector metadata workspaceSlug"
        )
    }

    async fn resolve_workspace_slug_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_workspace_slug.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "AnythingLLM search requires {ANYTHINGLLM_WORKSPACE_SLUG_ENV} or kb_source connector metadata workspaceSlug for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_workspace_slug_for_space(
            source_store,
            space_id,
            ANYTHINGLLM_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_workspace_slug.clone()),
            ANYTHINGLLM_WORKSPACE_SLUG_ENV,
        )
        .await
    }
}

#[async_trait]
impl KnowledgeEngine for AnythingLlmKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: ANYTHINGLLM_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let workspace_slug = match self
            .config
            .as_ref()
            .and_then(|config| config.default_workspace_slug.clone())
        {
            Some(workspace_slug) => workspace_slug,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: ANYTHINGLLM_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "AnythingLLM connector health requires {ANYTHINGLLM_WORKSPACE_SLUG_ENV} or per-space kb_source connector metadata workspaceSlug"
                    )),
                });
            }
        };

        match client.connector_health(&workspace_slug).await {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: ANYTHINGLLM_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: ANYTHINGLLM_IMPLEMENTATION_ID.to_string(),
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

        let workspace_slug = self
            .resolve_workspace_slug_for_space(request.space_id)
            .await?;
        client
            .vector_search(
                request.space_id,
                &workspace_slug,
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
                    "AnythingLLM read_document requires title#chunkId ids from search hits"
                        .to_string(),
                )
            })?;

        let workspace_slug = self
            .resolve_workspace_slug_for_space(request.space_id)
            .await?;
        client
            .read_chunk(request.space_id, &workspace_slug, &document_hint, &chunk_id)
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
impl ExternalKnowledgeEngine for AnythingLlmKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "AnythingLLM sync_sources is managed via workspace ingestion UI; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
