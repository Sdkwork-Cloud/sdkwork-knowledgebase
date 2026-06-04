pub const SOURCES_LIST: &str = "sources.list";
pub const SOURCES_CREATE: &str = "sources.create";
pub const DOCUMENTS_LIST: &str = "documents.list";
pub const DOCUMENTS_CREATE: &str = "documents.create";
pub const DOCUMENTS_RETRIEVE: &str = "documents.retrieve";
pub const DOCUMENTS_VERSIONS_CREATE: &str = "documents.versions.create";
pub const DOCUMENTS_VERSIONS_LIST: &str = "documents.versions.list";
pub const DRIVE_IMPORTS_CREATE: &str = "driveImports.create";
pub const INGESTS_CREATE: &str = "ingests.create";
pub const INGESTS_RETRIEVE: &str = "ingests.retrieve";
pub const SPACES_BROWSER_LIST: &str = "spaces.browser.list";
pub const WIKI_PAGES_LIST: &str = "wiki.pages.list";
pub const WIKI_PAGES_RETRIEVE: &str = "wiki.pages.retrieve";
pub const WIKI_PAGES_REVISIONS_LIST: &str = "wiki.pages.revisions.list";
pub const WIKI_PAGES_PUBLISH: &str = "wiki.pages.publish";
pub const WIKI_INDEX_RETRIEVE: &str = "wiki.index.retrieve";
pub const WIKI_INDEX_REBUILD: &str = "wiki.index.rebuild";
pub const WIKI_LOG_RETRIEVE: &str = "wiki.log.retrieve";
pub const WIKI_LOG_ENTRIES_CREATE: &str = "wiki.log.entries.create";
pub const WIKI_SCHEMA_RETRIEVE: &str = "wiki.schema.retrieve";
pub const WIKI_SCHEMA_PROFILES_CREATE: &str = "wiki.schema.profiles.create";
pub const WIKI_SCHEMA_PROFILES_UPDATE: &str = "wiki.schema.profiles.update";
pub const WIKI_QUERIES_CREATE: &str = "wiki.queries.create";
pub const WIKI_QUERIES_FILE_ANSWER: &str = "wiki.queries.fileAnswer";
pub const WIKI_CONTEXT_PACKS_CREATE: &str = "wiki.contextPacks.create";
pub const WIKI_COMPILE_JOBS_CREATE: &str = "wiki.compileJobs.create";
pub const WIKI_CANDIDATES_LIST: &str = "wiki.candidates.list";
pub const WIKI_CANDIDATES_APPROVE: &str = "wiki.candidates.approve";
pub const WIKI_CANDIDATES_REJECT: &str = "wiki.candidates.reject";
pub const WIKI_EXPORTS_CREATE: &str = "wiki.exports.create";
pub const WIKI_EXPORTS_RETRIEVE: &str = "wiki.exports.retrieve";
pub const WIKI_FILE_ENTRIES_LIST: &str = "wiki.fileEntries.list";
pub const WIKI_LINT_RUNS_CREATE: &str = "wiki.lintRuns.create";
pub const WIKI_EVAL_RUNS_CREATE: &str = "wiki.evalRuns.create";

pub const ALL_OPERATION_IDS: &[&str] = &[
    SOURCES_LIST,
    SOURCES_CREATE,
    DOCUMENTS_LIST,
    DOCUMENTS_CREATE,
    DOCUMENTS_RETRIEVE,
    DOCUMENTS_VERSIONS_CREATE,
    DOCUMENTS_VERSIONS_LIST,
    DRIVE_IMPORTS_CREATE,
    INGESTS_CREATE,
    INGESTS_RETRIEVE,
    SPACES_BROWSER_LIST,
    WIKI_PAGES_LIST,
    WIKI_PAGES_RETRIEVE,
    WIKI_PAGES_REVISIONS_LIST,
    WIKI_PAGES_PUBLISH,
    WIKI_INDEX_RETRIEVE,
    WIKI_INDEX_REBUILD,
    WIKI_LOG_RETRIEVE,
    WIKI_LOG_ENTRIES_CREATE,
    WIKI_SCHEMA_RETRIEVE,
    WIKI_SCHEMA_PROFILES_CREATE,
    WIKI_SCHEMA_PROFILES_UPDATE,
    WIKI_QUERIES_CREATE,
    WIKI_QUERIES_FILE_ANSWER,
    WIKI_CONTEXT_PACKS_CREATE,
    WIKI_COMPILE_JOBS_CREATE,
    WIKI_CANDIDATES_LIST,
    WIKI_CANDIDATES_APPROVE,
    WIKI_CANDIDATES_REJECT,
    WIKI_EXPORTS_CREATE,
    WIKI_EXPORTS_RETRIEVE,
    WIKI_FILE_ENTRIES_LIST,
    WIKI_LINT_RUNS_CREATE,
    WIKI_EVAL_RUNS_CREATE,
];
