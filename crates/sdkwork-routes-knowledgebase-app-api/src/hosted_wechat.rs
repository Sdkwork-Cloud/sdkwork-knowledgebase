use async_trait::async_trait;

use sdkwork_intelligence_knowledgebase_service::wechat::KnowledgeWechatService;
use sdkwork_knowledgebase_contract::wechat::{
    KnowledgeWechatAppletList, KnowledgeWechatArticlesPreviewRequest,
    KnowledgeWechatArticlesPublishRequest, KnowledgeWechatFanTagList,
    KnowledgeWechatOfficialAccountList, KnowledgeWechatOperationResult,
    KnowledgeWechatReplaceAppletsRequest, KnowledgeWechatReplaceOfficialAccountsRequest,
};

use crate::{
    hosted_access::ensure_runtime_tenant, runtime::KnowledgebaseRuntime, ApiError, ApiResult,
    KnowledgeAppRequestContext, KnowledgeWechatAppService,
};

#[derive(Clone)]
pub(crate) struct HostedWechatService {
    runtime: KnowledgebaseRuntime,
}

impl HostedWechatService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    fn service(&self) -> KnowledgeWechatService<'_> {
        KnowledgeWechatService::new(self.runtime.drive_storage(), self.runtime.tenant_id_str())
    }
}

#[async_trait]
impl KnowledgeWechatAppService for HostedWechatService {
    async fn list_official_accounts(
        &self,
        context: KnowledgeAppRequestContext,
    ) -> ApiResult<KnowledgeWechatOfficialAccountList> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let accounts = self
            .service()
            .list_official_accounts()
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeWechatOfficialAccountList { accounts })
    }

    async fn replace_official_accounts(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeWechatReplaceOfficialAccountsRequest,
    ) -> ApiResult<KnowledgeWechatOfficialAccountList> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let accounts = self
            .service()
            .replace_official_accounts(request.accounts)
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeWechatOfficialAccountList { accounts })
    }

    async fn list_official_account_fan_tags(
        &self,
        context: KnowledgeAppRequestContext,
        account_id: String,
    ) -> ApiResult<KnowledgeWechatFanTagList> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.service()
            .list_fan_tags(&account_id)
            .await
            .map_err(ApiError::from)
    }

    async fn list_applets(
        &self,
        context: KnowledgeAppRequestContext,
    ) -> ApiResult<KnowledgeWechatAppletList> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let applets = self
            .service()
            .list_applets()
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeWechatAppletList { applets })
    }

    async fn replace_applets(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeWechatReplaceAppletsRequest,
    ) -> ApiResult<KnowledgeWechatAppletList> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let applets = self
            .service()
            .replace_applets(request.applets)
            .await
            .map_err(ApiError::from)?;
        Ok(KnowledgeWechatAppletList { applets })
    }

    async fn publish_articles(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeWechatArticlesPublishRequest,
    ) -> ApiResult<KnowledgeWechatOperationResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.service()
            .publish_articles(request)
            .await
            .map_err(ApiError::from)
    }

    async fn preview_articles(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeWechatArticlesPreviewRequest,
    ) -> ApiResult<KnowledgeWechatOperationResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        self.service()
            .preview_articles(request)
            .await
            .map_err(ApiError::from)
    }
}
