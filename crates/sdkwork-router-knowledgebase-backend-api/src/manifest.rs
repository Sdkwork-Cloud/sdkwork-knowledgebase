pub const PACKAGE_NAME: &str = "sdkwork-router-knowledgebase-backend-api";
pub const SURFACE: &str = "backend-api";
pub const OWNER: &str = "sdkwork-knowledgebase";
pub const DOMAIN: &str = "intelligence";
pub const CAPABILITY: &str = "knowledgebase";
pub const API_AUTHORITY: &str = "sdkwork-knowledgebase-backend-api";
pub const SDK_FAMILY: &str = "sdkwork-knowledgebase-backend-sdk";
pub const PREFIX: &str = "/backend/v3/api";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteManifestEntry {
    pub method: &'static str,
    pub path: &'static str,
    pub operation_id: &'static str,
}

pub const ROUTES: &[RouteManifestEntry] = &[
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/sources",
        operation_id: "sources.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/sources",
        operation_id: "sources.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_compile_jobs",
        operation_id: "wiki.compileJobs.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/wiki_candidates",
        operation_id: "wiki.candidates.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_candidates/{candidateId}/approve",
        operation_id: "wiki.candidates.approve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_candidates/{candidateId}/reject",
        operation_id: "wiki.candidates.reject",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_pages/{pageId}/publish",
        operation_id: "wiki.pages.publish",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_schema_profiles",
        operation_id: "wiki.schema.profiles.create",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/backend/v3/api/knowledge/wiki_schema_profiles/{profileId}",
        operation_id: "wiki.schema.profiles.update",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_index/rebuild",
        operation_id: "wiki.index.rebuild",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_log_entries",
        operation_id: "wiki.log.entries.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_exports",
        operation_id: "wiki.exports.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/wiki_exports/{exportId}",
        operation_id: "wiki.exports.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/wiki_file_entries",
        operation_id: "wiki.fileEntries.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_lint_runs",
        operation_id: "wiki.lintRuns.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/wiki_eval_runs",
        operation_id: "wiki.evalRuns.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/indexes",
        operation_id: "indexes.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/indexes/{indexId}",
        operation_id: "indexes.retrieve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/indexes/{indexId}/rebuild",
        operation_id: "indexes.rebuild",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/retrieval_profiles",
        operation_id: "retrievalProfiles.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/retrieval_profiles/{profileId}",
        operation_id: "retrievalProfiles.retrieve",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/backend/v3/api/knowledge/retrieval_profiles/{profileId}",
        operation_id: "retrievalProfiles.update",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/retrieval_traces",
        operation_id: "retrievalTraces.list",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/retrieval_traces/{traceId}",
        operation_id: "retrievalTraces.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/provider_health",
        operation_id: "providerHealth.retrieve",
    },
];
