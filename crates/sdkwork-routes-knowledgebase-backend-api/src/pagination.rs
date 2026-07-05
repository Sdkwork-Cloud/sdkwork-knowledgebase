//! Shared cursor pagination helpers for Knowledgebase backend API list handlers.

use sdkwork_utils_rust::{
    DEFAULT_LIST_PAGE_SIZE, MAX_LIST_PAGE_SIZE, PageInfo, PageMode, SdkWorkPageData,
    SdkWorkResultCode,
};

pub fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_LIST_PAGE_SIZE as u32)
        .clamp(1, MAX_LIST_PAGE_SIZE as u32)
}

pub fn parse_u64_cursor(cursor: Option<&str>) -> Result<Option<u64>, SdkWorkResultCode> {
    let Some(cursor) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| SdkWorkResultCode::InvalidParameter)
}

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
