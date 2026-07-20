//! Haystack external knowledge engine adapter (`integrationTier: adapter`).
//!
//! Vendor catalog: `external/knowledge-engines/vendors/haystack/engine.manifest.json`
//! Handlers MUST NOT call Haystack HTTP directly; only this adapter crate may integrate upstream APIs.

mod client;
mod config;

use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::knowledge_engine::{
    resolve_connector_dataset_id_for_space, resolve_connector_workspace_slug_for_space,
    KnowledgeEngine,
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

pub use client::{chunk_id_from_content, HaystackApiClient};
pub use config::{
    pipeline_name_from_connector_metadata, workspace_name_from_connector_metadata,
    HaystackConnectorConfig, HaystackDeploymentMode, HAYSTACK_BASE_URL_ENV,
    HAYSTACK_CREDENTIAL_ENV, HAYSTACK_CREDENTIAL_FILE_ENV, HAYSTACK_DEPLOYMENT_MODE_ENV,
    HAYSTACK_PIPELINE_ENV, HAYSTACK_QUERY_FIELD_ENV, HAYSTACK_WORKSPACE_ENV,
};

pub const HAYSTACK_VENDOR_ID: &str = "haystack";
pub const HAYSTACK_IMPLEMENTATION_ID: &str = "engine.knowledge.external.haystack";
pub const HAYSTACK_AGENT_PROVIDER_ID: &str = "provider.knowledge.external.haystack";

pub struct HaystackKnowledgeEngine {
    config: Option<HaystackConnectorConfig>,
    client: Option<HaystackApiClient>,
    source_store: Option<Arc<dyn KnowledgeSourceStore>>,
}

impl HaystackKnowledgeEngine {
    pub fn from_env() -> Self {
        let config = HaystackConnectorConfig::from_env();
        let client = config
            .as_ref()
            .map(|value| HaystackApiClient::new(value.clone()));
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
        config: HaystackConnectorConfig,
        source_store: Option<Arc<dyn KnowledgeSourceStore>>,
    ) -> Self {
        let client = HaystackApiClient::new(config.clone());
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
            "Haystack (external adapter)"
        } else {
            "Haystack (external adapter — unconfigured)"
        };
        if self.config.is_some() {
            descriptor_for_external_search_read(HAYSTACK_VENDOR_ID, display_name)
        } else {
            descriptor_for_external(HAYSTACK_VENDOR_ID, display_name)
        }
    }

    fn unconfigured_message(&self) -> String {
        format!(
            "Haystack adapter requires {HAYSTACK_BASE_URL_ENV}; optional auth via {HAYSTACK_CREDENTIAL_ENV} or {HAYSTACK_CREDENTIAL_FILE_ENV}; pipeline via {HAYSTACK_PIPELINE_ENV} or kb_source connector metadata datasetId; workspace via {HAYSTACK_WORKSPACE_ENV} or connector metadata workspaceSlug for Deepset Cloud"
        )
    }

    async fn resolve_pipeline_for_space(
        &self,
        space_id: u64,
    ) -> Result<String, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return self
                .config
                .as_ref()
                .and_then(|config| config.default_pipeline.clone())
                .ok_or_else(|| {
                    KnowledgeEngineError::Validation(format!(
                        "Haystack search requires {HAYSTACK_PIPELINE_ENV} or kb_source connector metadata datasetId for space_id={space_id}"
                    ))
                });
        };

        resolve_connector_dataset_id_for_space(
            source_store,
            space_id,
            HAYSTACK_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_pipeline.clone()),
            HAYSTACK_PIPELINE_ENV,
        )
        .await
    }

    async fn resolve_workspace_for_space(
        &self,
        space_id: u64,
    ) -> Result<Option<String>, KnowledgeEngineError> {
        let Some(source_store) = self.source_store.as_deref() else {
            return Ok(self
                .config
                .as_ref()
                .and_then(|config| config.default_workspace.clone()));
        };

        match resolve_connector_workspace_slug_for_space(
            source_store,
            space_id,
            HAYSTACK_IMPLEMENTATION_ID,
            self.config
                .as_ref()
                .and_then(|config| config.default_workspace.clone()),
            HAYSTACK_WORKSPACE_ENV,
        )
        .await
        {
            Ok(workspace) => Ok(Some(workspace)),
            Err(KnowledgeEngineError::Validation(_)) => Ok(self
                .config
                .as_ref()
                .and_then(|config| config.default_workspace.clone())),
            Err(error) => Err(error),
        }
    }
}

#[async_trait]
impl KnowledgeEngine for HaystackKnowledgeEngine {
    fn descriptor(&self) -> KnowledgeEngineDescriptor {
        self.descriptor_value()
    }

    async fn health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        let Some(client) = self.client.as_ref() else {
            return Ok(KnowledgeEngineHealth {
                implementation_id: HAYSTACK_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Degraded,
                detail: Some(self.unconfigured_message()),
            });
        };

        let pipeline = match self
            .config
            .as_ref()
            .and_then(|config| config.default_pipeline.clone())
        {
            Some(pipeline) => pipeline,
            None => {
                return Ok(KnowledgeEngineHealth {
                    implementation_id: HAYSTACK_IMPLEMENTATION_ID.to_string(),
                    status: KnowledgeEngineHealthStatus::Degraded,
                    detail: Some(format!(
                        "Haystack connector health requires {HAYSTACK_PIPELINE_ENV} or per-space kb_source connector metadata datasetId"
                    )),
                });
            }
        };

        let workspace = self
            .config
            .as_ref()
            .and_then(|config| config.default_workspace.clone());

        match client
            .connector_health(workspace.as_deref(), &pipeline)
            .await
        {
            Ok(()) => Ok(KnowledgeEngineHealth {
                implementation_id: HAYSTACK_IMPLEMENTATION_ID.to_string(),
                status: KnowledgeEngineHealthStatus::Available,
                detail: None,
            }),
            Err(error) => Ok(KnowledgeEngineHealth {
                implementation_id: HAYSTACK_IMPLEMENTATION_ID.to_string(),
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

        let pipeline = self.resolve_pipeline_for_space(request.space_id).await?;
        let workspace = self.resolve_workspace_for_space(request.space_id).await?;
        client
            .search(
                request.space_id,
                workspace.as_deref(),
                &pipeline,
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
                    "Haystack read_document requires title#documentId ids from search hits"
                        .to_string(),
                )
            })?;

        let pipeline = self.resolve_pipeline_for_space(request.space_id).await?;
        let workspace = self.resolve_workspace_for_space(request.space_id).await?;
        client
            .read_document(
                request.space_id,
                workspace.as_deref(),
                &pipeline,
                &document_hint,
                &chunk_id,
            )
            .await
    }

    async fn list_documents(
        &self,
        _request: KnowledgeEngineListRequest,
    ) -> Result<KnowledgeEngineDocumentList, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Haystack adapter does not expose a document enumeration API".to_string(),
        ))
    }
}

#[async_trait]
impl ExternalKnowledgeEngine for HaystackKnowledgeEngine {
    async fn connector_health(&self) -> Result<KnowledgeEngineHealth, KnowledgeEngineError> {
        self.health().await
    }

    async fn sync_sources(&self, _space_id: u64) -> Result<u32, KnowledgeEngineError> {
        Err(KnowledgeEngineError::Unsupported(
            "Haystack sync_sources is managed via pipeline deployment; adapter exposes search/read only"
                .to_string(),
        ))
    }
}
