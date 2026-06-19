use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportSaveMode {
    Downloads,
    SaveAs,
}

impl ExportSaveMode {
    fn from_str(raw: &str) -> Result<Self, String> {
        match raw {
            "downloads" => Ok(Self::Downloads),
            "saveAs" => Ok(Self::SaveAs),
            _ => Err(format!("unsupported export save mode: {raw}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Downloads => "downloads",
            Self::SaveAs => "saveAs",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveExportFileRequest {
    suggested_name: String,
    data_base64: String,
    mode: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveExportFileResponse {
    pub saved: bool,
    pub cancelled: bool,
    pub path: Option<String>,
    pub mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealExportFileRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocateExportFileRequest {
    file_name: String,
}

pub fn sanitize_file_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "document.bin".to_string();
    }

    let mut sanitized = trimmed
        .chars()
        .map(|ch| {
            if matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        sanitized = "document.bin".to_string();
    }

    sanitized
}

pub fn resolve_downloads_dir() -> Result<PathBuf, String> {
    if let Some(dir) = dirs::download_dir() {
        return Ok(dir);
    }

    if let Ok(xdg_downloads) = std::env::var("XDG_DOWNLOAD_DIR") {
        let trimmed = xdg_downloads.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    if let Some(home) = dirs::home_dir() {
        let downloads = home.join("Downloads");
        return Ok(downloads);
    }

    if let Some(documents) = dirs::document_dir() {
        return Ok(documents);
    }

    Err("downloads directory is unavailable on this system".to_string())
}

pub fn unique_file_path(directory: &Path, file_name: &str) -> PathBuf {
    let safe_name = sanitize_file_name(file_name);
    let mut candidate = directory.join(&safe_name);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(&safe_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("document");
    let extension = path.extension().and_then(|value| value.to_str());

    for index in 1..=999 {
        let next_name = match extension {
            Some(ext) if !ext.is_empty() => format!("{stem} ({index}).{ext}"),
            _ => format!("{stem} ({index})"),
        };
        candidate = directory.join(next_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    let fallback = match extension {
        Some(ext) if !ext.is_empty() => format!("{stem}-{}.{}", chrono_like_suffix(), ext),
        _ => format!("{stem}-{}", chrono_like_suffix()),
    };
    directory.join(fallback)
}

fn chrono_like_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub fn save_bytes_to_export_path(
    bytes: Vec<u8>,
    suggested_name: &str,
    mode: ExportSaveMode,
) -> Result<SaveExportFileResponse, String> {
    let safe_name = sanitize_file_name(suggested_name);

    match mode {
        ExportSaveMode::Downloads => {
            let downloads_dir = resolve_downloads_dir()?;
            std::fs::create_dir_all(&downloads_dir).map_err(map_io_error)?;
            let target_path = unique_file_path(&downloads_dir, &safe_name);
            std::fs::write(&target_path, bytes).map_err(map_io_error)?;
            Ok(SaveExportFileResponse {
                saved: true,
                cancelled: false,
                path: Some(target_path.to_string_lossy().into_owned()),
                mode: mode.as_str().to_string(),
            })
        }
        ExportSaveMode::SaveAs => {
            let mut dialog = rfd::FileDialog::new().set_file_name(&safe_name);
            if let Ok(downloads_dir) = resolve_downloads_dir() {
                dialog = dialog.set_directory(downloads_dir);
            }

            let Some(path) = dialog.save_file() else {
                return Ok(SaveExportFileResponse {
                    saved: false,
                    cancelled: true,
                    path: None,
                    mode: mode.as_str().to_string(),
                });
            };

            std::fs::write(&path, bytes).map_err(map_io_error)?;
            Ok(SaveExportFileResponse {
                saved: true,
                cancelled: false,
                path: Some(path.to_string_lossy().into_owned()),
                mode: mode.as_str().to_string(),
            })
        }
    }
}

pub fn reveal_export_in_folder(raw_path: &str) -> Result<(), String> {
    let path = validate_export_file_path(raw_path)?;

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|error| format!("failed to reveal export file: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|error| format!("failed to reveal export file: {error}"))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let folder = path
            .parent()
            .ok_or_else(|| "export file has no parent directory".to_string())?;
        open::that(folder).map_err(|error| format!("failed to open export folder: {error}"))?;
        return Ok(());
    }

    #[cfg(not(any(windows, target_os = "macos", unix)))]
    {
        Err("reveal export file is not supported on this platform".to_string())
    }
}

pub fn launch_export_file(raw_path: &str) -> Result<(), String> {
    let path = validate_export_file_path(raw_path)?;
    open::that(&path).map_err(|error| format!("failed to open export file: {error}"))
}

pub fn locate_export_in_downloads(file_name: &str) -> Result<PathBuf, String> {
    let dir = resolve_downloads_dir()?;
    let safe = sanitize_file_name(file_name);
    let direct = dir.join(&safe);
    if direct.is_file() {
        return Ok(direct);
    }

    let path = Path::new(&safe);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("document");
    let extension = path.extension().and_then(|value| value.to_str());

    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in std::fs::read_dir(&dir).map_err(map_io_error)? {
        let entry = entry.map_err(map_io_error)?;
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let name = file_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let entry_path = Path::new(name);
        let file_stem = entry_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let file_ext = entry_path.extension().and_then(|value| value.to_str());

        let stem_matches = file_stem == stem || file_stem.starts_with(&format!("{stem} ("));
        let ext_matches = extension.is_none() || extension == file_ext;
        if !stem_matches || !ext_matches {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        if best.as_ref().is_none_or(|(_, current)| modified > *current) {
            best = Some((file_path, modified));
        }
    }

    best.map(|(path, _)| path).ok_or_else(|| {
        format!("export file not found in downloads folder: {safe}")
    })
}

fn validate_export_file_path(raw_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw_path.trim());
    if !path.is_absolute() {
        return Err("only absolute export paths can be opened".to_string());
    }
    if !path.exists() {
        return Err(format!("export file not found: {}", path.display()));
    }
    Ok(path)
}

fn map_io_error(error: std::io::Error) -> String {
    format!("export save failed: {error}")
}

#[tauri::command]
pub fn save_export_file(request: SaveExportFileRequest) -> Result<SaveExportFileResponse, String> {
    let mode = ExportSaveMode::from_str(&request.mode)?;
    let bytes = STANDARD
        .decode(request.data_base64.as_bytes())
        .map_err(|error| format!("invalid export payload: {error}"))?;

    save_bytes_to_export_path(bytes, &request.suggested_name, mode)
}

#[tauri::command]
pub fn reveal_export_file(request: RevealExportFileRequest) -> Result<(), String> {
    reveal_export_in_folder(&request.path)
}

#[tauri::command]
pub fn open_export_file(request: RevealExportFileRequest) -> Result<(), String> {
    launch_export_file(&request.path)
}

#[tauri::command]
pub fn locate_export_file(request: LocateExportFileRequest) -> Result<String, String> {
    let path = locate_export_in_downloads(&request.file_name)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn unique_file_path_appends_suffix_for_duplicates() {
        let temp_dir = std::env::temp_dir().join(format!(
            "kb-export-test-{}",
            chrono_like_suffix()
        ));
        fs::create_dir_all(&temp_dir).expect("temp dir should exist");
        let first = unique_file_path(&temp_dir, "note.pdf");
        fs::write(&first, b"first").expect("write first");
        let second = unique_file_path(&temp_dir, "note.pdf");
        assert_ne!(first, second);
        assert!(second.to_string_lossy().contains("(1)"));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn sanitize_file_name_replaces_invalid_chars() {
        assert_eq!(sanitize_file_name("bad:name?.pdf"), "bad_name_.pdf");
    }
}
