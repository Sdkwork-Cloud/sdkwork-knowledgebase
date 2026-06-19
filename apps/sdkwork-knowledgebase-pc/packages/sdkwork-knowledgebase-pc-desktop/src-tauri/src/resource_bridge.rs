use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Url;
use serde::Deserialize;
use std::path::{Component, PathBuf};

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
        "https" | "http" => Ok(parsed),
        _ => Err("only HTTP(S) resource URLs are allowed".to_string()),
    }
}

fn validate_external_url(raw: &str) -> Result<Url, String> {
    let parsed = Url::parse(raw.trim()).map_err(|error| format!("invalid URL: {error}"))?;
    if parsed.scheme() == "https" || parsed.scheme() == "http" {
        Ok(parsed)
    } else {
        Err("only HTTP(S) URLs can be opened externally".to_string())
    }
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
    let url = validate_remote_url(&request.url)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| format!("HTTP client init failed: {error}"))?;

    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|error| format!("resource fetch failed: {error}"))?;

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

    Ok(payload_from_bytes(bytes, mime_type))
}

#[tauri::command]
pub fn read_local_resource(request: ReadLocalResourceRequest) -> Result<BinaryResourcePayload, String> {
    let path = normalize_local_path(&request.path)?;
    if !path.exists() {
        return Err(format!("local resource not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("local resource is not a file: {}", path.display()));
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
    fn accepts_http_urls() {
        assert!(validate_remote_url("http://example.com/file.pdf").is_ok());
    }
}
