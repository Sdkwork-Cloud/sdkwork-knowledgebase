use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest, RateLimitTier};

const fn abuse_sensitive_open_route(
    method: HttpMethod,
    path: &'static str,
    tag: &'static str,
    operation_id: &'static str,
) -> HttpRoute {
    HttpRoute::api_key(method, path, tag, operation_id)
        .with_rate_limit_tier(RateLimitTier::AuthCritical)
}

const HTTP_ROUTES: &[HttpRoute] = &[
    abuse_sensitive_open_route(
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
    abuse_sensitive_open_route(
        HttpMethod::Post,
        "/knowledge/v3/api/context_packs",
        "knowledge",
        "contextPacks.create",
    ),
    abuse_sensitive_open_route(
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
