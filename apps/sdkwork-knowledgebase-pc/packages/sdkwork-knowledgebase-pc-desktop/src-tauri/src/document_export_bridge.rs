use serde::Deserialize;
use tauri::AppHandle;

use crate::document_export::export_markdown_to_pdf;
use crate::document_export_webview::export_html_to_pdf;
use crate::resource_bridge::{binary_payload_from_bytes, BinaryResourcePayload};

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

#[tauri::command]
pub async fn export_document_pdf(
    app: AppHandle,
    request: ExportDocumentPdfRequest,
) -> Result<BinaryResourcePayload, String> {
    match select_strategy(&request) {
        NativePdfStrategy::MarkdownTypst => {
            let markdown = request
                .markdown
                .as_ref()
                .map(String::as_str)
                .unwrap_or("");
            let bytes = export_markdown_to_pdf(&request.title, markdown)?;
            Ok(binary_payload_from_bytes(bytes, Some("application/pdf".to_string())))
        }
        NativePdfStrategy::HtmlWebView => {
            let bytes = export_html_to_pdf(&app, &request.title, &request.html).await?;
            Ok(binary_payload_from_bytes(bytes, Some("application/pdf".to_string())))
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
