use crate::bounded_http_body::{
    read_bounded_http_body, redacted_reqwest_error_detail, BoundedHttpBodyError,
};
use reqwest::Client;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

const WECHAT_API_HOST: &str = "api.weixin.qq.com";
const WECHAT_API_TIMEOUT_SECS: u64 = 30;
const MAX_WECHAT_JSON_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    tags: Option<Vec<TagEntry>>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TagEntry {
    id: Option<i64>,
    name: Option<String>,
    count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatUserTag {
    pub id: i64,
    pub name: String,
    pub count: u64,
}

pub struct WechatApiClient {
    http: Result<Client, String>,
}

impl Default for WechatApiClient {
    fn default() -> Self {
        Self {
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(WECHAT_API_TIMEOUT_SECS))
                .build()
                .map_err(|error| redacted_reqwest_error_detail(&error)),
        }
    }
}

impl WechatApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    fn http(&self) -> Result<&Client, WechatApiClientError> {
        self.http
            .as_ref()
            .map_err(|detail| WechatApiClientError::Configuration(detail.clone()))
    }

    pub async fn fetch_access_token(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> Result<String, WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            urlencoding::encode(app_id),
            urlencoding::encode(app_secret),
        ))?;
        let response = self
            .http()?
            .get(url)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: AccessTokenResponse = parse_wechat_json(response).await?;
        if let Some(token) = body.access_token.filter(|value| !value.is_empty()) {
            return Ok(token);
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat token request failed with code {:?}", body.errcode),
        )))
    }

    pub async fn list_user_tags(
        &self,
        access_token: &str,
    ) -> Result<Vec<WechatUserTag>, WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/tags/get?access_token={}",
            urlencoding::encode(access_token),
        ))?;
        let response = self
            .http()?
            .get(url)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: TagsResponse = parse_wechat_json(response).await?;
        if let Some(errcode) = body.errcode.filter(|code| *code != 0) {
            return Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
                || format!("wechat tag list failed with code {errcode}"),
            )));
        }
        Ok(body
            .tags
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| {
                let id = entry.id?;
                let name = entry.name.filter(|value| !value.is_empty())?;
                Some(WechatUserTag {
                    id,
                    name,
                    count: entry.count.unwrap_or(0),
                })
            })
            .collect())
    }
}

fn build_wechat_url(path_and_query: &str) -> Result<Url, WechatApiClientError> {
    let url = Url::parse(&format!("https://{WECHAT_API_HOST}{path_and_query}")).map_err(|_| {
        WechatApiClientError::InvalidRequest(
            "failed to construct allowlisted WeChat API URL".to_string(),
        )
    })?;
    if url.host_str() != Some(WECHAT_API_HOST) {
        return Err(WechatApiClientError::InvalidRequest(
            "wechat api host is not allowlisted".to_string(),
        ));
    }
    Ok(url)
}

fn redacted_reqwest_error(error: reqwest::Error) -> WechatApiClientError {
    WechatApiClientError::Http(redacted_reqwest_error_detail(&error))
}

async fn parse_wechat_json<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, WechatApiClientError> {
    let body = read_bounded_http_body(response, MAX_WECHAT_JSON_RESPONSE_BYTES)
        .await
        .map_err(|error| match error {
            BoundedHttpBodyError::TooLarge { max_bytes } => {
                WechatApiClientError::Api(format!("wechat response exceeds {max_bytes} bytes"))
            }
            BoundedHttpBodyError::Read { detail } => WechatApiClientError::Http(detail),
        })?;
    serde_json::from_slice(&body)
        .map_err(|_| WechatApiClientError::Http("upstream response decoding failed".to_string()))
}

#[derive(Debug, Error)]
pub enum WechatApiClientError {
    #[error("invalid wechat api request: {0}")]
    InvalidRequest(String),
    #[error("wechat api client configuration failed: {0}")]
    Configuration(String),
    #[error("wechat api call failed: {0}")]
    Api(String),
    #[error("wechat api transport failed: {0}")]
    Http(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_errors_are_rendered_without_request_urls_or_credentials() {
        let error = Client::new()
            .get("http://[::1?access_token=super-secret")
            .build()
            .expect_err("invalid URL must fail request construction");

        let rendered = redacted_reqwest_error(error).to_string();

        assert!(!rendered.contains("super-secret"));
        assert!(!rendered.contains("access_token"));
        assert!(!rendered.contains("http://"));
    }
}
