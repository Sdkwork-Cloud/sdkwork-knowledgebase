use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        header::{
            CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, ETAG, HOST, IF_NONE_MATCH,
            LOCATION, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
        },
        HeaderMap, HeaderValue, StatusCode,
    },
    response::Response,
    routing::get,
    Router,
};
use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_site_artifact_store::{
            KnowledgeSiteArtifactStore, ReadKnowledgeSiteArtifactRequest,
        },
        knowledge_site_store::{KnowledgeSiteStore, ResolvedPublicKnowledgeSite},
    },
    site::{KnowledgeSiteReleaseManifest, KnowledgeSiteReleaseManifestEntry},
};

use crate::runtime::KnowledgebaseRuntime;

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PUBLIC_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const CSP: &str = "default-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; img-src 'self'; style-src 'unsafe-inline'; font-src 'self'; connect-src 'self'";

#[derive(Clone)]
struct PublicSiteState {
    runtime: KnowledgebaseRuntime,
}

pub(crate) fn build_public_site_router(runtime: KnowledgebaseRuntime) -> Router {
    Router::new()
        .route("/wiki/{space_id}", get(standalone_root_redirect))
        .route("/wiki/{space_id}/", get(standalone_root))
        .route("/wiki/{space_id}/{*path}", get(standalone_path))
        .route("/", get(cloud_root))
        .route("/{*path}", get(cloud_path))
        .with_state(PublicSiteState { runtime })
}

async fn standalone_root_redirect(Path(space_id): Path<u64>) -> Response {
    redirect_response(&format!("/wiki/{space_id}/"), false)
}

async fn standalone_root(
    State(state): State<PublicSiteState>,
    Path(space_id): Path<u64>,
    headers: HeaderMap,
) -> Response {
    serve_standalone(&state.runtime, space_id, "", &headers).await
}

async fn standalone_path(
    State(state): State<PublicSiteState>,
    Path((space_id, path)): Path<(u64, String)>,
    headers: HeaderMap,
) -> Response {
    serve_standalone(&state.runtime, space_id, &path, &headers).await
}

async fn cloud_root(State(state): State<PublicSiteState>, headers: HeaderMap) -> Response {
    serve_cloud(&state.runtime, "", &headers).await
}

async fn cloud_path(
    State(state): State<PublicSiteState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Response {
    serve_cloud(&state.runtime, &path, &headers).await
}

async fn serve_standalone(
    runtime: &KnowledgebaseRuntime,
    space_id: u64,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let Ok(path) = normalize_public_path(path) else {
        return not_found();
    };
    let Ok(resolved) = runtime
        .site_store()
        .resolve_public_site_by_space(space_id)
        .await
    else {
        return not_found();
    };
    serve_resolved(runtime, resolved, path, headers, false).await
}

async fn serve_cloud(
    runtime: &KnowledgebaseRuntime,
    path: &str,
    headers: &HeaderMap,
) -> Response {
    let Some(host) = normalized_request_host(headers) else {
        return not_found();
    };
    let Ok(path) = normalize_public_path(path) else {
        return not_found();
    };
    let Ok(resolved) = runtime
        .site_store()
        .resolve_public_site_by_host(&host)
        .await
    else {
        return not_found();
    };
    if let Some(canonical_host) = resolved.canonical_host.as_deref() {
        if canonical_host != host {
            let location = if path.is_empty() {
                format!("https://{canonical_host}/")
            } else {
                format!("https://{canonical_host}/{path}")
            };
            return redirect_response(&location, true);
        }
    }
    serve_resolved(runtime, resolved, path, headers, true).await
}

async fn serve_resolved(
    runtime: &KnowledgebaseRuntime,
    resolved: ResolvedPublicKnowledgeSite,
    requested_path: String,
    headers: &HeaderMap,
    cloud_mode: bool,
) -> Response {
    let Some(manifest_space_id) = resolved.release.manifest_drive_space_id.as_deref() else {
        return not_found();
    };
    let Some(manifest_node_id) = resolved.release.manifest_drive_node_id.as_deref() else {
        return not_found();
    };
    let Ok(manifest_artifact) = runtime
        .site_artifact_store()
        .read_artifact(ReadKnowledgeSiteArtifactRequest {
            tenant_id: runtime.tenant_id(),
            drive_space_id: manifest_space_id.to_string(),
            drive_node_id: manifest_node_id.to_string(),
            max_bytes: MAX_MANIFEST_BYTES,
        })
        .await
    else {
        return not_found();
    };
    if resolved
        .release
        .manifest_checksum_sha256_hex
        .as_deref()
        != Some(manifest_artifact.checksum_sha256_hex.as_str())
    {
        return not_found();
    }
    let Ok(manifest) = serde_json::from_slice::<KnowledgeSiteReleaseManifest>(
        &manifest_artifact.body,
    ) else {
        return not_found();
    };
    if manifest.schema_version != 1
        || manifest.site_id != resolved.site.id.to_string()
        || manifest.release_id != resolved.release.id.to_string()
        || manifest.source_content_hash != resolved.release.source_content_hash
    {
        return not_found();
    }

    let (entry, immutable) = match manifest_entry_for_path(
        &manifest,
        &requested_path,
        resolved.release.id,
    ) {
        Some(value) => value,
        None => {
            if !requested_path.is_empty()
                && !requested_path.ends_with('/')
                && manifest
                    .pages
                    .iter()
                    .any(|entry| entry.public_path == format!("{requested_path}/"))
            {
                let location = if cloud_mode {
                    format!("/{requested_path}/")
                } else {
                    format!("/wiki/{}/{requested_path}/", resolved.site.space_id)
                };
                return redirect_response(&location, false);
            }
            return not_found();
        }
    };
    if !allowed_content_type(&entry.content_type) || entry.content_length > MAX_PUBLIC_ARTIFACT_BYTES
    {
        return not_found();
    }
    let Ok(artifact) = runtime
        .site_artifact_store()
        .read_artifact(ReadKnowledgeSiteArtifactRequest {
            tenant_id: runtime.tenant_id(),
            drive_space_id: entry.drive_space_id.clone(),
            drive_node_id: entry.drive_node_id.clone(),
            max_bytes: entry.content_length.min(MAX_PUBLIC_ARTIFACT_BYTES),
        })
        .await
    else {
        return not_found();
    };
    if artifact.content_type != entry.content_type
        || artifact.checksum_sha256_hex != entry.checksum_sha256_hex
        || artifact.body.len() as u64 != entry.content_length
    {
        return not_found();
    }
    let etag = format!("\"sha256:{}\"", artifact.checksum_sha256_hex);
    if headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|candidate| candidate.trim() == etag))
    {
        return response_with_headers(StatusCode::NOT_MODIFIED, Body::empty(), None, &etag, immutable);
    }
    response_with_headers(
        StatusCode::OK,
        Body::from(artifact.body),
        Some(&artifact.content_type),
        &etag,
        immutable,
    )
}

fn manifest_entry_for_path<'a>(
    manifest: &'a KnowledgeSiteReleaseManifest,
    requested_path: &str,
    release_id: u64,
) -> Option<(&'a KnowledgeSiteReleaseManifestEntry, bool)> {
    if requested_path.is_empty() {
        return manifest
            .pages
            .iter()
            .find(|entry| entry.public_path == manifest.homepage_path)
            .map(|entry| (entry, false));
    }
    if requested_path == "assets/search-index.json" {
        return Some((&manifest.search_index, false));
    }
    if requested_path == "sitemap.xml" {
        return Some((&manifest.sitemap, false));
    }
    let release_prefix = format!("_releases/{release_id}/");
    if let Some(release_path) = requested_path.strip_prefix(&release_prefix) {
        return all_manifest_entries(manifest)
            .find(|entry| entry.public_path == release_path)
            .map(|entry| (entry, true));
    }
    manifest
        .pages
        .iter()
        .find(|entry| entry.public_path == requested_path)
        .map(|entry| (entry, false))
}

fn all_manifest_entries(
    manifest: &KnowledgeSiteReleaseManifest,
) -> impl Iterator<Item = &KnowledgeSiteReleaseManifestEntry> {
    manifest
        .pages
        .iter()
        .chain(std::iter::once(&manifest.search_index))
        .chain(std::iter::once(&manifest.sitemap))
}

fn normalize_public_path(path: &str) -> Result<String, ()> {
    if path.len() > 2_048
        || path.contains('\\')
        || path.chars().any(|character| character.is_control())
    {
        return Err(());
    }
    let path = path.trim_start_matches('/');
    if path.split('/').any(|segment| segment == "." || segment == "..") {
        return Err(());
    }
    Ok(path.to_string())
}

fn normalized_request_host(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(HOST)?.to_str().ok()?.trim().to_ascii_lowercase();
    if value.is_empty() || value.len() > 259 || value.contains(['/', '\\', '@', ' ', '\t']) {
        return None;
    }
    let host = if value.starts_with('[') {
        return None;
    } else {
        value.split_once(':').map(|(host, _)| host).unwrap_or(&value)
    };
    if host.is_empty()
        || host.starts_with('.')
        || host.ends_with('.')
        || host.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
    {
        return None;
    }
    Some(host.to_string())
}

fn allowed_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "text/html; charset=utf-8"
            | "application/json; charset=utf-8"
            | "application/xml; charset=utf-8"
            | "text/css; charset=utf-8"
            | "image/png"
            | "image/jpeg"
            | "image/webp"
            | "image/gif"
    )
}

fn response_with_headers(
    status: StatusCode,
    body: Body,
    content_type: Option<&str>,
    etag: &str,
    immutable: bool,
) -> Response {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    let headers = response.headers_mut();
    if let Some(content_type) = content_type.and_then(header_value) {
        headers.insert(CONTENT_TYPE, content_type);
    }
    if let Some(etag) = header_value(etag) {
        headers.insert(ETAG, etag);
    }
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static(if immutable {
            "public, max-age=31536000, immutable"
        } else {
            "public, max-age=60, stale-while-revalidate=300"
        }),
    );
    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP));
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    response
}

fn redirect_response(location: &str, permanent: bool) -> Response {
    let Some(location) = header_value(location) else {
        return not_found();
    };
    let mut response = Response::new(Body::empty());
    *response.status_mut() = if permanent {
        StatusCode::PERMANENT_REDIRECT
    } else {
        StatusCode::TEMPORARY_REDIRECT
    };
    response.headers_mut().insert(LOCATION, location);
    response
}

fn not_found() -> Response {
    let mut response = Response::new(Body::from("Not Found"));
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn header_value(value: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(value).ok()
}

#[cfg(test)]
mod tests {
    use super::{allowed_content_type, normalize_public_path, normalized_request_host};
    use axum::http::{header::HOST, HeaderMap, HeaderValue};

    #[test]
    fn public_path_rejects_traversal_and_backslashes() {
        assert_eq!(normalize_public_path("guides/start/").unwrap(), "guides/start/");
        assert!(normalize_public_path("../private").is_err());
        assert!(normalize_public_path("guides\\private").is_err());
    }

    #[test]
    fn host_normalization_rejects_credentials_and_invalid_labels() {
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static("docs.kb.sdkwork.com:443"));
        assert_eq!(
            normalized_request_host(&headers).as_deref(),
            Some("docs.kb.sdkwork.com")
        );
        headers.insert(HOST, HeaderValue::from_static("user@docs.kb.sdkwork.com"));
        assert!(normalized_request_host(&headers).is_none());
    }

    #[test]
    fn public_mime_policy_rejects_svg_and_javascript() {
        assert!(allowed_content_type("text/html; charset=utf-8"));
        assert!(!allowed_content_type("image/svg+xml"));
        assert!(!allowed_content_type("application/javascript"));
    }
}

