use sdkwork_knowledgebase_contract::knowledge_engine::{
    implementation_id_from_provider, KnowledgeEngineCapability, KnowledgeEngineError,
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

        self.registry.resolve_for_mode(mode)
    }

    async fn resolve_external_for_space(
        &self,
        space_id: u64,
    ) -> Result<Arc<dyn KnowledgeEngine>, KnowledgeEngineError> {
        let sources = self
            .source_store
            .list_sources_for_space(space_id)
            .await
            .map_err(|error| KnowledgeEngineError::Internal(error.to_string()))?;

        let mut resolved = Vec::new();
        let mut resolved_ids = std::collections::HashSet::new();
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
                if !engine
                    .descriptor()
                    .supports(KnowledgeEngineCapability::Search)
                {
                    continue;
                }
                if resolved_ids.insert(implementation_id.clone()) {
                    resolved.push((implementation_id, engine));
                }
            }
        }

        match resolved.len() {
            0 => Err(KnowledgeEngineError::NotFound(format!(
                "no external knowledge engine registered for space_id={space_id}"
            ))),
            1 => Ok(resolved.pop().expect("length checked").1),
            _ => {
                let mut implementation_ids = resolved_ids.into_iter().collect::<Vec<_>>();
                implementation_ids.sort();
                Err(KnowledgeEngineError::Validation(format!(
                    "multiple external knowledge engines are configured for space_id={space_id}: {}; explicit provider binding is required",
                    implementation_ids.join(",")
                )))
            }
        }
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
