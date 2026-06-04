use serde::{Deserialize, Serialize};

pub use crate::enums::{WikiCandidateType, WikiLogEventType, WikiPageType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmWikiPaths {
    pub agents_md: &'static str,
    pub claude_md: &'static str,
    pub schema_yaml: &'static str,
    pub index_md: &'static str,
    pub log_md: &'static str,
    pub local_mirror_agents_md: &'static str,
    pub local_mirror_schema_root: &'static str,
    pub local_mirror_raw_root: &'static str,
    pub local_mirror_wiki_root: &'static str,
}

impl Default for LlmWikiPaths {
    fn default() -> Self {
        Self {
            agents_md: "wiki/schema/AGENTS.md",
            claude_md: "wiki/schema/CLAUDE.md",
            schema_yaml: "wiki/schema/wiki_schema.yaml",
            index_md: "wiki/index.md",
            log_md: "wiki/log.md",
            local_mirror_agents_md: "AGENTS.md",
            local_mirror_schema_root: "schema/",
            local_mirror_raw_root: "raw/",
            local_mirror_wiki_root: "wiki/",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageSummary {
    pub title: String,
    pub slug: String,
    pub page_type: WikiPageType,
    pub logical_path: String,
    pub summary: String,
    pub source_count: u32,
    pub updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiPagePublishState {
    Draft,
    CandidateReady,
    NeedsReview,
    Published,
    Stale,
    Rejected,
    Failed,
}

impl WikiPagePublishState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::CandidateReady => "candidate_ready",
            Self::NeedsReview => "needs_review",
            Self::Published => "published",
            Self::Stale => "stale",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiRevisionReviewState {
    Pending,
    Approved,
    Rejected,
}

impl WikiRevisionReviewState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeWikiPage {
    pub id: u64,
    pub space_id: u64,
    pub slug: String,
    pub title: String,
    pub page_type: WikiPageType,
    pub logical_path: String,
    pub summary: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub current_revision_id: Option<u64>,
    pub publish_state: WikiPagePublishState,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeWikiPageRevision {
    pub id: u64,
    pub page_id: u64,
    pub revision_no: u64,
    pub markdown_object_ref_id: u64,
    pub content_hash: String,
    pub review_state: WikiRevisionReviewState,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeWikiPagePublication {
    pub page: KnowledgeWikiPage,
    pub revision: KnowledgeWikiPageRevision,
    pub current_file_path: String,
    pub revision_file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishKnowledgeWikiPageRequest {
    pub space_id: u64,
    pub slug: String,
    pub title: String,
    pub page_type: WikiPageType,
    pub summary: String,
    pub markdown: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub actor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLogEntry {
    pub occurred_at: String,
    pub event_type: WikiLogEventType,
    pub title: String,
    pub actor: String,
    pub affected_pages: Vec<String>,
    pub audit_event_id: Option<String>,
    pub warnings: Vec<String>,
}
