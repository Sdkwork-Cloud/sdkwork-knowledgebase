//! Knowledge Engine SPI contract types.
//!
//! Product-level switchable backends (native OKF, native RAG, third-party) are
//! identified here. Implementation traits live in
//! `sdkwork-intelligence-knowledgebase-service::ports::knowledge_engine`.

use serde::{Deserialize, Serialize};

use crate::rag::KnowledgeAgentKnowledgeMode;

pub const RAG_KNOWLEDGE_PROVIDER_ID: &str = "provider.knowledge.sdkwork-knowledgebase";

/// Stable implementation id for a registered knowledge engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeEngineId(pub String);

impl KnowledgeEngineId {
    pub const OKF_NATIVE: &'static str = "engine.knowledge.okf.native";
    pub const RAG_NATIVE: &'static str = "engine.knowledge.rag.native";

    pub fn okf_native() -> Self {
        Self(Self::OKF_NATIVE.to_string())
    }

    pub fn rag_native() -> Self {
        Self(Self::RAG_NATIVE.to_string())
    }

    pub fn external(vendor: &str) -> Self {
        Self(format!("engine.knowledge.external.{vendor}"))
    }

    pub fn external_agent_provider(vendor: &str) -> String {
        format!("provider.knowledge.external.{vendor}")
    }
}

/// Describes a resolved engine instance for observability and health surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineDescriptor {
    pub implementation_id: String,
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
    pub display_name: String,
    pub agent_provider_id: String,
    pub native: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEngineHealthStatus {
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineHealth {
    pub implementation_id: String,
    pub status: KnowledgeEngineHealthStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineSearchRequest {
    pub tenant_id: u64,
    pub space_id: u64,
    pub query: String,
    pub top_k: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineDocumentRef {
    pub document_id: String,
    pub title: String,
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineSearchHit {
    pub document: KnowledgeEngineDocumentRef,
    pub snippet: String,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineSearchResult {
    pub implementation_id: String,
    pub hits: Vec<KnowledgeEngineSearchHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineReadRequest {
    pub tenant_id: u64,
    pub space_id: u64,
    pub document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineDocument {
    pub document_id: String,
    pub title: String,
    pub content: String,
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineListRequest {
    pub tenant_id: u64,
    pub space_id: u64,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineDocumentList {
    pub items: Vec<KnowledgeEngineDocumentRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineError {
    NotFound(String),
    Unsupported(String),
    Validation(String),
    Internal(String),
}

impl std::fmt::Display for KnowledgeEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(message) => write!(f, "knowledge engine not found: {message}"),
            Self::Unsupported(message) => {
                write!(f, "knowledge engine capability unsupported: {message}")
            }
            Self::Validation(message) => write!(f, "knowledge engine validation failed: {message}"),
            Self::Internal(message) => write!(f, "knowledge engine internal error: {message}"),
        }
    }
}

impl std::error::Error for KnowledgeEngineError {}

pub fn descriptor_for_mode(mode: KnowledgeAgentKnowledgeMode) -> KnowledgeEngineDescriptor {
    match mode {
        KnowledgeAgentKnowledgeMode::OkfBundle => KnowledgeEngineDescriptor {
            implementation_id: KnowledgeEngineId::OKF_NATIVE.to_string(),
            knowledge_mode: mode,
            display_name: "OKF Bundle (native)".to_string(),
            agent_provider_id: crate::okf::OKF_KNOWLEDGE_PROVIDER_ID.to_string(),
            native: true,
        },
        KnowledgeAgentKnowledgeMode::Rag => KnowledgeEngineDescriptor {
            implementation_id: KnowledgeEngineId::RAG_NATIVE.to_string(),
            knowledge_mode: mode,
            display_name: "RAG (native)".to_string(),
            agent_provider_id: RAG_KNOWLEDGE_PROVIDER_ID.to_string(),
            native: true,
        },
        KnowledgeAgentKnowledgeMode::External => KnowledgeEngineDescriptor {
            implementation_id: "engine.knowledge.external.unresolved".to_string(),
            knowledge_mode: mode,
            display_name: "External knowledge backend".to_string(),
            agent_provider_id: "provider.knowledge.external.unresolved".to_string(),
            native: false,
        },
    }
}

/// Maps `kb_source.provider` or catalog ids to a registered engine implementation id.
pub fn implementation_id_from_provider(provider: &str) -> Option<String> {
    let trimmed = provider.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("engine.knowledge.") {
        return Some(trimmed.to_string());
    }
    if let Some(vendor) = trimmed.strip_prefix("provider.knowledge.external.") {
        return Some(KnowledgeEngineId::external(vendor).0);
    }
    if vendor_id_pattern_matches(trimmed) {
        return Some(KnowledgeEngineId::external(trimmed).0);
    }
    None
}

fn vendor_id_pattern_matches(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}

pub fn descriptor_for_external(vendor_id: &str, display_name: &str) -> KnowledgeEngineDescriptor {
    KnowledgeEngineDescriptor {
        implementation_id: KnowledgeEngineId::external(vendor_id).0,
        knowledge_mode: KnowledgeAgentKnowledgeMode::External,
        display_name: display_name.to_string(),
        agent_provider_id: KnowledgeEngineId::external_agent_provider(vendor_id),
        native: false,
    }
}

/// Parses external engine search/read refs shaped as `{parentDocumentId}#{segmentOrChunkId}`.
pub fn parse_compound_document_ref(document_id: &str) -> Option<(String, String)> {
    let (parent_id, child_id) = document_id.split_once('#')?;
    if parent_id.is_empty() || child_id.is_empty() {
        return None;
    }
    Some((parent_id.to_string(), child_id.to_string()))
}
