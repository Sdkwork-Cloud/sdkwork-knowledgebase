pub const PACKAGE_NAME: &str = "sdkwork-routes-knowledgebase-open-api";
pub const SURFACE: &str = "open-api";
pub const OWNER: &str = "sdkwork-knowledgebase";
pub const DOMAIN: &str = "intelligence";
pub const CAPABILITY: &str = "knowledgebase";
pub const API_AUTHORITY: &str = "sdkwork-knowledgebase-open-api";
pub const SDK_FAMILY: &str = "sdkwork-knowledgebase-sdk";
pub const PREFIX: &str = "/knowledge/v3/api";
pub const AUTH_MODE: &str = "api-key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteManifestEntry {
    pub method: &'static str,
    pub path: &'static str,
    pub operation_id: &'static str,
    pub auth_mode: &'static str,
}

pub const ROUTES: &[RouteManifestEntry] = &[
    RouteManifestEntry {
        method: "POST",
        path: "/knowledge/v3/api/retrievals",
        operation_id: "retrievals.create",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "GET",
        path: "/knowledge/v3/api/retrievals/{retrievalId}",
        operation_id: "retrievals.retrieve",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "POST",
        path: "/knowledge/v3/api/context_packs",
        operation_id: "contextPacks.create",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "POST",
        path: "/knowledge/v3/api/ingests",
        operation_id: "ingests.create",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "GET",
        path: "/knowledge/v3/api/ingests/{ingestId}",
        operation_id: "ingests.retrieve",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "GET",
        path: "/knowledge/v3/api/documents",
        operation_id: "documents.list",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "GET",
        path: "/knowledge/v3/api/documents/{documentId}",
        operation_id: "documents.retrieve",
        auth_mode: AUTH_MODE,
    },
    RouteManifestEntry {
        method: "GET",
        path: "/knowledge/v3/api/spaces/{spaceId}/browser",
        operation_id: "spaces.browser.list",
        auth_mode: AUTH_MODE,
    },
];
