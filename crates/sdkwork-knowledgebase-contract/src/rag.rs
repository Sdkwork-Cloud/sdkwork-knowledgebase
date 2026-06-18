use crate::serde_int64::{
    deserialize_option_u64_from_string_or_number, deserialize_u64_from_string_or_number,
    serialize_option_u64_as_string, serialize_u64_as_string,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeRetrievalMethod {
    Exact,
    Keyword,
    FullText,
    Structured,
    Graph,
    Vector,
    Hybrid,
    LlmRerank,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeAgentStatus {
    Draft,
    Active,
    Disabled,
    Archived,
}

/// How an agent or knowledge space resolves content for chat and retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeAgentKnowledgeMode {
    /// Karpathy-style llm-wiki page lookup over wiki pages and index.
    #[default]
    LlmWiki,
    /// Chunk-based hybrid / vector retrieval over indexed knowledge.
    Rag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeFilter {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalBinding {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub collection_id: Option<u64>,
    pub source_filter: Option<Vec<KnowledgeFilter>>,
    pub document_filter: Option<Vec<KnowledgeFilter>>,
    pub priority: i32,
    pub top_k: Option<u32>,
    pub min_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub actor_id: Option<u64>,
    pub query: String,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_profile_id: Option<u64>,
    pub bindings: Vec<KnowledgeRetrievalBinding>,
    #[serde(default)]
    pub methods: Vec<KnowledgeRetrievalMethod>,
    pub top_k: Option<u32>,
    pub include_citations: bool,
    pub include_trace: bool,
    pub context_budget_tokens: Option<u32>,
    #[serde(default)]
    pub metadata: Vec<KnowledgeFilter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalTrace {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub retrieval_trace_id: u64,
    pub status: String,
    pub latency_ms: Option<u64>,
    pub result_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCitation {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub document_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub document_version_id: Option<u64>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub chunk_id: Option<u64>,
    pub title: String,
    pub source_uri: Option<String>,
    pub locator: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeContextFragment {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub chunk_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub document_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub document_version_id: Option<u64>,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub collection_id: Option<u64>,
    pub title: String,
    pub content: String,
    pub score: Option<f64>,
    pub rank: u32,
    pub token_count: Option<u32>,
    pub retrieval_method: KnowledgeRetrievalMethod,
    pub citation: Option<KnowledgeCitation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeMemoryContextFragment {
    pub memory_id: String,
    pub title: Option<String>,
    pub content: String,
    pub score: Option<f64>,
    pub rank: u32,
    pub token_count: Option<u32>,
    pub source_uri: Option<String>,
    pub policy_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalResult {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub retrieval_id: u64,
    pub trace: Option<KnowledgeRetrievalTrace>,
    pub hits: Vec<KnowledgeContextFragment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeContextPackRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub actor_id: Option<u64>,
    pub query: String,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_profile_id: Option<u64>,
    pub bindings: Vec<KnowledgeRetrievalBinding>,
    pub context_budget_tokens: u32,
    pub include_citations: bool,
    #[serde(default)]
    pub memory_policy_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeContextPack {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub context_pack_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_id: Option<u64>,
    pub query: String,
    pub fragments: Vec<KnowledgeContextFragment>,
    #[serde(default)]
    pub memory_fragments: Vec<KnowledgeMemoryContextFragment>,
    pub estimated_tokens: u32,
    pub citations: Vec<KnowledgeCitation>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentBinding {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub binding_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub profile_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub collection_id: Option<u64>,
    pub source_filter: Option<Vec<KnowledgeFilter>>,
    pub document_filter: Option<Vec<KnowledgeFilter>>,
    pub priority: i32,
    pub top_k: Option<u32>,
    pub min_score: Option<f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentBindingList {
    pub items: Vec<KnowledgeAgentBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentBindingRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub profile_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub collection_id: Option<u64>,
    pub source_filter: Option<Vec<KnowledgeFilter>>,
    pub document_filter: Option<Vec<KnowledgeFilter>>,
    pub priority: i32,
    pub top_k: Option<u32>,
    pub min_score: Option<f64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentProfile {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub profile_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub system_instruction: String,
    pub model_provider_id: String,
    pub model_id: String,
    pub model_parameters: Option<String>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_profile_id: Option<u64>,
    pub citation_policy: Option<String>,
    pub memory_policy_ref: Option<String>,
    pub tool_policy_ref: Option<String>,
    pub answer_policy: Option<String>,
    #[serde(default)]
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
    pub status: KnowledgeAgentStatus,
    pub bindings: Vec<KnowledgeAgentBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentProfileRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub system_instruction: String,
    pub model_provider_id: String,
    pub model_id: String,
    pub model_parameters: Option<String>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_profile_id: Option<u64>,
    pub citation_policy: Option<String>,
    pub memory_policy_ref: Option<String>,
    pub tool_policy_ref: Option<String>,
    pub answer_policy: Option<String>,
    #[serde(default)]
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
    pub status: KnowledgeAgentStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIndexRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub collection_id: Option<u64>,
    pub index_kind: String,
    pub embedding_provider_id: Option<String>,
    pub embedding_model: Option<String>,
    pub dimension: Option<u32>,
    pub metric: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeIndex {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub index_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub space_id: u64,
    pub index_kind: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalProfile {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub retrieval_profile_id: u64,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub name: String,
    pub strategy: String,
    pub top_k: u32,
    pub min_score: Option<f64>,
    pub rerank_enabled: bool,
    pub context_budget_tokens: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalProfileRequest {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub name: String,
    pub strategy: String,
    pub top_k: u32,
    pub min_score: Option<f64>,
    pub rerank_enabled: bool,
    pub context_budget_tokens: u32,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRetrievalTraceList {
    pub items: Vec<KnowledgeRetrievalTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeProviderHealth {
    pub status: String,
    pub provider_id: String,
    pub checked_at: Option<String>,
}
