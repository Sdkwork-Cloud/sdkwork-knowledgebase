use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OkfBundleFileKind {
    BundleProfile,
    BundleAgents,
    BundleIndex,
    BundleLog,
    ConceptRevision,
    GraphExport,
    ContextPack,
    OutputExport,
}

impl OkfBundleFileKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BundleProfile => "bundle_profile",
            Self::BundleAgents => "bundle_agents",
            Self::BundleIndex => "bundle_index",
            Self::BundleLog => "bundle_log",
            Self::ConceptRevision => "concept_revision",
            Self::GraphExport => "graph_export",
            Self::ContextPack => "context_pack",
            Self::OutputExport => "output_export",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OkfCandidateType {
    SourceSummary,
    ConceptCreate,
    ConceptUpdate,
    QueryAnswer,
    Comparison,
    Presentation,
    Chart,
    ProfileUpdate,
    IndexRebuild,
}

impl OkfCandidateType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceSummary => "source_summary",
            Self::ConceptCreate => "concept_create",
            Self::ConceptUpdate => "concept_update",
            Self::QueryAnswer => "query_answer",
            Self::Comparison => "comparison",
            Self::Presentation => "presentation",
            Self::Chart => "chart",
            Self::ProfileUpdate => "profile_update",
            Self::IndexRebuild => "index_rebuild",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OkfLogEventType {
    Ingest,
    Query,
    FiledAnswer,
    Compile,
    Review,
    Publish,
    Lint,
    Eval,
    Package,
    Mirror,
    DeltaUpdate,
}

impl OkfLogEventType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ingest => "ingest",
            Self::Query => "query",
            Self::FiledAnswer => "filed_answer",
            Self::Compile => "compile",
            Self::Review => "review",
            Self::Publish => "publish",
            Self::Lint => "lint",
            Self::Eval => "eval",
            Self::Package => "package",
            Self::Mirror => "mirror",
            Self::DeltaUpdate => "delta_update",
        }
    }
}
