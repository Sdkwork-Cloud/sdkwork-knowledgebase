pub const PREFIX: &str = "/app/v3/api";
pub const HEALTHZ: &str = "/healthz";
pub const SPACES: &str = "/app/v3/api/knowledge/spaces";
pub const SPACE: &str = "/app/v3/api/knowledge/spaces/{space_id}";
pub const DRIVE_IMPORTS: &str = "/app/v3/api/knowledge/drive_imports";
pub const INGESTS: &str = "/app/v3/api/knowledge/ingests";
pub const INGEST: &str = "/app/v3/api/knowledge/ingests/{ingest_id}";
pub const DOCUMENTS: &str = "/app/v3/api/knowledge/documents";
pub const DOCUMENT: &str = "/app/v3/api/knowledge/documents/{document_id}";
pub const DOCUMENT_VERSIONS: &str = "/app/v3/api/knowledge/documents/{document_id}/versions";
pub const WIKI_PAGES: &str = "/app/v3/api/knowledge/wiki_pages";
pub const WIKI_PAGE: &str = "/app/v3/api/knowledge/wiki_pages/{page_id}";
pub const WIKI_PAGE_REVISIONS: &str = "/app/v3/api/knowledge/wiki_pages/{page_id}/revisions";
pub const WIKI_INDEX: &str = "/app/v3/api/knowledge/wiki_index";
pub const WIKI_LOG: &str = "/app/v3/api/knowledge/wiki_log";
pub const WIKI_SCHEMA: &str = "/app/v3/api/knowledge/wiki_schema";
pub const WIKI_QUERIES: &str = "/app/v3/api/knowledge/wiki_queries";
pub const WIKI_QUERY_FILE_ANSWER: &str =
    "/app/v3/api/knowledge/wiki_queries/{query_id}/file_answer";
pub const WIKI_CONTEXT_PACKS: &str = "/app/v3/api/knowledge/wiki_context_packs";
pub const SPACE_BROWSER: &str = "/app/v3/api/knowledge/spaces/{space_id}/browser";
pub const RETRIEVALS: &str = "/app/v3/api/knowledge/retrievals";
pub const RETRIEVAL: &str = "/app/v3/api/knowledge/retrievals/{retrieval_id}";
pub const CONTEXT_PACKS: &str = "/app/v3/api/knowledge/context_packs";
pub const AGENT_PROFILES: &str = "/app/v3/api/knowledge/agent_profiles";
pub const AGENT_PROFILE: &str = "/app/v3/api/knowledge/agent_profiles/{profile_id}";
pub const AGENT_PROFILE_BINDINGS: &str =
    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings";
pub const AGENT_PROFILE_BINDING: &str =
    "/app/v3/api/knowledge/agent_profiles/{profile_id}/bindings/{binding_id}";
pub const AGENT_PROFILE_RETRIEVAL_PREVIEW: &str =
    "/app/v3/api/knowledge/agent_profiles/{profile_id}/retrieval_preview";
pub const AGENT_PROFILE_CHAT: &str = "/app/v3/api/knowledge/agent_profiles/{profile_id}/chat";
pub const SPACE_CONTEXT_BINDINGS: &str = "/app/v3/api/knowledge/spaces/{space_id}/context_bindings";
pub const CONTEXT_BINDING: &str = "/app/v3/api/knowledge/context_bindings/{binding_id}";
