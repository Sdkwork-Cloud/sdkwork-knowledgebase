pub const PREFIX: &str = "/backend/v3/api";
pub const HEALTHZ: &str = "/healthz";
pub const SOURCES: &str = "/backend/v3/api/knowledge/sources";
pub const WIKI_COMPILE_JOBS: &str = "/backend/v3/api/knowledge/wiki_compile_jobs";
pub const WIKI_CANDIDATES: &str = "/backend/v3/api/knowledge/wiki_candidates";
pub const WIKI_CANDIDATE_APPROVE: &str =
    "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/approve";
pub const WIKI_CANDIDATE_REJECT: &str =
    "/backend/v3/api/knowledge/wiki_candidates/:candidate_id/reject";
pub const WIKI_PAGE_PUBLISH: &str = "/backend/v3/api/knowledge/wiki_pages/:page_id/publish";
pub const WIKI_SCHEMA_PROFILES: &str = "/backend/v3/api/knowledge/wiki_schema_profiles";
pub const WIKI_SCHEMA_PROFILE: &str = "/backend/v3/api/knowledge/wiki_schema_profiles/:profile_id";
pub const WIKI_INDEX_REBUILD: &str = "/backend/v3/api/knowledge/wiki_index/rebuild";
pub const WIKI_LOG_ENTRIES: &str = "/backend/v3/api/knowledge/wiki_log_entries";
pub const WIKI_EXPORTS: &str = "/backend/v3/api/knowledge/wiki_exports";
pub const WIKI_EXPORT: &str = "/backend/v3/api/knowledge/wiki_exports/:export_id";
pub const WIKI_FILE_ENTRIES: &str = "/backend/v3/api/knowledge/wiki_file_entries";
pub const WIKI_LINT_RUNS: &str = "/backend/v3/api/knowledge/wiki_lint_runs";
pub const WIKI_EVAL_RUNS: &str = "/backend/v3/api/knowledge/wiki_eval_runs";
pub const INDEXES: &str = "/backend/v3/api/knowledge/indexes";
pub const INDEX: &str = "/backend/v3/api/knowledge/indexes/:index_id";
pub const INDEX_REBUILD: &str = "/backend/v3/api/knowledge/indexes/:index_id/rebuild";
pub const RETRIEVAL_PROFILES: &str = "/backend/v3/api/knowledge/retrieval_profiles";
pub const RETRIEVAL_PROFILE: &str = "/backend/v3/api/knowledge/retrieval_profiles/:profile_id";
pub const RETRIEVAL_TRACES: &str = "/backend/v3/api/knowledge/retrieval_traces";
pub const RETRIEVAL_TRACE: &str = "/backend/v3/api/knowledge/retrieval_traces/:trace_id";
pub const PROVIDER_HEALTH: &str = "/backend/v3/api/knowledge/provider_health";
