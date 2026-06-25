use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::wechat::api_client::{WechatApiClient, WechatApiClientError};
use crate::wechat::config_store::WechatConfigStore;
use sdkwork_knowledgebase_contract::wechat::{
    KnowledgeWechatApplet, KnowledgeWechatArticlesPreviewRequest,
    KnowledgeWechatArticlesPublishRequest, KnowledgeWechatOfficialAccount,
    KnowledgeWechatOperationResult,
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

    pub async fn publish_articles(
        &self,
        request: KnowledgeWechatArticlesPublishRequest,
    ) -> Result<KnowledgeWechatOperationResult, KnowledgeWechatServiceError> {
        validate_publish_request(&request)?;
        let mut draft_count = 0u32;
        for account_id in &request.account_ids {
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
            let access_token = self
                .api_client
                .fetch_access_token(&account.app_id, app_secret)
                .await?;
            let thumb_media_id = self.api_client.upload_thumb_media(&access_token).await?;
            for article in &request.articles {
                let content = article.content.clone().unwrap_or_default();
                if is_blank(Some(content.as_str())) {
                    return Err(KnowledgeWechatServiceError::InvalidRequest(
                        "article content must not be empty".to_string(),
                    ));
                }
                self.api_client
                    .add_draft_article(
                        &access_token,
                        &thumb_media_id,
                        &article.title,
                        &article.author,
                        article.r#abstract.as_deref().unwrap_or(""),
                        &content,
                    )
                    .await?;
                draft_count += 1;
            }
        }
        Ok(KnowledgeWechatOperationResult {
            success: true,
            message: format!(
                "Created {draft_count} WeChat draft article(s) for {} account(s).",
                request.account_ids.len()
            ),
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
        let account = self
            .config_store
            .find_official_account(&request.account_id)
            .await
            .map_err(KnowledgeWechatServiceError::Storage)?
            .ok_or_else(|| {
                KnowledgeWechatServiceError::InvalidRequest(format!(
                    "official account was not found: {}",
                    request.account_id
                ))
            })?;
        let app_secret = account.app_secret.as_deref().ok_or_else(|| {
            KnowledgeWechatServiceError::InvalidRequest(format!(
                "official account {} is missing appSecret",
                request.account_id
            ))
        })?;
        let access_token = self
            .api_client
            .fetch_access_token(&account.app_id, app_secret)
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
            success: true,
            message: format!(
                "Preview sent to {} recipient(s) for account {}.",
                request.wechat_ids.len(),
                request.account_id
            ),
        })
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

#[derive(Debug, Error)]
pub enum KnowledgeWechatServiceError {
    #[error("invalid wechat request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
    #[error(transparent)]
    Api(#[from] WechatApiClientError),
}
