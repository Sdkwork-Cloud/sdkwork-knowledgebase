//! Shared cursor pagination helpers for Knowledgebase app API list handlers.

use sdkwork_knowledgebase_contract::KnowledgeBrowserNode;
use sdkwork_utils_rust::{
    DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE, PageInfo, PageMode, SdkWorkPageData,
    SdkWorkResultCode,
};

/// Normalize optional `page_size` to the canonical default (20) with max 200.
pub fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_LIST_PAGE_SIZE as u32)
        .clamp(1, MAX_LIST_PAGE_SIZE as u32)
}

/// Parse an opaque numeric id cursor for keyset pagination.
pub fn parse_u64_cursor(cursor: Option<&str>) -> Result<Option<u64>, SdkWorkResultCode> {
    let Some(cursor) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| SdkWorkResultCode::InvalidParameter)
}

/// Build cursor-mode `SdkWorkPageData` from a bounded page window.
pub fn cursor_page_data<T>(
    items: Vec<T>,
    next_cursor: Option<String>,
    has_more: bool,
    page_size: u32,
) -> SdkWorkPageData<T> {
    SdkWorkPageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Cursor,
            page: None,
            page_size: Some(page_size as i32),
            total_items: None,
            total_pages: None,
            next_cursor,
            has_more: Some(has_more),
        },
    }
}

/// Map a browser list window to standard cursor-mode `SdkWorkPageData`.
pub fn browser_list_page_data(
    items: Vec<KnowledgeBrowserNode>,
    next_cursor: Option<String>,
    page_size: u32,
) -> SdkWorkPageData<KnowledgeBrowserNode> {
    cursor_page_data(
        items,
        next_cursor.clone(),
        next_cursor.is_some(),
        page_size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_page_size_defaults_and_clamps() {
        assert_eq!(20, normalize_page_size(None));
        assert_eq!(50, normalize_page_size(Some(50)));
        assert_eq!(200, normalize_page_size(Some(500)));
        assert_eq!(1, normalize_page_size(Some(0)));
    }

    #[test]
    fn parse_u64_cursor_accepts_missing_and_numeric_tokens() {
        assert_eq!(None, parse_u64_cursor(None).expect("missing"));
        assert_eq!(None, parse_u64_cursor(Some("  ")).expect("blank"));
        assert_eq!(Some(42), parse_u64_cursor(Some("42")).expect("numeric"));
    }

    #[test]
    fn parse_u64_cursor_rejects_invalid_tokens() {
        assert_eq!(
            SdkWorkResultCode::InvalidParameter,
            parse_u64_cursor(Some("not-a-number")).expect_err("invalid")
        );
    }

    #[test]
    fn cursor_page_data_uses_cursor_mode_page_info() {
        let page = cursor_page_data(vec!["a".to_string()], Some("99".to_string()), true, 20);
        assert_eq!(PageMode::Cursor, page.page_info.mode);
        assert_eq!(Some(20), page.page_info.page_size);
        assert_eq!(Some("99".to_string()), page.page_info.next_cursor);
        assert_eq!(Some(true), page.page_info.has_more);
    }
}
