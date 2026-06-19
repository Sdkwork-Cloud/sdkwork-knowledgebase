use crate::serde_int64::{
    deserialize_option_u64_from_string_or_number, deserialize_u64_from_string_or_number,
    serialize_option_u64_as_string, serialize_u64_as_string,
};
use serde::{Deserialize, Serialize};

pub use crate::rag::KnowledgeAgentKnowledgeMode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentChatRequest {
    #[serde(
        default,
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
    pub message: String,
    #[serde(default)]
    pub mode: Option<KnowledgeAgentKnowledgeMode>,
    pub session_id: Option<String>,
    pub model_provider_id: Option<String>,
    pub model_id: Option<String>,
    pub agent_implementation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentChatCitation {
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub document_id: Option<u64>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub wiki_page_id: Option<u64>,
    pub title: String,
    pub source_uri: Option<String>,
    pub logical_path: Option<String>,
    pub locator: Option<String>,
    pub score: Option<f64>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeAgentChatResponse {
    pub chat_id: String,
    pub answer: String,
    pub mode: KnowledgeAgentKnowledgeMode,
    pub agent_implementation_id: String,
    pub model_provider_id: String,
    pub model_id: String,
    pub citations: Vec<KnowledgeAgentChatCitation>,
    #[serde(
        default,
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub retrieval_id: Option<u64>,
    pub session_id: Option<String>,
}

impl KnowledgeAgentChatRequest {
    pub fn with_tenant_id(mut self, tenant_id: u64) -> Self {
        self.tenant_id = tenant_id;
        self
    }
}
