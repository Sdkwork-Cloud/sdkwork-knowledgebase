use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum BoundedHttpBodyError {
    #[error("response body exceeds the {max_bytes} byte limit")]
    TooLarge { max_bytes: usize },
    #[error("response body read failed: {0}")]
    Read(#[source] reqwest::Error),
}

pub(crate) async fn read_bounded_http_body(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, BoundedHttpBodyError> {
    let declared_length = response.content_length();
    if declared_length.is_some_and(|length| length > u64::try_from(max_bytes).unwrap_or(u64::MAX)) {
        return Err(BoundedHttpBodyError::TooLarge { max_bytes });
    }

    let initial_capacity = declared_length
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or_default()
        .min(max_bytes);
    let mut body = Vec::with_capacity(initial_capacity);

    while let Some(chunk) = response.chunk().await.map_err(BoundedHttpBodyError::Read)? {
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(BoundedHttpBodyError::TooLarge { max_bytes });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}
