//! Shared ingest payload limits aligned with client upload gates and SECURITY_SPEC.

use sdkwork_utils_rust::is_blank;

pub const MAX_MARKDOWN_PAYLOAD_BYTES: usize = 512 * 1024;
pub const MAX_MARKDOWN_CHUNK_CHARS: usize = 8_192;
pub const MAX_MARKDOWN_CHUNKS: usize = 1_024;
pub const GIT_IMPORT_CONCURRENCY: usize = 4;
pub const OKF_IMPORT_CONCURRENCY: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PayloadLimitError {
    #[error("payload exceeds maximum allowed size of {max_bytes} bytes")]
    PayloadTooLarge { max_bytes: usize },
    #[error("payload must not be empty")]
    PayloadEmpty,
}

pub fn validate_markdown_payload(payload: &str) -> Result<(), PayloadLimitError> {
    if is_blank(Some(payload)) {
        return Err(PayloadLimitError::PayloadEmpty);
    }
    if payload.len() > MAX_MARKDOWN_PAYLOAD_BYTES {
        return Err(PayloadLimitError::PayloadTooLarge {
            max_bytes: MAX_MARKDOWN_PAYLOAD_BYTES,
        });
    }
    Ok(())
}

pub fn split_oversized_paragraph(content: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 || content.is_empty() {
        return Vec::new();
    }
    if content.chars().count() <= max_chars {
        return vec![content.to_string()];
    }

    let mut segments = Vec::new();
    let mut start = 0usize;
    while start < content.len() {
        let remaining = &content[start..];
        let end = remaining
            .char_indices()
            .nth(max_chars)
            .map(|(offset, _)| start + offset)
            .unwrap_or(content.len());
        let mut split_at = end;
        if end < content.len() {
            if let Some(relative) = content[start..end].rfind('\n') {
                split_at = start + relative + 1;
            } else if let Some(relative) = content[start..end].rfind(' ') {
                split_at = start + relative + 1;
            }
        }
        segments.push(content[start..split_at].trim().to_string());
        start = split_at;
    }
    segments.retain(|segment| !is_blank(Some(segment.as_str())));
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_payload() {
        assert_eq!(
            validate_markdown_payload("   "),
            Err(PayloadLimitError::PayloadEmpty)
        );
    }

    #[test]
    fn splits_long_paragraphs() {
        let content = "a".repeat(10_000);
        let segments = split_oversized_paragraph(&content, MAX_MARKDOWN_CHUNK_CHARS);
        assert!(segments.len() > 1);
        assert!(segments
            .iter()
            .all(|segment| segment.chars().count() <= MAX_MARKDOWN_CHUNK_CHARS));
    }

    #[test]
    fn splits_multibyte_paragraphs_only_on_utf8_boundaries() {
        let content = "知识库".repeat(4_000);
        let segments = split_oversized_paragraph(&content, MAX_MARKDOWN_CHUNK_CHARS);

        assert_eq!(segments.concat(), content);
        assert!(segments.len() > 1);
        assert!(segments
            .iter()
            .all(|segment| segment.chars().count() <= MAX_MARKDOWN_CHUNK_CHARS));
    }

    #[test]
    fn zero_character_limit_returns_no_segments() {
        assert!(split_oversized_paragraph("content", 0).is_empty());
    }
}
