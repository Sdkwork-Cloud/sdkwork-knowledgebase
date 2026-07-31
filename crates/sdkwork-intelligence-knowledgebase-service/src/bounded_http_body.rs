use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum BoundedHttpBodyError {
    #[error("response body exceeds the {max_bytes} byte limit")]
    TooLarge { max_bytes: usize },
    #[error("response body read failed: {detail}")]
    Read { detail: String },
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

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| BoundedHttpBodyError::Read {
            detail: redacted_reqwest_error_detail(&error),
        })?
    {
        if chunk.len() > max_bytes.saturating_sub(body.len()) {
            return Err(BoundedHttpBodyError::TooLarge { max_bytes });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

pub(crate) fn redacted_reqwest_error_detail(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        return "request timed out".to_string();
    }
    if error.is_connect() {
        return "connection failed".to_string();
    }
    if let Some(status) = error.status() {
        return format!("upstream returned HTTP {}", status.as_u16());
    }
    if error.is_decode() {
        return "upstream response decoding failed".to_string();
    }
    if error.is_request() {
        return "request construction failed".to_string();
    }
    "transport failed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_error_detail_does_not_render_url_or_credentials() {
        let error = reqwest::Client::new()
            .get("http://[::1?access_token=super-secret")
            .build()
            .expect_err("invalid URL must fail request construction");

        let detail = redacted_reqwest_error_detail(&error);

        assert!(!detail.contains("super-secret"));
        assert!(!detail.contains("access_token"));
        assert!(!detail.contains("http://"));
    }
}
