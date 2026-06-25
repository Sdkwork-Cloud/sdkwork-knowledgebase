use sdkwork_knowledgebase_contract::knowledge_engine::{
    implementation_id_from_provider, KnowledgeEngineError,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::source::KnowledgeSourceType;
use std::sync::Arc;

use crate::ports::knowledge_engine::{
    KnowledgeEngine, KnowledgeEngineRegistry, KnowledgeEngineSpaceRegistry,
};
use crate::ports::knowledge_source_store::KnowledgeSourceStore;
use crate::ports::knowledge_space_store::KnowledgeSpaceStore;

pub struct KnowledgeEngineSpaceResolver<R> {
    registry: Arc<R>,
    space_store: Arc<dyn KnowledgeSpaceStore>,
    source_store: Arc<dyn KnowledgeSourceStore>,
}

impl<R> KnowledgeEngineSpaceResolver<R>
where
    R: KnowledgeEngineRegistry + 'static,
{
    pub fn new(
        registry: Arc<R>,
        space_store: Arc<dyn KnowledgeSpaceStore>,
        source_store: Arc<dyn KnowledgeSourceStore>,
    ) -> Self {
        Self {
            registry,
            space_store,
            source_store,
        }
    }

    pub async fn resolve_for_space(
        &self,
        space_id: u64,
        mode_override: Option<KnowledgeAgentKnowledgeMode>,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        let space = self
            .space_store
            .get_space(space_id)
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let mode = mode_override.unwrap_or(space.knowledge_mode);
        if mode == KnowledgeAgentKnowledgeMode::External {
            return self.resolve_external_for_space(space_id).await;
        }

        if let Some(engine) = self.resolve_connector_override(space_id).await? {
            return Ok(engine);
        }

        self.registry.resolve_for_mode(mode)
    }

    async fn resolve_connector_override(
        &self,
        space_id: u64,
    ) -> Result<Option<Arc<dyn KnowledgeEngine>>, KnowledgeEngineError> {
        let sources = self
            .source_store
            .list_sources_for_space(space_id)
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        for source in sources {
            if source.source_type != KnowledgeSourceType::Connector {
                continue;
            }
            let Some(provider) = source.provider.as_deref() else {
                continue;
            };
            let Some(implementation_id) = implementation_id_from_provider(provider) else {
                continue;
            };
            if let Ok(engine) = self.registry.resolve_by_id(&implementation_id) {
                return Ok(Some(engine));
            }
        }

        Ok(None)
    }

    async fn resolve_external_for_space(
        &self,
        space_id: u64,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        self.resolve_connector_override(space_id)
            .await?
            .ok_or_else(|| {
                KnowledgeEngineError::NotFound(format!(
                    "no external knowledge engine registered for space_id={space_id}"
                ))
            })
    }
}

#[async_trait::async_trait]
impl<R> KnowledgeEngineSpaceRegistry for KnowledgeEngineSpaceResolver<R>
where
    R: KnowledgeEngineRegistry + 'static,
{
    async fn resolve_for_space(
        &self,
        space_id: u64,
        mode_override: Option<KnowledgeAgentKnowledgeMode>,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        KnowledgeEngineSpaceResolver::resolve_for_space(self, space_id, mode_override).await
    }
}
