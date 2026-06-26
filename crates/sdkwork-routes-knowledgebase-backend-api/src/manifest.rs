pub const PACKAGE_NAME: &str = "sdkwork-routes-knowledgebase-backend-api";
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
        path: "/backend/v3/api/knowledge/okf/compile_jobs",
        operation_id: "okf.compileJobs.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/okf/candidates",
        operation_id: "okf.candidates.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/candidates/{candidateId}/approve",
        operation_id: "okf.candidates.approve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/candidates/{candidateId}/reject",
        operation_id: "okf.candidates.reject",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/concepts/{conceptId}/publish",
        operation_id: "okf.concepts.publish",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/profile",
        operation_id: "okf.profile.create",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/backend/v3/api/knowledge/okf/profile/{profileId}",
        operation_id: "okf.profile.update",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/index/rebuild",
        operation_id: "okf.bundle.index.rebuild",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/log_entries",
        operation_id: "okf.log.entries.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/exports",
        operation_id: "okf.bundle.export.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/okf/exports/{exportId}",
        operation_id: "okf.bundle.export.retrieve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/imports",
        operation_id: "okf.bundle.import.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/backend/v3/api/knowledge/okf/bundle/files",
        operation_id: "okf.bundle.files.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/lint_runs",
        operation_id: "okf.lintRuns.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/backend/v3/api/knowledge/okf/eval_runs",
        operation_id: "okf.evalRuns.create",
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
