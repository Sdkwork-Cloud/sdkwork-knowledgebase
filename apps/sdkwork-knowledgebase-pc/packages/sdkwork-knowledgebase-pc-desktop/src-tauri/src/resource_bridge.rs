use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::Url;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Component, Path, PathBuf};
use tauri::Manager;

const MAX_REMOTE_RESOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_REMOTE_REDIRECTS: usize = 5;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchBinaryResourceRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadLocalResourceRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenExternalUrlRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBinaryResourceRequest {
    suggested_name: String,
    data_base64: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryResourcePayload {
    data_base64: String,
    mime_type: Option<String>,
    byte_length: usize,
}

fn map_io_error(error: std::io::Error) -> String {
    format!("resource read failed: {error}")
}

fn normalize_local_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("local path is empty".to_string());
    }

    let without_scheme = trimmed
        .strip_prefix("file://")
        .unwrap_or(trimmed)
        .trim();

    let path = PathBuf::from(without_scheme);
    if !path.is_absolute() {
        return Err("only absolute local paths are allowed".to_string());
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err("parent directory traversal is not allowed".to_string());
        }
    }

    Ok(path)
}

fn validate_remote_url(raw: &str) -> Result<Url, String> {
    let parsed = Url::parse(raw.trim()).map_err(|error| format!("invalid URL: {error}"))?;
    match parsed.scheme() {
        "https" | "http" => {}
        _ => return Err("only HTTP(S) resource URLs are allowed".to_string()),
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "resource URL host is required".to_string())?;
    if is_blocked_hostname(host) {
        return Err("resource URL host is not allowed".to_string());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err("resource URL must not target private or loopback addresses".to_string());
        }
    }

    Ok(parsed)
}

fn validate_external_url(raw: &str) -> Result<Url, String> {
    let parsed = Url::parse(raw.trim()).map_err(|error| format!("invalid URL: {error}"))?;
    if parsed.scheme() == "https" || parsed.scheme() == "http" {
        Ok(parsed)
    } else {
        Err("only HTTP(S) URLs can be opened externally".to_string())
    }
}

async fn ensure_public_resolved_target(url: &Url) -> Result<(), String> {
    let host = url
        .host_str()
        .ok_or_else(|| "resource URL host is required".to_string())?;
    let port = url.port_or_known_default().unwrap_or(443);
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(
                "resource URL must not target private or loopback addresses".to_string(),
            );
        }
        return Ok(());
    }

    let authority = format!("{host}:{port}");
    let mut resolved_any = false;
    let addresses: Vec<std::net::SocketAddr> = tokio::net::lookup_host(authority.as_str())
        .await
        .map_err(|error| format!("resource URL DNS lookup failed: {error}"))?
        .collect();
    for address in addresses {
        resolved_any = true;
        if is_blocked_ip(address.ip()) {
            return Err(
                "resource URL resolves to a private or loopback address".to_string(),
            );
        }
    }
    if !resolved_any {
        return Err("resource URL host could not be resolved".to_string());
    }
    Ok(())
}

fn is_blocked_hostname(host: &str) -> bool {
    let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "localhost"
            | "metadata.google.internal"
            | "metadata"
            | "127.0.0.1"
            | "::1"
            | "0.0.0.0"
    ) || normalized.ends_with(".localhost")
        || normalized.ends_with(".local")
        || normalized.ends_with(".internal")
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => is_blocked_ipv4(value),
        IpAddr::V6(value) => is_blocked_ipv6(value),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.octets()[0] == 169 && ip.octets()[1] == 254
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.segments()[0] & 0xfe00 == 0xfc00
}

fn allowed_local_roots(app: &tauri::AppHandle) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    if let Some(home) = app.path().home_dir().ok() {
        roots.push(home);
    }
    if let Ok(app_data) = app.path().app_data_dir() {
        roots.push(app_data);
    }
    if let Ok(app_cache) = app.path().app_cache_dir() {
        roots.push(app_cache);
    }
    if let Ok(downloads) = app.path().download_dir() {
        roots.push(downloads);
    }
    if let Ok(documents) = app.path().document_dir() {
        roots.push(documents);
    }
    if roots.is_empty() {
        return Err("no allowed local resource roots are available".to_string());
    }
    Ok(roots)
}

fn validate_local_read_path(app: &tauri::AppHandle, path: &Path) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| format!("local resource not accessible: {error}"))?;
    let roots = allowed_local_roots(app)?;
    let allowed = roots.iter().any(|root| {
        std::fs::canonicalize(root)
            .ok()
            .is_some_and(|canonical_root| canonical.starts_with(&canonical_root))
    });
    if !allowed {
        return Err("local resource path is outside the desktop sandbox".to_string());
    }
    Ok(canonical)
}

fn payload_from_bytes(bytes: Vec<u8>, mime_type: Option<String>) -> BinaryResourcePayload {
    BinaryResourcePayload {
        byte_length: bytes.len(),
        data_base64: STANDARD.encode(bytes),
        mime_type,
    }
}

pub fn binary_payload_from_bytes(bytes: Vec<u8>, mime_type: Option<String>) -> BinaryResourcePayload {
    payload_from_bytes(bytes, mime_type)
}

#[tauri::command]
pub async fn fetch_binary_resource(
    request: FetchBinaryResourceRequest,
) -> Result<BinaryResourcePayload, String> {
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client init failed: {error}"))?;

    let mut current_url = validate_remote_url(&request.url)?;
    ensure_public_resolved_target(&current_url).await?;

    let mut response = None;
    for redirect_count in 0..=MAX_REMOTE_REDIRECTS {
        let next = client
            .get(current_url.clone())
            .send()
            .await
            .map_err(|error| format!("resource fetch failed: {error}"))?;

        if next.status().is_redirection() {
            if redirect_count == MAX_REMOTE_REDIRECTS {
                return Err("resource fetch exceeded redirect limit".to_string());
            }
            let location = next
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| "redirect response missing Location header".to_string())?;
            current_url = current_url
                .join(location)
                .map_err(|error| format!("invalid redirect location: {error}"))?;
            current_url = validate_remote_url(current_url.as_str())?;
            ensure_public_resolved_target(&current_url).await?;
            continue;
        }

        response = Some(next);
        break;
    }

    let response = response.ok_or_else(|| "resource fetch did not return a response".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "resource fetch failed with status {}",
            response.status()
        ));
    }

    let mime_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("resource body read failed: {error}"))?
        .to_vec();
    if bytes.len() > MAX_REMOTE_RESOURCE_BYTES {
        return Err(format!(
            "resource exceeds maximum allowed size of {MAX_REMOTE_RESOURCE_BYTES} bytes"
        ));
    }

    Ok(payload_from_bytes(bytes, mime_type))
}

#[tauri::command]
pub fn read_local_resource(
    app: tauri::AppHandle,
    request: ReadLocalResourceRequest,
) -> Result<BinaryResourcePayload, String> {
    let path = normalize_local_path(&request.path)?;
    if !path.exists() {
        return Err(format!("local resource not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("local resource is not a file: {}", path.display()));
    }

    let path = validate_local_read_path(&app, &path)?;
    let metadata = std::fs::metadata(&path).map_err(map_io_error)?;
    if metadata.len() as usize > MAX_REMOTE_RESOURCE_BYTES {
        return Err(format!(
            "local resource exceeds maximum allowed size of {MAX_REMOTE_RESOURCE_BYTES} bytes"
        ));
    }

    let bytes = std::fs::read(&path).map_err(map_io_error)?;
    let mime_type = match path.extension().and_then(|value| value.to_str()) {
        Some("pdf") => Some("application/pdf".to_string()),
        Some("png") => Some("image/png".to_string()),
        Some("jpg") | Some("jpeg") => Some("image/jpeg".to_string()),
        _ => None,
    };

    Ok(payload_from_bytes(bytes, mime_type))
}

#[tauri::command]
pub fn open_external_url(request: OpenExternalUrlRequest) -> Result<(), String> {
    let url = validate_external_url(&request.url)?;
    open::that(url.as_str()).map_err(|error| format!("open external URL failed: {error}"))
}

#[tauri::command]
pub fn save_binary_resource(request: SaveBinaryResourceRequest) -> Result<bool, String> {
    use crate::export_save::{save_bytes_to_export_path, ExportSaveMode};

    let bytes = STANDARD
        .decode(request.data_base64.as_bytes())
        .map_err(|error| format!("invalid save payload: {error}"))?;

    let response = save_bytes_to_export_path(bytes, &request.suggested_name, ExportSaveMode::Downloads)?;
    Ok(response.saved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_dir_paths() {
        assert!(normalize_local_path("C:\\Users\\..\\secret.pdf").is_err());
    }

    #[test]
    fn accepts_https_urls() {
        assert!(validate_remote_url("https://example.com/file.pdf").is_ok());
    }

    #[test]
    fn rejects_loopback_urls() {
        assert!(validate_remote_url("http://127.0.0.1/file.pdf").is_err());
    }

    #[test]
    fn rejects_metadata_host() {
        assert!(validate_remote_url("http://metadata.google.internal/file").is_err());
    }
}
