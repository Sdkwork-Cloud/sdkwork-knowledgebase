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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEngineCapability {
    Health,
    Search,
    ReadDocument,
    ListDocuments,
    Ingest,
    SyncSources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineDescriptor {
    pub implementation_id: String,
    pub knowledge_mode: KnowledgeAgentKnowledgeMode,
    pub display_name: String,
    pub agent_provider_id: String,
    pub native: bool,
    #[serde(default)]
    pub capabilities: Vec<KnowledgeEngineCapability>,
}

impl KnowledgeEngineDescriptor {
    pub fn supports(&self, capability: KnowledgeEngineCapability) -> bool {
        self.capabilities.contains(&capability)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEngineProviderOperation {
    Health,
    Search,
    Read,
    List,
    Ingest,
    Sync,
}

impl std::fmt::Display for KnowledgeEngineProviderOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Health => "health",
            Self::Search => "search",
            Self::Read => "read",
            Self::List => "list",
            Self::Ingest => "ingest",
            Self::Sync => "sync",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEngineProviderErrorCategory {
    Authentication,
    PermissionDenied,
    RateLimited,
    Timeout,
    Unavailable,
    CircuitOpen,
    BulkheadSaturated,
    InvalidResponse,
    ResponseTooLarge,
    InvalidTarget,
    NotFound,
    Validation,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEngineProviderFailure {
    pub category: KnowledgeEngineProviderErrorCategory,
    pub operation: KnowledgeEngineProviderOperation,
    pub implementation_id: String,
    pub binding_id: Option<String>,
    pub status_code: Option<u16>,
    pub retryable: bool,
    pub retry_after_ms: Option<u64>,
    pub safe_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeEngineError {
    NotFound(String),
    Unsupported(String),
    Validation(String),
    Provider(KnowledgeEngineProviderFailure),
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
            Self::Provider(failure) => write!(
                f,
                "knowledge provider {} failed ({:?}): {}",
                failure.operation, failure.category, failure.safe_message
            ),
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
            capabilities: native_core_capabilities(),
        },
        KnowledgeAgentKnowledgeMode::Rag => KnowledgeEngineDescriptor {
            implementation_id: KnowledgeEngineId::RAG_NATIVE.to_string(),
            knowledge_mode: mode,
            display_name: "RAG (native)".to_string(),
            agent_provider_id: RAG_KNOWLEDGE_PROVIDER_ID.to_string(),
            native: true,
            capabilities: native_core_capabilities(),
        },
        KnowledgeAgentKnowledgeMode::External => KnowledgeEngineDescriptor {
            implementation_id: "engine.knowledge.external.unresolved".to_string(),
            knowledge_mode: mode,
            display_name: "External knowledge backend".to_string(),
            agent_provider_id: "provider.knowledge.external.unresolved".to_string(),
            native: false,
            capabilities: Vec::new(),
        },
    }
}

pub fn descriptor_for_external(vendor_id: &str, display_name: &str) -> KnowledgeEngineDescriptor {
    descriptor_for_external_with_capabilities(vendor_id, display_name, Vec::new())
}

pub fn descriptor_for_external_search_read(
    vendor_id: &str,
    display_name: &str,
) -> KnowledgeEngineDescriptor {
    descriptor_for_external_with_capabilities(
        vendor_id,
        display_name,
        vec![
            KnowledgeEngineCapability::Health,
            KnowledgeEngineCapability::Search,
            KnowledgeEngineCapability::ReadDocument,
        ],
    )
}

pub fn descriptor_for_external_with_capabilities(
    vendor_id: &str,
    display_name: &str,
    capabilities: Vec<KnowledgeEngineCapability>,
) -> KnowledgeEngineDescriptor {
    KnowledgeEngineDescriptor {
        implementation_id: KnowledgeEngineId::external(vendor_id).0,
        knowledge_mode: KnowledgeAgentKnowledgeMode::External,
        display_name: display_name.to_string(),
        agent_provider_id: KnowledgeEngineId::external_agent_provider(vendor_id),
        native: false,
        capabilities,
    }
}

fn native_core_capabilities() -> Vec<KnowledgeEngineCapability> {
    vec![
        KnowledgeEngineCapability::Health,
        KnowledgeEngineCapability::Search,
        KnowledgeEngineCapability::ReadDocument,
        KnowledgeEngineCapability::ListDocuments,
    ]
}

/// Parses external engine search/read refs shaped as `{parentDocumentId}#{segmentOrChunkId}`.
pub fn parse_compound_document_ref(document_id: &str) -> Option<(String, String)> {
    let (parent_id, child_id) = document_id.split_once('#')?;
    if parent_id.is_empty() || child_id.is_empty() {
        return None;
    }
    Some((parent_id.to_string(), child_id.to_string()))
}
