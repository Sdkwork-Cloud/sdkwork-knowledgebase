use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

const WECHAT_API_HOST: &str = "api.weixin.qq.com";
const WECHAT_API_TIMEOUT_SECS: u64 = 30;
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

pub struct WechatApiClient {
    http: Client,
}

impl Default for WechatApiClient {
    fn default() -> Self {
        Self {
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(WECHAT_API_TIMEOUT_SECS))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

impl WechatApiClient {
    pub fn new() -> Self {
        Self::default()
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
        let response = self.http.get(url).send().await?;
        let body: AccessTokenResponse = response.json().await?;
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
                .mime_str("image/png")?,
        );
        let response = self.http.post(url).multipart(form).send().await?;
        let body: MediaUploadResponse = response.json().await?;
        if let Some(media_id) = body.media_id.filter(|value| !value.is_empty()) {
            return Ok(media_id);
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat thumb upload failed with code {:?}", body.errcode),
        )))
    }

    pub async fn add_draft_article(
        &self,
        access_token: &str,
        thumb_media_id: &str,
        title: &str,
        author: &str,
        digest: &str,
        content: &str,
    ) -> Result<String, WechatApiClientError> {
        let url = build_wechat_url(&format!(
            "/cgi-bin/draft/add?access_token={}",
            urlencoding::encode(access_token),
        ))?;
        let payload = serde_json::json!({
            "articles": [{
                "title": title,
                "author": author,
                "digest": digest,
                "content": content,
                "content_source_url": "",
                "thumb_media_id": thumb_media_id,
                "need_open_comment": 0,
                "only_fans_can_comment": 0
            }]
        });
        let response = self.http.post(url).json(&payload).send().await?;
        let body: DraftAddResponse = response.json().await?;
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
        let response = self.http.post(url).json(&payload).send().await?;
        let body: PreviewResponse = response.json().await?;
        if body.errcode.unwrap_or(0) == 0 {
            return Ok(());
        }
        Err(WechatApiClientError::Api(body.errmsg.unwrap_or_else(
            || format!("wechat preview failed with code {:?}", body.errcode),
        )))
    }
}

fn build_wechat_url(path_and_query: &str) -> Result<Url, WechatApiClientError> {
    let url = Url::parse(&format!("https://{WECHAT_API_HOST}{path_and_query}"))
        .map_err(|error| WechatApiClientError::InvalidRequest(error.to_string()))?;
    if url.host_str() != Some(WECHAT_API_HOST) {
        return Err(WechatApiClientError::InvalidRequest(
            "wechat api host is not allowlisted".to_string(),
        ));
    }
    Ok(url)
}

#[derive(Debug, Error)]
pub enum WechatApiClientError {
    #[error("invalid wechat api request: {0}")]
    InvalidRequest(String),
    #[error("wechat api call failed: {0}")]
    Api(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
