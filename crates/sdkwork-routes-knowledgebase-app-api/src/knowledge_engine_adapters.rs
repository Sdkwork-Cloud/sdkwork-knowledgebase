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
    let mut engines = Vec::new();
    push_if_configured(&mut engines, DifyKnowledgeEngine::from_env());
    push_if_configured(&mut engines, RagflowKnowledgeEngine::from_env());
    push_if_configured(&mut engines, OnyxKnowledgeEngine::from_env());
    push_if_configured(&mut engines, AnythingLlmKnowledgeEngine::from_env());
    push_if_configured(&mut engines, OpenWebuiKnowledgeEngine::from_env());
    push_if_configured(&mut engines, FlowiseKnowledgeEngine::from_env());
    push_if_configured(&mut engines, ChromaKnowledgeEngine::from_env());
    push_if_configured(&mut engines, QdrantKnowledgeEngine::from_env());
    push_if_configured(&mut engines, WeaviateKnowledgeEngine::from_env());
    push_if_configured(&mut engines, HaystackKnowledgeEngine::from_env());
    engines
}

fn push_if_configured<E>(engines: &mut Vec<Arc<dyn KnowledgeEngine>>, engine: Option<E>)
where
    E: KnowledgeEngine + 'static,
{
    if let Some(engine) = engine {
        engines.push(Arc::new(engine));
    }
}
