use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiPageType {
    Source,
    Entity,
    Topic,
    Concept,
    HowTo,
    Reference,
    Faq,
    Glossary,
    Answer,
    Comparison,
    Presentation,
    Chart,
    Index,
    Policy,
    Runbook,
}

impl WikiPageType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Entity => "entity",
            Self::Topic => "topic",
            Self::Concept => "concept",
            Self::HowTo => "how_to",
            Self::Reference => "reference",
            Self::Faq => "faq",
            Self::Glossary => "glossary",
            Self::Answer => "answer",
            Self::Comparison => "comparison",
            Self::Presentation => "presentation",
            Self::Chart => "chart",
            Self::Index => "index",
            Self::Policy => "policy",
            Self::Runbook => "runbook",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiCandidateType {
    SourceSummary,
    PageCreate,
    PageUpdate,
    QueryAnswer,
    Comparison,
    Presentation,
    Chart,
    SchemaUpdate,
    IndexRebuild,
}

impl WikiCandidateType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceSummary => "source_summary",
            Self::PageCreate => "page_create",
            Self::PageUpdate => "page_update",
            Self::QueryAnswer => "query_answer",
            Self::Comparison => "comparison",
            Self::Presentation => "presentation",
            Self::Chart => "chart",
            Self::SchemaUpdate => "schema_update",
            Self::IndexRebuild => "index_rebuild",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiLogEventType {
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

impl WikiLogEventType {
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
