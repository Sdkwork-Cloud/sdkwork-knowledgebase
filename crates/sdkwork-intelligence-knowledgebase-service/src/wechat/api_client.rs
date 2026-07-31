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
const DEFAULT_THUMB_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MediaUploadResponse {
    media_id: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DraftAddResponse {
    media_id: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PreviewResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MassSendResponse {
    msg_id: Option<i64>,
    msg_data_id: Option<i64>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WechatDraftArticle {
    pub title: String,
    pub author: String,
    pub digest: String,
    pub content: String,
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

    pub async fn upload_thumb_media(
        &self,
        access_token: &str,
    ) -> Result<String, WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/material/add_material?access_token={}&type=thumb",
            urlencoding::encode(access_token),
        ))?;
        let form = reqwest::multipart::Form::new().part(
            "media",
            reqwest::multipart::Part::bytes(DEFAULT_THUMB_PNG.to_vec())
                .file_name("thumb.png")
                .mime_str("image/png")
                .map_err(redacted_reqwest_error)?,
        );
        let response = self
            .http()?
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: MediaUploadResponse = parse_wechat_json(response).await?;
        if let Some(media_id) = body.media_id.filter(|value| !value.is_empty()) {
            return Ok(media_id);
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat thumb upload failed with code {:?}", body.errcode),
        )))
    }

    pub async fn add_draft_articles(
        &self,
        access_token: &str,
        thumb_media_id: &str,
        articles: &[WechatDraftArticle],
    ) -> Result<String, WechatApiClientError> {
        if articles.is_empty() {
            return Err(WechatApiClientError::InvalidRequest(
                "at least one WeChat draft article is required".to_string(),
            ));
        }
        let url = build_wechat_url(&format!(
            "/cgi-bin/draft/add?access_token={}",
            urlencoding::encode(access_token),
        ))?;
        let payload = build_draft_payload(thumb_media_id, articles);
        let response = self
            .http()?
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: DraftAddResponse = parse_wechat_json(response).await?;
        if let Some(media_id) = body.media_id.filter(|value| !value.is_empty()) {
            return Ok(media_id);
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat draft add failed with code {:?}", body.errcode),
        )))
    }

    pub async fn preview_mpnews(
        &self,
        access_token: &str,
        to_wxname: &str,
        media_id: &str,
    ) -> Result<(), WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/message/mass/preview?access_token={}",
            urlencoding::encode(access_token),
        ))?;
        let payload = serde_json::json!({
            "towxname": to_wxname,
            "msgtype": "mpnews",
            "mpnews": {
                "media_id": media_id
            }
        });
        let response = self
            .http()?
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: PreviewResponse = parse_wechat_json(response).await?;
        if body.errcode == Some(0) {
            return Ok(());
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat preview failed with code {:?}", body.errcode),
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

    pub async fn mass_send_mpnews(
        &self,
        access_token: &str,
        media_id: &str,
        send_to_all: bool,
        tag_id: Option<i64>,
    ) -> Result<(), WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/message/mass/sendall?access_token={}",
            urlencoding::encode(access_token),
        ))?;
        let filter = if send_to_all {
            serde_json::json!({ "is_to_all": true })
        } else {
            serde_json::json!({
                "is_to_all": false,
                "tag_id": tag_id.ok_or_else(|| {
                    WechatApiClientError::InvalidRequest(
                        "tag_id is required when send_to_all is false".to_string(),
                    )
                })?,
            })
        };
        let payload = serde_json::json!({
            "filter": filter,
            "msgtype": "mpnews",
            "mpnews": {
                "media_id": media_id
            },
            "send_ignore_reprint": 0
        });
        let response = self
            .http()?
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(redacted_reqwest_error)?;
        let body: MassSendResponse = parse_wechat_json(response).await?;
        if body.errcode == Some(0) {
            let _message_ids = (&body.msg_id, &body.msg_data_id);
            return Ok(());
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat mass send failed with code {:?}", body.errcode),
        )))
    }
}

fn build_draft_payload(thumb_media_id: &str, articles: &[WechatDraftArticle]) -> serde_json::Value {
    let articles = articles
        .iter()
        .map(|article| {
            serde_json::json!({
                "title": article.title,
                "author": article.author,
                "digest": article.digest,
                "content": article.content,
                "content_source_url": "",
                "thumb_media_id": thumb_media_id,
                "need_open_comment": 0,
                "only_fans_can_comment": 0
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "articles": articles })
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
    fn draft_payload_contains_every_selected_article() {
        let payload = build_draft_payload(
            "thumb-1",
            &[
                WechatDraftArticle {
                    title: "First".to_string(),
                    author: "Author".to_string(),
                    digest: "First digest".to_string(),
                    content: "<p>First</p>".to_string(),
                },
                WechatDraftArticle {
                    title: "Second".to_string(),
                    author: "Author".to_string(),
                    digest: "Second digest".to_string(),
                    content: "<p>Second</p>".to_string(),
                },
            ],
        );

        let articles = payload["articles"]
            .as_array()
            .expect("draft articles array");
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0]["title"], "First");
        assert_eq!(articles[1]["title"], "Second");
        assert!(articles
            .iter()
            .all(|article| article["thumb_media_id"] == "thumb-1"));
    }

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
