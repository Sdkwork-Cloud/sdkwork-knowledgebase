use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::api_key(
        HttpMethod::Post,
        "/knowledge/v3/api/retrievals",
        "knowledge",
        "retrievals.create",
    ),
    HttpRoute::api_key(
        HttpMethod::Get,
        "/knowledge/v3/api/retrievals/{retrievalId}",
        "knowledge",
        "retrievals.retrieve",
    ),
    HttpRoute::api_key(
        HttpMethod::Post,
        "/knowledge/v3/api/context_packs",
        "knowledge",
        "contextPacks.create",
    ),
    HttpRoute::api_key(
        HttpMethod::Post,
        "/knowledge/v3/api/ingests",
        "knowledge",
        "ingests.create",
    ),
    HttpRoute::api_key(
        HttpMethod::Get,
        "/knowledge/v3/api/ingests/{ingestId}",
        "knowledge",
        "ingests.retrieve",
    ),
    HttpRoute::api_key(
        HttpMethod::Get,
        "/knowledge/v3/api/documents",
        "knowledge",
        "documents.list",
    ),
    HttpRoute::api_key(
        HttpMethod::Get,
        "/knowledge/v3/api/documents/{documentId}",
        "knowledge",
        "documents.retrieve",
    ),
    HttpRoute::api_key(
        HttpMethod::Get,
        "/knowledge/v3/api/spaces/{spaceId}/browser",
        "knowledge",
        "spaces.browser.list",
    ),
];

pub fn open_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
