//! Runtime wiring for approved external knowledge engine adapter crates.

use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_source_store::KnowledgeSourceStore;
use sdkwork_knowledgebase_engine_dify::DifyKnowledgeEngine;
use sdkwork_knowledgebase_engine_onyx::OnyxKnowledgeEngine;
use sdkwork_knowledgebase_engine_ragflow::RagflowKnowledgeEngine;
use std::sync::Arc;

pub fn load_runtime_external_adapter_engines(
    source_store: Arc<dyn KnowledgeSourceStore>,
) -> Vec<Arc<dyn KnowledgeEngine>> {
    vec![
        Arc::new(DifyKnowledgeEngine::from_runtime(source_store.clone())),
        Arc::new(RagflowKnowledgeEngine::from_runtime(source_store.clone())),
        Arc::new(OnyxKnowledgeEngine::from_env()),
    ]
}
