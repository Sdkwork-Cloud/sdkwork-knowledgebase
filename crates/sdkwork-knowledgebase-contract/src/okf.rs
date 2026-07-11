use serde::{Deserialize, Serialize};

pub use crate::enums::{OkfBundleFileKind, OkfCandidateType, OkfLogEventType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfBundlePaths {
    pub agents_md: &'static str,
    pub profile_yaml: &'static str,
    pub index_md: &'static str,
    pub log_md: &'static str,
    pub governance_root: &'static str,
    pub local_mirror_agents_md: &'static str,
    pub local_mirror_profile: &'static str,
    pub local_mirror_raw_root: &'static str,
    pub local_mirror_bundle_root: &'static str,
}

impl Default for OkfBundlePaths {
    fn default() -> Self {
        Self {
            agents_md: "okf/schema/AGENTS.md",
            profile_yaml: "okf/schema/okf_profile.yaml",
            index_md: "okf/index.md",
            log_md: "okf/log.md",
            governance_root: ".sdkwork/governance",
            local_mirror_agents_md: "schema/AGENTS.md",
            local_mirror_profile: "schema/okf_profile.yaml",
            local_mirror_raw_root: "raw/",
            local_mirror_bundle_root: ".",
        }
    }
}

impl OkfBundlePaths {
    pub fn concept_logical_path(concept_id: &str) -> String {
        format!("okf/{concept_id}.md")
    }

    pub fn concept_id_from_logical_path(logical_path: &str) -> Option<String> {
        let path = logical_path.trim();
        let path = path.strip_prefix("okf/")?;
        let path = path.strip_suffix(".md")?;
        if path.is_empty() || path.contains("..") {
            return None;
        }
        Some(path.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfConceptSummary {
    pub title: String,
    pub concept_id: String,
    pub concept_type: String,
    pub logical_path: String,
    pub bundle_relative_path: String,
    pub description: String,
    pub source_count: u32,
    pub updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfConceptSummaryList {
    pub items: Vec<OkfConceptSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOkfConceptsQuery {
    pub space_id: u64,
    pub cursor: Option<String>,
    #[serde(rename = "page_size")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfIndexDocument {
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfLogDocument {
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfProfileDocument {
    pub agents_markdown: String,
    pub profile_yaml: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfQueryRequest {
    pub space_id: u64,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfQueryResult {
    pub answer_markdown: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfFileAnswerRequest {
    pub space_id: u64,
    pub title: String,
    pub answer_markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfContextPackRequest {
    pub space_id: u64,
    pub query: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfCompileJobRequest {
    pub space_id: u64,
    pub source_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfCandidateResult {
    pub id: u64,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfCandidateResultList {
    pub items: Vec<OkfCandidateResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfCandidateReviewRequest {
    pub reviewer_id: Option<u64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfConceptPublishRequest {
    pub publisher_id: Option<u64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfProfileRequest {
    pub space_id: u64,
    pub profile_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfIndexRebuildRequest {
    pub space_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfBundleExportRequest {
    pub space_id: u64,
    pub export_type: String,
    #[serde(default)]
    pub stage_for_import: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfBundleImportRequest {
    pub space_id: u64,
    pub import_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfBundleImportResult {
    pub imported_concept_count: u32,
    pub skipped_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfQualityRunRequest {
    pub space_id: u64,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfQualityRun {
    pub id: u64,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfBundleLintResult {
    pub conformance: String,
    pub issues: Vec<OkfLintIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfLintIssue {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub concept_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OkfConceptPublishState {
    Draft,
    CandidateReady,
    NeedsReview,
    Published,
    Stale,
    Rejected,
    Failed,
}

impl OkfConceptPublishState {
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
pub enum OkfRevisionReviewState {
    Pending,
    Approved,
    Rejected,
}

impl OkfRevisionReviewState {
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
pub struct KnowledgeOkfConcept {
    pub id: u64,
    pub space_id: u64,
    pub concept_id: String,
    pub title: String,
    pub concept_type: String,
    pub logical_path: String,
    pub bundle_relative_path: String,
    pub description: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub current_revision_id: Option<u64>,
    pub publish_state: OkfConceptPublishState,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfConceptRevision {
    pub id: u64,
    pub concept_row_id: u64,
    pub revision_no: u64,
    pub markdown_object_ref_id: u64,
    pub content_hash: String,
    pub review_state: OkfRevisionReviewState,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfConceptRevisionList {
    pub items: Vec<KnowledgeOkfConceptRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeOkfConceptPublication {
    pub concept: KnowledgeOkfConcept,
    pub revision: KnowledgeOkfConceptRevision,
    pub published_logical_path: String,
    pub governance_revision_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishKnowledgeOkfConceptRequest {
    pub space_id: u64,
    pub concept_id: String,
    pub title: String,
    pub concept_type: String,
    pub description: String,
    pub markdown: String,
    pub source_count: u32,
    pub tags: Vec<String>,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfConceptUpsertRequest {
    pub space_id: u64,
    pub concept_id: String,
    pub markdown: String,
    pub actor: String,
    pub publish: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkfLogEntry {
    pub occurred_at: String,
    pub event_type: OkfLogEventType,
    pub title: String,
    pub actor: String,
    pub affected_concepts: Vec<String>,
    pub audit_event_id: Option<String>,
    pub warnings: Vec<String>,
}

pub const OKF_KNOWLEDGE_PROVIDER_ID: &str = "provider.knowledge.okf";

pub fn okf_document_id(space_id: u64, concept_id: &str) -> String {
    format!("okf:{space_id}:{concept_id}")
}
