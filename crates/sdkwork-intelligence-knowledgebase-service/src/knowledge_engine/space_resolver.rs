use sdkwork_knowledgebase_contract::knowledge_engine::{
    KnowledgeEngineCapability, KnowledgeEngineError,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use std::sync::Arc;

use crate::ports::knowledge_engine::{
    KnowledgeEngine, KnowledgeEngineRegistry, KnowledgeEngineSpaceRegistry,
};
use crate::ports::knowledge_provider_binding_store::{
    KnowledgeEngineProviderBindingStore, KnowledgeEngineProviderScope,
};
use crate::ports::knowledge_space_store::KnowledgeSpaceStore;

pub struct KnowledgeEngineSpaceResolver<R> {
    registry: Arc<R>,
    space_store: Arc<dyn KnowledgeSpaceStore>,
    provider_binding_store: Arc<dyn KnowledgeEngineProviderBindingStore>,
    provider_scope: KnowledgeEngineProviderScope,
}

impl<R> KnowledgeEngineSpaceResolver<R>
where
    R: KnowledgeEngineRegistry + 'static,
{
    pub fn new(
        registry: Arc<R>,
        space_store: Arc<dyn KnowledgeSpaceStore>,
        provider_binding_store: Arc<dyn KnowledgeEngineProviderBindingStore>,
        provider_scope: KnowledgeEngineProviderScope,
    ) -> Self {
        Self {
            registry,
            space_store,
            provider_binding_store,
            provider_scope,
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

        self.registry.resolve_for_mode(mode)
    }

    async fn resolve_external_for_space(
        &self,
        space_id: u64,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        let binding = self
            .provider_binding_store
            .get_active_binding_for_space(self.provider_scope, space_id)
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?
            .ok_or_else(|| {
                KnowledgeEngineError::NotFound(format!(
                    "no active external Provider binding for space_id={space_id}"
                ))
            })?;
        if !binding
            .capability_snapshot
            .contains(&KnowledgeEngineCapability::Search)
        {
            return Err(KnowledgeEngineError::Unsupported(format!(
                "active Provider binding_id={} has no tested search capability",
                binding.id
            )));
        }

        let engine = self.registry.resolve_by_id(&binding.implementation_id)?;
        if !engine
            .descriptor()
            .supports(KnowledgeEngineCapability::Search)
        {
            return Err(KnowledgeEngineError::Unsupported(format!(
                "Provider implementation_id={} no longer supports search",
                binding.implementation_id
            )));
        }
        engine.bind_provider(&binding)
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
