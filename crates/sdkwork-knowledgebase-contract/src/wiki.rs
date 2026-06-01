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
    pub summary: String,
    pub source_count: u32,
    pub updated_at: String,
    pub tags: Vec<String>,
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
