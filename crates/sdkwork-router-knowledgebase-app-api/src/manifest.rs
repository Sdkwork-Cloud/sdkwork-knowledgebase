pub const PACKAGE_NAME: &str = "sdkwork-router-knowledgebase-app-api";
pub const SURFACE: &str = "app-api";
pub const OWNER: &str = "sdkwork-knowledgebase";
pub const DOMAIN: &str = "intelligence";
pub const CAPABILITY: &str = "knowledgebase";
pub const API_AUTHORITY: &str = "sdkwork-knowledgebase-app-api";
pub const SDK_FAMILY: &str = "sdkwork-knowledgebase-app-sdk";
pub const PREFIX: &str = "/app/v3/api";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteManifestEntry {
    pub method: &'static str,
    pub path: &'static str,
    pub operation_id: &'static str,
}

pub const ROUTES: &[RouteManifestEntry] = &[
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/spaces",
        operation_id: "spaces.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/spaces/{spaceId}",
        operation_id: "spaces.retrieve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/drive_imports",
        operation_id: "driveImports.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/ingests",
        operation_id: "ingests.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/ingests/{ingestId}",
        operation_id: "ingests.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/documents",
        operation_id: "documents.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/documents",
        operation_id: "documents.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/documents/{documentId}",
        operation_id: "documents.retrieve",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/app/v3/api/knowledge/documents/{documentId}",
        operation_id: "documents.update",
    },
    RouteManifestEntry {
        method: "DELETE",
        path: "/app/v3/api/knowledge/documents/{documentId}",
        operation_id: "documents.delete",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/documents/{documentId}/versions",
        operation_id: "documents.versions.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/documents/{documentId}/versions",
        operation_id: "documents.versions.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_pages",
        operation_id: "wiki.pages.list",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_pages/{pageId}",
        operation_id: "wiki.pages.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_pages/{pageId}/revisions",
        operation_id: "wiki.pages.revisions.list",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_index",
        operation_id: "wiki.index.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_log",
        operation_id: "wiki.log.retrieve",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/wiki_schema",
        operation_id: "wiki.schema.retrieve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/wiki_queries",
        operation_id: "wiki.queries.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/wiki_queries/{queryId}/file_answer",
        operation_id: "wiki.queries.fileAnswer",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/wiki_context_packs",
        operation_id: "wiki.contextPacks.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/spaces/{spaceId}/browser",
        operation_id: "spaces.browser.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/retrievals",
        operation_id: "retrievals.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/retrievals/{retrievalId}",
        operation_id: "retrievals.retrieve",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/context_packs",
        operation_id: "contextPacks.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/agent_profiles",
        operation_id: "agentProfiles.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}",
        operation_id: "agentProfiles.retrieve",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}",
        operation_id: "agentProfiles.update",
    },
    RouteManifestEntry {
        method: "DELETE",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}",
        operation_id: "agentProfiles.delete",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings",
        operation_id: "agentProfiles.bindings.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings",
        operation_id: "agentProfiles.bindings.create",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}",
        operation_id: "agentProfiles.bindings.update",
    },
    RouteManifestEntry {
        method: "DELETE",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}",
        operation_id: "agentProfiles.bindings.delete",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/retrieval_preview",
        operation_id: "agentProfiles.retrievalPreview.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/agent_profiles/{profileId}/chat",
        operation_id: "agentProfiles.chat.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/spaces/{spaceId}/context_bindings",
        operation_id: "spaces.contextBindings.list",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/spaces/{spaceId}/context_bindings",
        operation_id: "spaces.contextBindings.create",
    },
    RouteManifestEntry {
        method: "GET",
        path: "/app/v3/api/knowledge/context_bindings/{bindingId}",
        operation_id: "contextBindings.retrieve",
    },
    RouteManifestEntry {
        method: "PATCH",
        path: "/app/v3/api/knowledge/context_bindings/{bindingId}",
        operation_id: "contextBindings.update",
    },
    RouteManifestEntry {
        method: "DELETE",
        path: "/app/v3/api/knowledge/context_bindings/{bindingId}",
        operation_id: "contextBindings.delete",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/upload_sessions",
        operation_id: "uploadSessions.create",
    },
    RouteManifestEntry {
        method: "POST",
        path: "/app/v3/api/knowledge/upload_sessions/{sessionId}/complete",
        operation_id: "uploadSessions.complete",
    },
];
