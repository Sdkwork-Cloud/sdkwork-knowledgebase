//! Agent provider adapter for SDKWork Knowledgebase.

pub mod agent_implementation;
pub mod agent_runtime;
pub mod claw_router;
pub mod claw_router_embeddings;
pub mod client;
pub mod knowledge_access;
pub mod llm_wiki;
mod mapper;
pub mod provider;
pub mod retrieval_plan;

pub use agent_implementation::{
    default_profile_agent_implementation_id, is_rig_agent_implementation,
    resolve_model_provider_for_implementation, resolve_rig_model_provider_id,
    validate_registered_agent_implementation, CONTRACT_MODEL_PROVIDER_ID,
};
pub use agent_runtime::{build_knowledge_agent_runtime, KnowledgeAgentRuntimeBuildRequest};
pub use claw_router::{
    is_rig_model_provider, resolve_claw_router_client_from_env, ClawRouterChatModelProvider,
    CLAW_ROUTER_CHAT_COMPLETION_METHOD, CLAW_ROUTER_OPEN_HTTP_URL_ENV, CLAW_ROUTER_OPEN_SDK_CRATE,
    DEFAULT_CLAW_ROUTER_UPSTREAM_MODEL_ID, RIG_DEFAULT_MODEL_ID, RIG_MODEL_PROVIDER_ID,
};
pub use claw_router_embeddings::{
    cosine_similarity, deserialize_embedding_vector, serialize_embedding_vector,
    ClawRouterEmbeddingClient, CLAW_ROUTER_EMBEDDINGS_METHOD,
    DEFAULT_CLAW_ROUTER_EMBEDDING_MODEL_ID,
};
pub use client::KnowledgebaseRetrievalClient;
pub use knowledge_access::{
    default_top_k, enabled_bindings, resolve_chat_knowledge_mode, validate_bindings_support_mode,
    validate_rag_profile_requirements, KnowledgeAccessGateway, KnowledgeAccessRequest,
    KnowledgeAccessResult, KnowledgeAccessRetrievalExecutor, KnowledgeRetrievalPlanResolver,
    KnowledgeSpaceModeResolver,
};
pub use llm_wiki::{
    citations_from_rag_hits, citations_from_wiki_pages, wiki_document_id, LlmWikiKnowledgeClient,
    LlmWikiKnowledgeProvider, LLM_WIKI_KNOWLEDGE_PROVIDER_ID,
};
pub use provider::{SdkworkKnowledgebaseProvider, SDKWORK_KNOWLEDGEBASE_PROVIDER_ID};
pub use retrieval_plan::{
    default_rag_methods, kernel_methods_for_retrieval, merge_retrieval_plan,
    retrieval_methods_for_strategy, KnowledgeRetrievalPlan,
};
