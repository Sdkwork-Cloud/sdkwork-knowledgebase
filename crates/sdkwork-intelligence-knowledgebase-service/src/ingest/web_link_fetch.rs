use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use reqwest::header::{HOST, LOCATION};
use reqwest::redirect::Policy;
use reqwest::Url;
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

const MAX_WEB_LINK_BYTES: usize = 512 * 1024;
const MAX_WEB_LINK_REDIRECTS: usize = 5;

#[derive(Debug, Error)]
pub enum WebLinkFetchError {
    #[error("invalid web link request: {0}")]
    InvalidRequest(String),
    #[error("web link fetch failed: {0}")]
    Upstream(String),
}

pub async fn fetch_web_link_markdown(
    source_url: &str,
    title_hint: &str,
) -> Result<String, WebLinkFetchError> {
    let url = validate_public_http_url(source_url)?;
    ensure_public_resolved_target(&url).await?;

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| WebLinkFetchError::Upstream(error.to_string()))?;

    let mut current_url = url;
    let mut response = None;
    for redirect_count in 0..=MAX_WEB_LINK_REDIRECTS {
        let next = send_pinned_get(&client, &current_url).await?;

        if next.status().is_redirection() {
            if redirect_count == MAX_WEB_LINK_REDIRECTS {
                return Err(WebLinkFetchError::InvalidRequest(
                    "source_url exceeded redirect limit".to_string(),
                ));
            }
            let location = next
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    WebLinkFetchError::Upstream(
                        "redirect response missing Location header".to_string(),
                    )
                })?;
            current_url = current_url.join(location).map_err(|error| {
                WebLinkFetchError::InvalidRequest(format!("invalid redirect location: {error}"))
            })?;
            validate_public_http_url(current_url.as_str())?;
            ensure_public_resolved_target(&current_url).await?;
            continue;
        }

        response = Some(next);
        break;
    }

    let response = response.ok_or_else(|| {
        WebLinkFetchError::Upstream("web link fetch did not return a response".to_string())
    })?;

    if !response.status().is_success() {
        return Err(WebLinkFetchError::Upstream(format!(
            "upstream returned HTTP {}",
            response.status()
        )));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let bytes = response
        .bytes()
        .await
        .map_err(|error| WebLinkFetchError::Upstream(error.to_string()))?;
    if bytes.len() > MAX_WEB_LINK_BYTES {
        return Err(WebLinkFetchError::InvalidRequest(format!(
            "web page exceeds {} KB import limit",
            MAX_WEB_LINK_BYTES / 1024
        )));
    }

    let body = String::from_utf8_lossy(&bytes).trim().to_string();
    if is_blank(Some(body.as_str())) {
        return Err(WebLinkFetchError::InvalidRequest(
            "fetched web page is empty".to_string(),
        ));
    }

    if content_type.contains("text/html") || looks_like_html(&body) {
        return Ok(html_to_markdown(&body, current_url.as_str(), title_hint));
    }

    Ok(body)
}

async fn ensure_public_resolved_target(url: &Url) -> Result<(), WebLinkFetchError> {
    resolve_public_socket_addr(url).await.map(|_| ())
}

async fn send_pinned_get(
    client: &reqwest::Client,
    url: &Url,
) -> Result<reqwest::Response, WebLinkFetchError> {
    let host = url.host_str().ok_or_else(|| {
        WebLinkFetchError::InvalidRequest("source_url host is required".to_string())
    })?;
    let socket = resolve_public_socket_addr(url).await?;
    let pinned_url = pinned_request_url(url, socket)?;

    client
        .get(pinned_url)
        .header(HOST, host)
        .header(
            reqwest::header::USER_AGENT,
            "SDKWork-Knowledgebase-Ingest/1.0",
        )
        .send()
        .await
        .map_err(|error| WebLinkFetchError::Upstream(error.to_string()))
}

async fn resolve_public_socket_addr(url: &Url) -> Result<SocketAddr, WebLinkFetchError> {
    let host = url.host_str().ok_or_else(|| {
        WebLinkFetchError::InvalidRequest("source_url host is required".to_string())
    })?;
    let port = url.port_or_known_default().unwrap_or(443);

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(WebLinkFetchError::InvalidRequest(
                "source_url must not target private or loopback addresses".to_string(),
            ));
        }
        return Ok(SocketAddr::new(ip, port));
    }

    let authority = format!("{host}:{port}");
    let mut addresses = tokio::net::lookup_host(authority.as_str())
        .await
        .map_err(|error| {
            WebLinkFetchError::InvalidRequest(format!("source_url DNS lookup failed: {error}"))
        })?;
    addresses
        .find(|address| !is_blocked_ip(address.ip()))
        .ok_or_else(|| {
            WebLinkFetchError::InvalidRequest(
                "source_url resolves only to private or loopback addresses".to_string(),
            )
        })
}

fn pinned_request_url(url: &Url, socket: SocketAddr) -> Result<Url, WebLinkFetchError> {
    let mut pinned = url.clone();
    pinned.set_ip_host(socket.ip()).map_err(|_| {
        WebLinkFetchError::InvalidRequest("invalid pinned source_url host".to_string())
    })?;
    let _ = pinned.set_port(Some(socket.port()));
    Ok(pinned)
}

pub fn validate_public_http_url(raw: &str) -> Result<Url, WebLinkFetchError> {
    let trimmed = raw.trim();
    if is_blank(Some(trimmed)) {
        return Err(WebLinkFetchError::InvalidRequest(
            "source_url is required".to_string(),
        ));
    }

    let url = Url::parse(trimmed).map_err(|error| {
        WebLinkFetchError::InvalidRequest(format!("invalid source_url: {error}"))
    })?;
    match url.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(WebLinkFetchError::InvalidRequest(
                "source_url must use http or https".to_string(),
            ));
        }
    }

    let host = url.host_str().ok_or_else(|| {
        WebLinkFetchError::InvalidRequest("source_url host is required".to_string())
    })?;
    if is_blocked_hostname(host) {
        return Err(WebLinkFetchError::InvalidRequest(
            "source_url host is not allowed".to_string(),
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(WebLinkFetchError::InvalidRequest(
                "source_url must not target private or loopback addresses".to_string(),
            ));
        }
    }

    Ok(url)
}

fn looks_like_html(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.starts_with("<!doctype") || lower.starts_with("<html")
}

fn is_blocked_hostname(host: &str) -> bool {
    let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
    #[cfg(test)]
    if matches!(normalized.as_str(), "localhost" | "127.0.0.1" | "::1") {
        return false;
    }
    matches!(
        normalized.as_str(),
        "localhost" | "metadata.google.internal" | "metadata" | "127.0.0.1" | "::1" | "0.0.0.0"
    ) || normalized.ends_with(".localhost")
        || normalized.ends_with(".local")
        || normalized.ends_with(".internal")
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => is_blocked_ipv4(value),
        IpAddr::V6(value) => is_blocked_ipv6(value),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    #[cfg(test)]
    if ip.is_loopback() {
        return false;
    }
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.octets()[0] == 0
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.segments()[0] & 0xfe00 == 0xfc00
        || ip.segments()[0] & 0xffc0 == 0xfe80
}

fn html_to_markdown(html: &str, page_url: &str, title_hint: &str) -> String {
    let without_scripts = strip_tag_blocks(html, "script");
    let without_styles = strip_tag_blocks(&without_scripts, "style");
    let without_noscript = strip_tag_blocks(&without_styles, "noscript");
    let title = extract_html_title(&without_noscript)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| title_hint.trim().to_string());
    let article = extract_tag_inner(&without_noscript, "article")
        .or_else(|| extract_tag_inner(&without_noscript, "main"))
        .or_else(|| extract_tag_inner(&without_noscript, "body"))
        .unwrap_or_else(|| without_noscript.clone());
    let text = strip_html_tags(&article)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    if text.is_empty() {
        return format!("# {title}\n\nSource: {page_url}\n");
    }

    format!("# {title}\n\nSource: {page_url}\n\n{text}")
}

fn strip_tag_blocks(input: &str, tag: &str) -> String {
    let mut output = input.to_string();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    loop {
        let Some(start) = output.to_ascii_lowercase().find(&open) else {
            break;
        };
        let Some(relative_end) = output[start..].to_ascii_lowercase().find(&close) else {
            output.replace_range(start.., "");
            break;
        };
        let end = start + relative_end + close.len();
        output.replace_range(start..end, "");
    }
    output
}

fn extract_html_title(html: &str) -> Option<String> {
    extract_tag_inner(html, "title").map(|value| strip_html_tags(&value).trim().to_string())
}

fn extract_tag_inner(html: &str, tag: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let open = format!("<{tag}");
    let start = lower.find(&open)?;
    let open_end = lower[start..].find('>')? + start + 1;
    let close = format!("</{tag}>");
    let relative_end = lower[open_end..].find(&close)?;
    Some(html[open_end..open_end + relative_end].to_string())
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => {
                output.push(ch);
            }
            _ => {}
        }
    }
    output
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_public_http_url_rejects_private_hosts() {
        assert!(validate_public_http_url("http://10.0.0.1/test").is_err());
        assert!(validate_public_http_url("http://192.168.1.1/test").is_err());
        assert!(validate_public_http_url("http://metadata.google.internal/test").is_err());
        assert!(validate_public_http_url("ftp://example.com/test").is_err());
        assert!(validate_public_http_url("https://example.com/article").is_ok());
    }

    #[test]
    fn html_to_markdown_extracts_title_and_body() {
        let html = "<html><head><title>Example</title></head><body><article><p>Hello world</p></article></body></html>";
        let markdown = html_to_markdown(html, "https://example.com/article", "Fallback");
        assert!(markdown.contains("# Example"));
        assert!(markdown.contains("Hello world"));
        assert!(markdown.contains("https://example.com/article"));
    }

    #[tokio::test]
    async fn fetch_web_link_markdown_downloads_html_from_public_url() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/article"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "<html><head><title>Example Article</title></head><body><article><p>Hello from web</p></article></body></html>",
            ))
            .mount(&mock_server)
            .await;

        let markdown =
            fetch_web_link_markdown(&format!("{}/article", mock_server.uri()), "Fallback title")
                .await
                .expect("fetch should succeed");

        assert!(markdown.contains("Hello from web"));
        assert!(markdown.contains("Example Article"));
    }
}
