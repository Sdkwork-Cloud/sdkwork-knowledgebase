//! Runtime wiring for approved external knowledge engine adapter crates.

use sdkwork_intelligence_knowledgebase_service::knowledge_engine::KnowledgeEngine;
use sdkwork_knowledgebase_engine_anythingllm::AnythingLlmKnowledgeEngine;
use sdkwork_knowledgebase_engine_chroma::ChromaKnowledgeEngine;
use sdkwork_knowledgebase_engine_dify::DifyKnowledgeEngine;
use sdkwork_knowledgebase_engine_flowise::FlowiseKnowledgeEngine;
use sdkwork_knowledgebase_engine_haystack::HaystackKnowledgeEngine;
use sdkwork_knowledgebase_engine_onyx::OnyxKnowledgeEngine;
use sdkwork_knowledgebase_engine_open_webui::OpenWebuiKnowledgeEngine;
use sdkwork_knowledgebase_engine_qdrant::QdrantKnowledgeEngine;
use sdkwork_knowledgebase_engine_ragflow::RagflowKnowledgeEngine;
use sdkwork_knowledgebase_engine_weaviate::WeaviateKnowledgeEngine;
use std::sync::Arc;

pub fn load_runtime_external_adapter_engines() -> Vec<Arc<dyn KnowledgeEngine>> {
    vec![
        Arc::new(DifyKnowledgeEngine::from_env()),
        Arc::new(RagflowKnowledgeEngine::from_env()),
        Arc::new(OnyxKnowledgeEngine::from_env()),
        Arc::new(AnythingLlmKnowledgeEngine::from_env()),
        Arc::new(OpenWebuiKnowledgeEngine::from_env()),
        Arc::new(FlowiseKnowledgeEngine::from_env()),
        Arc::new(ChromaKnowledgeEngine::from_env()),
        Arc::new(QdrantKnowledgeEngine::from_env()),
        Arc::new(WeaviateKnowledgeEngine::from_env()),
        Arc::new(HaystackKnowledgeEngine::from_env()),
    ]
}
