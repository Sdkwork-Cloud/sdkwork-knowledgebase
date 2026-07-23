use ammonia::{Builder, UrlRelative};
use pulldown_cmark::{html, CowStr, Event, Options, Parser};
use sdkwork_utils_rust::sha256_hash;
use thiserror::Error;

use crate::ports::knowledge_wiki_persistence::WikiSourceFileKind;

pub const WIKI_RENDERER_POLICY_VERSION: &str = "wiki-safe-renderer-v1";
pub const MAX_WIKI_RENDERED_BYTES: usize = 32 * 1024 * 1024;
pub const WIKI_HTML_MEDIA_TYPE: &str = "text/html; charset=utf-8";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiRenderedRepresentation {
    pub bytes: Vec<u8>,
    pub media_type: &'static str,
    pub content_sha256: String,
}

pub fn canonical_route_for_source(
    source_path: &str,
    file_kind: WikiSourceFileKind,
) -> Result<String, WikiRepresentationError> {
    validate_source_path(source_path)?;
    if file_kind != WikiSourceFileKind::Page {
        return Ok(format!("/{source_path}"));
    }

    let (stem, extension) = source_path.rsplit_once('.').ok_or_else(|| {
        WikiRepresentationError::UnsupportedPageFormat(
            "Wiki page source must have an approved extension".to_string(),
        )
    })?;
    if !is_supported_page_extension(extension) {
        return Err(WikiRepresentationError::UnsupportedPageFormat(format!(
            "Wiki page extension .{} is not supported by the active renderer",
            extension.to_ascii_lowercase()
        )));
    }

    let route = if stem
        .rsplit_once('/')
        .map_or(stem, |(_, file_name)| file_name)
        .eq_ignore_ascii_case("index")
    {
        stem.rsplit_once('/')
            .map(|(directory, _)| format!("/{directory}/"))
            .unwrap_or_else(|| "/".to_string())
    } else {
        format!("/{stem}/")
    };
    validate_canonical_route(&route)?;
    Ok(route)
}

pub fn render_wiki_page(
    source_path: &str,
    file_kind: WikiSourceFileKind,
    source: &[u8],
) -> Result<Option<WikiRenderedRepresentation>, WikiRepresentationError> {
    if file_kind != WikiSourceFileKind::Page {
        return Ok(None);
    }
    validate_source_path(source_path)?;
    let extension = source_path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .ok_or_else(|| {
            WikiRepresentationError::UnsupportedPageFormat(
                "Wiki page source must have an approved extension".to_string(),
            )
        })?;
    let source = std::str::from_utf8(source).map_err(|_| WikiRepresentationError::InvalidUtf8)?;
    let fragment = match extension.as_str() {
        "md" | "markdown" | "mdx" => render_markdown(source),
        "html" | "htm" => sanitize_html(source),
        "txt" | "rst" | "adoc" | "asciidoc" => render_plain_text(source),
        _ => {
            return Err(WikiRepresentationError::UnsupportedPageFormat(format!(
                "Wiki page extension .{extension} is not supported by the active renderer"
            )))
        }
    };
    let document = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"generator\" content=\"SDKWork Wiki\"></head><body><main class=\"sdkwork-wiki-page\">{fragment}</main></body></html>"
    );
    if document.len() > MAX_WIKI_RENDERED_BYTES {
        return Err(WikiRepresentationError::RenderedContentTooLarge);
    }
    let bytes = document.into_bytes();
    Ok(Some(WikiRenderedRepresentation {
        content_sha256: format!("sha256:{}", sha256_hash(&bytes)),
        bytes,
        media_type: WIKI_HTML_MEDIA_TYPE,
    }))
}

pub fn is_rendered_page(source_path: &str, file_kind: WikiSourceFileKind) -> bool {
    file_kind == WikiSourceFileKind::Page
        && source_path
            .rsplit_once('.')
            .is_some_and(|(_, extension)| is_supported_page_extension(extension))
}

fn render_markdown(source: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(source, options);
    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    sanitize_html(&rendered)
}

fn render_plain_text(source: &str) -> String {
    let mut escaped = String::new();
    html::push_html(
        &mut escaped,
        std::iter::once(Event::Text(CowStr::Borrowed(source))),
    );
    format!("<pre>{escaped}</pre>")
}

fn sanitize_html(source: &str) -> String {
    let mut builder = Builder::default();
    builder.url_relative(UrlRelative::PassThrough);
    builder.clean(source).to_string()
}

fn is_supported_page_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "md" | "markdown" | "mdx" | "html" | "htm" | "txt" | "rst" | "adoc" | "asciidoc"
    )
}

fn validate_source_path(source_path: &str) -> Result<(), WikiRepresentationError> {
    if source_path.is_empty()
        || source_path.len() > 4_096
        || source_path.starts_with('/')
        || source_path.contains('\\')
        || source_path.contains('%')
        || source_path.contains('?')
        || source_path.contains('#')
        || source_path.chars().any(char::is_control)
        || source_path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(WikiRepresentationError::InvalidSourcePath);
    }
    Ok(())
}

fn validate_canonical_route(route: &str) -> Result<(), WikiRepresentationError> {
    if route.is_empty()
        || route.len() > 2_048
        || !route.starts_with('/')
        || route.contains('\\')
        || route.contains('%')
        || route.contains('?')
        || route.contains('#')
        || route.contains("//")
        || route.chars().any(char::is_control)
        || route
            .split('/')
            .any(|segment| segment == "." || segment == "..")
    {
        return Err(WikiRepresentationError::InvalidCanonicalRoute);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WikiRepresentationError {
    #[error("Wiki source path is not normalized")]
    InvalidSourcePath,
    #[error("Wiki canonical route is not normalized")]
    InvalidCanonicalRoute,
    #[error("Wiki page source is not valid UTF-8")]
    InvalidUtf8,
    #[error("Wiki page format is unsupported: {0}")]
    UnsupportedPageFormat(String),
    #[error("Wiki rendered representation exceeds its bounded output limit")]
    RenderedContentTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_routes_are_extensionless_and_directory_index_aware() {
        for (path, expected) in [
            ("index.md", "/"),
            ("guide/index.md", "/guide/"),
            ("guide/start.md", "/guide/start/"),
            ("guide/logo.png", "/guide/logo.png"),
        ] {
            let kind = if path.ends_with(".md") {
                WikiSourceFileKind::Page
            } else {
                WikiSourceFileKind::Asset
            };
            assert_eq!(canonical_route_for_source(path, kind).unwrap(), expected);
        }
        assert!(canonical_route_for_source("../private.md", WikiSourceFileKind::Page).is_err());
    }

    #[test]
    fn markdown_and_html_active_content_is_removed() {
        for (path, source) in [
            (
                "guide.md",
                "# Guide\n<script>alert(1)</script>\n[x](javascript:alert(2))",
            ),
            (
                "guide.html",
                "<h1>Guide</h1><img src=x onerror=alert(1)><script>alert(2)</script>",
            ),
        ] {
            let rendered = render_wiki_page(path, WikiSourceFileKind::Page, source.as_bytes())
                .unwrap()
                .unwrap();
            let html = String::from_utf8(rendered.bytes).unwrap();
            assert!(html.contains("Guide"));
            assert!(!html.to_ascii_lowercase().contains("<script"));
            assert!(!html.to_ascii_lowercase().contains("onerror"));
            assert!(!html.to_ascii_lowercase().contains("javascript:"));
            assert_eq!(rendered.media_type, WIKI_HTML_MEDIA_TYPE);
        }
    }

    #[test]
    fn plain_text_is_escaped_inside_a_preformatted_page() {
        let rendered = render_wiki_page(
            "notes.txt",
            WikiSourceFileKind::Page,
            b"<script>alert(1)</script>",
        )
        .unwrap()
        .unwrap();
        let html = String::from_utf8(rendered.bytes).unwrap();
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
