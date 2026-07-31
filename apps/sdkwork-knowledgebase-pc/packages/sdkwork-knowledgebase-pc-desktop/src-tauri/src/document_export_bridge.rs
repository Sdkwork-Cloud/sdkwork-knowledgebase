use serde::Deserialize;
use std::sync::LazyLock;
use tauri::AppHandle;
use tokio::sync::Semaphore;

use crate::document_export::export_markdown_to_pdf;
use crate::document_export_webview::export_html_to_pdf;
use crate::export_save::MAX_EXPORT_FILE_BYTES;
use crate::resource_bridge::{binary_payload_from_bytes, BinaryResourcePayload};

const MAX_DOCUMENT_TITLE_BYTES: usize = 1024;
const MAX_DOCUMENT_SOURCE_BYTES: usize = 16 * 1024 * 1024;
static PDF_EXPORT_LIMIT: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDocumentPdfRequest {
    title: String,
    html: String,
    markdown: Option<String>,
    source_kind: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePdfStrategy {
    MarkdownTypst,
    HtmlWebView,
    Unavailable,
}

fn has_markdown(request: &ExportDocumentPdfRequest) -> bool {
    request
        .markdown
        .as_ref()
        .is_some_and(|markdown| !markdown.trim().is_empty())
}

fn select_strategy(request: &ExportDocumentPdfRequest) -> NativePdfStrategy {
    let markdown_source = request.source_kind.as_deref() == Some("markdown")
        || (request.source_kind.is_none() && has_markdown(request));

    if markdown_source && has_markdown(request) {
        return NativePdfStrategy::MarkdownTypst;
    }

    if request.source_kind.as_deref() == Some("richtext") && !request.html.trim().is_empty() {
        #[cfg(windows)]
        {
            return NativePdfStrategy::HtmlWebView;
        }
        #[cfg(not(windows))]
        {
            return NativePdfStrategy::Unavailable;
        }
    }

    NativePdfStrategy::Unavailable
}

fn validate_export_request(request: &ExportDocumentPdfRequest) -> Result<(), String> {
    if request.title.len() > MAX_DOCUMENT_TITLE_BYTES {
        return Err("document title exceeds the maximum allowed size".to_string());
    }
    let markdown_bytes = request.markdown.as_ref().map_or(0, String::len);
    if request.html.len() > MAX_DOCUMENT_SOURCE_BYTES || markdown_bytes > MAX_DOCUMENT_SOURCE_BYTES
    {
        return Err("document source exceeds the maximum allowed size".to_string());
    }
    Ok(())
}

fn validate_pdf_output(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_EXPORT_FILE_BYTES {
        return Err("generated PDF exceeds the maximum allowed size".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn export_document_pdf(
    app: AppHandle,
    request: ExportDocumentPdfRequest,
) -> Result<BinaryResourcePayload, String> {
    validate_export_request(&request)?;
    let _permit = PDF_EXPORT_LIMIT
        .try_acquire()
        .map_err(|_| "another native PDF export is already running".to_string())?;
    match select_strategy(&request) {
        NativePdfStrategy::MarkdownTypst => {
            let markdown = request.markdown.as_deref().unwrap_or("");
            let bytes = export_markdown_to_pdf(&request.title, markdown)?;
            validate_pdf_output(&bytes)?;
            Ok(binary_payload_from_bytes(
                bytes,
                Some("application/pdf".to_string()),
            ))
        }
        NativePdfStrategy::HtmlWebView => {
            let bytes = export_html_to_pdf(&app, &request.title, &request.html).await?;
            validate_pdf_output(&bytes)?;
            Ok(binary_payload_from_bytes(
                bytes,
                Some("application/pdf".to_string()),
            ))
        }
        NativePdfStrategy::Unavailable => Err(
            "native PDF export is unavailable for this content; use canvas fallback".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_markdown_strategy_for_markdown_source_kind() {
        let request = ExportDocumentPdfRequest {
            title: "Note".to_string(),
            html: "<p>html</p>".to_string(),
            markdown: Some("# md".to_string()),
            source_kind: Some("markdown".to_string()),
        };
        assert_eq!(select_strategy(&request), NativePdfStrategy::MarkdownTypst);
    }

    #[test]
    fn prefers_markdown_strategy_when_source_kind_missing_but_markdown_present() {
        let request = ExportDocumentPdfRequest {
            title: "Note".to_string(),
            html: "<p>html</p>".to_string(),
            markdown: Some("# md".to_string()),
            source_kind: None,
        };
        assert_eq!(select_strategy(&request), NativePdfStrategy::MarkdownTypst);
    }

    #[test]
    fn prefers_html_webview_for_richtext_on_windows() {
        let request = ExportDocumentPdfRequest {
            title: "Note".to_string(),
            html: "<p><strong>rich</strong></p>".to_string(),
            markdown: Some("plain fallback".to_string()),
            source_kind: Some("richtext".to_string()),
        };
        #[cfg(windows)]
        assert_eq!(select_strategy(&request), NativePdfStrategy::HtmlWebView);
        #[cfg(not(windows))]
        assert_eq!(select_strategy(&request), NativePdfStrategy::Unavailable);
    }
}
