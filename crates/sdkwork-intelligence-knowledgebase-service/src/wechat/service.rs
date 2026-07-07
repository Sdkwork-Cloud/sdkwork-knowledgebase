use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::wechat::api_client::{WechatApiClient, WechatApiClientError};
use crate::wechat::config_store::WechatConfigStore;
use sdkwork_knowledgebase_contract::wechat::{
    KnowledgeWechatApplet, KnowledgeWechatArticlesPreviewRequest,
    KnowledgeWechatArticlesPublishRequest, KnowledgeWechatFanTag, KnowledgeWechatFanTagList,
    KnowledgeWechatOfficialAccount, KnowledgeWechatOperationResult,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

pub struct KnowledgeWechatService<'a> {
    config_store: WechatConfigStore<'a>,
    api_client: WechatApiClient,
}

impl<'a> KnowledgeWechatService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage, tenant_id: &str) -> Self {
        Self {
            config_store: WechatConfigStore::new(drive, tenant_id),
            api_client: WechatApiClient::new(),
        }
    }

    pub async fn list_official_accounts(
        &self,
    ) -> Result<Vec<KnowledgeWechatOfficialAccount>, KnowledgeWechatServiceError> {
        self.config_store
            .load_official_accounts()
            .await
            .map_err(KnowledgeWechatServiceError::Storage)
    }

    pub async fn replace_official_accounts(
        &self,
        accounts: Vec<KnowledgeWechatOfficialAccount>,
    ) -> Result<Vec<KnowledgeWechatOfficialAccount>, KnowledgeWechatServiceError> {
        self.config_store
            .replace_official_accounts(accounts)
            .await
            .map_err(KnowledgeWechatServiceError::Storage)
    }

    pub async fn list_applets(
        &self,
    ) -> Result<Vec<KnowledgeWechatApplet>, KnowledgeWechatServiceError> {
        self.config_store
            .load_applets()
            .await
            .map_err(KnowledgeWechatServiceError::Storage)
    }

    pub async fn replace_applets(
        &self,
        applets: Vec<KnowledgeWechatApplet>,
    ) -> Result<Vec<KnowledgeWechatApplet>, KnowledgeWechatServiceError> {
        self.config_store
            .replace_applets(applets)
            .await
            .map_err(KnowledgeWechatServiceError::Storage)
    }

    pub async fn list_fan_tags(
        &self,
        account_id: &str,
    ) -> Result<KnowledgeWechatFanTagList, KnowledgeWechatServiceError> {
        if is_blank(Some(account_id)) {
            return Err(KnowledgeWechatServiceError::InvalidRequest(
                "accountId is required".to_string(),
            ));
        }
        let access_token = self.resolve_account_access_token(account_id).await?;
        let tags = self.api_client.list_user_tags(&access_token).await?;
        Ok(KnowledgeWechatFanTagList {
            tags: tags
                .into_iter()
                .map(|tag| KnowledgeWechatFanTag {
                    id: tag.id.to_string(),
                    name: tag.name,
                    fan_count: tag.count,
                })
                .collect(),
        })
    }

    pub async fn publish_articles(
        &self,
        request: KnowledgeWechatArticlesPublishRequest,
    ) -> Result<KnowledgeWechatOperationResult, KnowledgeWechatServiceError> {
        validate_publish_request(&request)?;
        if !is_blank(request.schedule_time.as_deref()) {
            return Err(KnowledgeWechatServiceError::InvalidRequest(
                "scheduleTime is not supported; publish immediately or save drafts without scheduling"
                    .to_string(),
            ));
        }

        let send_notification = request.send_notification.unwrap_or(false);
        let group_notification = request.group_notification.unwrap_or(false);
        let tag_id = resolve_fan_tag_id(request.selected_group_id.as_deref(), group_notification)?;

        let mut draft_count = 0u32;
        let mut mass_send_count = 0u32;
        for account_id in &request.account_ids {
            let access_token = self.resolve_account_access_token(account_id).await?;
            let thumb_media_id = self.api_client.upload_thumb_media(&access_token).await?;
            let mut last_media_id: Option<String> = None;
            for article in &request.articles {
                let content = article.content.clone().unwrap_or_default();
                if is_blank(Some(content.as_str())) {
                    return Err(KnowledgeWechatServiceError::InvalidRequest(
                        "article content must not be empty".to_string(),
                    ));
                }
                let media_id = self
                    .api_client
                    .add_draft_article(
                        &access_token,
                        &thumb_media_id,
                        &article.title,
                        &article.author,
                        article.r#abstract.as_deref().unwrap_or(""),
                        &content,
                    )
                    .await?;
                last_media_id = Some(media_id);
                draft_count += 1;
            }

            if send_notification {
                let media_id = last_media_id.ok_or_else(|| {
                    KnowledgeWechatServiceError::InvalidRequest(
                        "no draft media_id available for mass send".to_string(),
                    )
                })?;
                self.api_client
                    .mass_send_mpnews(&access_token, &media_id, tag_id.is_none(), tag_id)
                    .await?;
                mass_send_count += 1;
            }
        }

        let _ = (draft_count, mass_send_count);
        Ok(KnowledgeWechatOperationResult {
            accepted: true,
            status: "completed".to_string(),
        })
    }

    pub async fn preview_articles(
        &self,
        request: KnowledgeWechatArticlesPreviewRequest,
    ) -> Result<KnowledgeWechatOperationResult, KnowledgeWechatServiceError> {
        if is_blank(Some(request.account_id.as_str())) || request.wechat_ids.is_empty() {
            return Err(KnowledgeWechatServiceError::InvalidRequest(
                "accountId and wechatIds are required".to_string(),
            ));
        }
        if request.articles.is_empty() {
            return Err(KnowledgeWechatServiceError::InvalidRequest(
                "at least one article is required".to_string(),
            ));
        }
        let access_token = self
            .resolve_account_access_token(&request.account_id)
            .await?;
        let thumb_media_id = self.api_client.upload_thumb_media(&access_token).await?;
        let article = &request.articles[0];
        let content = article.content.clone().unwrap_or_default();
        if is_blank(Some(content.as_str())) {
            return Err(KnowledgeWechatServiceError::InvalidRequest(
                "article content must not be empty".to_string(),
            ));
        }
        let media_id = self
            .api_client
            .add_draft_article(
                &access_token,
                &thumb_media_id,
                &article.title,
                &article.author,
                article.r#abstract.as_deref().unwrap_or(""),
                &content,
            )
            .await?;
        for recipient in &request.wechat_ids {
            self.api_client
                .preview_mpnews(&access_token, recipient, &media_id)
                .await?;
        }
        Ok(KnowledgeWechatOperationResult {
            accepted: true,
            status: "completed".to_string(),
        })
    }

    async fn resolve_account_access_token(
        &self,
        account_id: &str,
    ) -> Result<String, KnowledgeWechatServiceError> {
        let account = self
            .config_store
            .find_official_account(account_id)
            .await
            .map_err(KnowledgeWechatServiceError::Storage)?
            .ok_or_else(|| {
                KnowledgeWechatServiceError::InvalidRequest(format!(
                    "official account was not found: {account_id}"
                ))
            })?;
        let app_secret = account.app_secret.as_deref().ok_or_else(|| {
            KnowledgeWechatServiceError::InvalidRequest(format!(
                "official account {account_id} is missing appSecret"
            ))
        })?;
        self.api_client
            .fetch_access_token(&account.app_id, app_secret)
            .await
            .map_err(KnowledgeWechatServiceError::from)
    }
}

fn validate_publish_request(
    request: &KnowledgeWechatArticlesPublishRequest,
) -> Result<(), KnowledgeWechatServiceError> {
    if request.account_ids.is_empty() || request.articles.is_empty() {
        return Err(KnowledgeWechatServiceError::InvalidRequest(
            "accountIds and articles are required".to_string(),
        ));
    }
    Ok(())
}

fn resolve_fan_tag_id(
    selected_group_id: Option<&str>,
    group_notification: bool,
) -> Result<Option<i64>, KnowledgeWechatServiceError> {
    if !group_notification {
        return Ok(None);
    }
    let Some(group_id) = selected_group_id.filter(|value| !is_blank(Some(value))) else {
        return Err(KnowledgeWechatServiceError::InvalidRequest(
            "selectedGroupId is required when groupNotification is enabled".to_string(),
        ));
    };
    if group_id.eq_ignore_ascii_case("all") {
        return Ok(None);
    }
    group_id.parse::<i64>().map(Some).map_err(|_| {
        KnowledgeWechatServiceError::InvalidRequest(format!(
            "selectedGroupId must be 'all' or a numeric WeChat tag id, got {group_id}"
        ))
    })
}

#[derive(Debug, Error)]
pub enum KnowledgeWechatServiceError {
    #[error("invalid wechat request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
    #[error(transparent)]
    Api(#[from] WechatApiClientError),
}
