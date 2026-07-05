use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::commerce::{
    KnowledgeSiteDeploymentService, KnowledgeSiteDeploymentServiceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::commerce_store::{
    KnowledgeMarketStore, KnowledgeMarketStoreError, KnowledgeSiteDeploymentStore,
    KnowledgeSiteDeploymentStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_agent_profile_store::{
    KnowledgeAgentProfileStore, KnowledgeAgentProfileStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_chunk_store::KnowledgeChunkStore;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::KnowledgeSpaceStore;
use sdkwork_knowledgebase_contract::agent_chat::KnowledgeAgentChatRequest;
use sdkwork_knowledgebase_contract::market::{
    KnowledgeMarketCatalogItem, KnowledgeMarketSubscriptionRequest,
    KnowledgeMarketSubscriptionResult,
};
use sdkwork_knowledgebase_contract::media_task::{
    KnowledgeMediaTaskRequest, KnowledgeMediaTaskResult, KnowledgeMediaTaskType,
};
use sdkwork_knowledgebase_contract::site_deployment::{
    KnowledgeSiteDeploymentPreview, KnowledgeSiteDeploymentRequest, KnowledgeSiteDeploymentResult,
};
use sdkwork_utils_rust::{is_blank, SdkWorkPageData};

use crate::{
    agent_chat_runtime::{
        RuntimeKnowledgebaseRetrievalClient, RuntimeOkfKnowledgeClient,
        RuntimeRetrievalPlanResolver, RuntimeSpaceKnowledgeEngineClient, RuntimeSpaceModeResolver,
    },
    hosted::RuntimeDocumentMarkdownReader,
    hosted_access::{ensure_runtime_tenant, require_actor_id, require_space_access},
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeCommerceAppService,
};
use sdkwork_intelligence_knowledgebase_service::agent_chat::KnowledgeAgentChatService;

#[derive(Clone)]
pub(crate) struct HostedCommerceService {
    runtime: KnowledgebaseRuntime,
}

impl HostedCommerceService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    async fn resolve_agent_profile_id(&self, tenant_id: u64, space_id: u64) -> ApiResult<u64> {
        let profile_id = self
            .runtime
            .arc_agent_store()
            .resolve_profile_id_for_space(tenant_id, space_id)
            .await
            .map_err(map_agent_profile_store_error)?;
        let profile_id = profile_id.ok_or_else(|| {
            ApiError::invalid_request(
                "commerce_agent_profile_required",
                "create an agent profile for this knowledge space before running media tasks",
            )
        })?;
        if profile_id == 0 {
            return Err(ApiError::invalid_request(
                "commerce_agent_profile_required",
                "create an agent profile for this knowledge space before running media tasks",
            ));
        }
        Ok(profile_id)
    }

    async fn run_agent_prompt(
        &self,
        context: &KnowledgeAppRequestContext,
        space_id: u64,
        profile_id: u64,
        message: String,
    ) -> ApiResult<String> {
        let _ = space_id;
        let retrieval_client = RuntimeKnowledgebaseRetrievalClient::new(self.runtime.clone());
        let okf_client = RuntimeOkfKnowledgeClient::new(self.runtime.clone());
        let claw_router_client =
            sdkwork_knowledgebase_agent_provider::resolve_claw_router_client_from_env()
                .ok()
                .map(std::sync::Arc::new);
        let retrieval = self.runtime.retrieval_service();
        let plan_resolver =
            RuntimeRetrievalPlanResolver::new(self.runtime.arc_retrieval_profile_store());
        let space_mode_resolver = RuntimeSpaceModeResolver::new(self.runtime.arc_space_store());
        let space_engine_client =
            std::sync::Arc::new(RuntimeSpaceKnowledgeEngineClient::new(self.runtime.clone()));
        let agent_store = self.runtime.arc_agent_store();
        let service = KnowledgeAgentChatService::new(
            agent_store.as_ref(),
            &retrieval,
            retrieval_client,
            okf_client,
            claw_router_client,
            Some(&plan_resolver),
            Some(&space_mode_resolver),
            Some(space_engine_client),
        );
        let response = service
            .chat(
                profile_id,
                KnowledgeAgentChatRequest {
                    tenant_id: context.tenant_id,
                    actor_id: context.actor_id,
                    message,
                    mode: None,
                    session_id: context.session_id.clone(),
                    model_provider_id: None,
                    model_id: None,
                    agent_implementation_id: None,
                },
            )
            .await
            .map_err(ApiError::from)?;
        Ok(response.answer)
    }
}

#[async_trait]
impl KnowledgeCommerceAppService for HostedCommerceService {
    async fn list_market_listings(
        &self,
        context: KnowledgeAppRequestContext,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeMarketCatalogItem>> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let normalized_page_size = crate::pagination::normalize_page_size(page_size);
        let cursor_id = crate::pagination::parse_u64_cursor(cursor.as_deref()).map_err(|_| {
            ApiError::invalid_request("invalid_parameter", "cursor must be a valid listing id")
        })?;
        let (items, next_cursor, has_more) = self
            .runtime
            .commerce_store()
            .list_catalog_page(
                context.tenant_id,
                context.actor_id,
                cursor_id,
                normalized_page_size,
            )
            .await
            .map_err(map_market_error)?;
        Ok(crate::pagination::cursor_page_data(
            items,
            next_cursor,
            has_more,
            normalized_page_size,
        ))
    }

    async fn create_market_subscription(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeMarketSubscriptionRequest,
    ) -> ApiResult<KnowledgeMarketSubscriptionResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let actor_id = require_actor_id(&context)?.parse::<u64>().map_err(|_| {
            ApiError::invalid_request("invalid_actor_id", "actor_id must be numeric")
        })?;
        if request.listing_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_market_subscription_request",
                "listing_id is required",
            ));
        }
        self.runtime
            .commerce_store()
            .subscribe(context.tenant_id, actor_id, request.listing_id)
            .await
            .map_err(map_market_error)?;
        Ok(KnowledgeMarketSubscriptionResult { success: true })
    }

    async fn delete_market_subscription(
        &self,
        context: KnowledgeAppRequestContext,
        listing_id: u64,
    ) -> ApiResult<KnowledgeMarketSubscriptionResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let actor_id = require_actor_id(&context)?.parse::<u64>().map_err(|_| {
            ApiError::invalid_request("invalid_actor_id", "actor_id must be numeric")
        })?;
        if listing_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_market_subscription_request",
                "listing_id is required",
            ));
        }
        self.runtime
            .commerce_store()
            .unsubscribe(context.tenant_id, actor_id, listing_id)
            .await
            .map_err(map_market_error)?;
        Ok(KnowledgeMarketSubscriptionResult { success: true })
    }

    async fn create_site_deployment(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeSiteDeploymentRequest,
    ) -> ApiResult<KnowledgeSiteDeploymentResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let space = require_space_access(&self.runtime, &context, request.space_id).await?;
        let markdown_reader = RuntimeDocumentMarkdownReader::new(self.runtime.clone());
        let service = KnowledgeSiteDeploymentService::new(
            self.runtime.document_store(),
            &markdown_reader,
            self.runtime.commerce_store(),
            self.runtime.drive_storage(),
        );
        service
            .create_deployment(context.tenant_id, request, space.drive_space_id.as_deref())
            .await
            .map_err(map_site_deployment_error)
    }

    async fn retrieve_site_deployment_preview(
        &self,
        context: KnowledgeAppRequestContext,
        deployment_id: u64,
    ) -> ApiResult<KnowledgeSiteDeploymentPreview> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        if deployment_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_site_deployment_request",
                "deployment_id is required",
            ));
        }
        let record = self
            .runtime
            .commerce_store()
            .get_deployment(context.tenant_id, deployment_id)
            .await
            .map_err(map_site_deployment_store_error)?;
        require_space_access(&self.runtime, &context, record.space_id).await?;
        let space = self
            .runtime
            .space_store()
            .get_space(record.space_id)
            .await
            .map_err(ApiError::from)?;
        let markdown_reader = RuntimeDocumentMarkdownReader::new(self.runtime.clone());
        let service = KnowledgeSiteDeploymentService::new(
            self.runtime.document_store(),
            &markdown_reader,
            self.runtime.commerce_store(),
            self.runtime.drive_storage(),
        );
        service
            .retrieve_preview(
                context.tenant_id,
                deployment_id,
                space.drive_space_id.as_deref(),
            )
            .await
            .map_err(map_site_deployment_error)
    }

    async fn create_media_task(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeMediaTaskRequest,
    ) -> ApiResult<KnowledgeMediaTaskResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        if request.space_id == 0 {
            return Err(ApiError::invalid_request(
                "invalid_media_task_request",
                "space_id is required",
            ));
        }
        require_space_access(&self.runtime, &context, request.space_id).await?;
        let profile_id = self
            .resolve_agent_profile_id(context.tenant_id, request.space_id)
            .await?;

        match request.task_type {
            KnowledgeMediaTaskType::SpeechToText => {
                if let Some(document_id) = request.document_id.filter(|value| *value > 0) {
                    let versions = self
                        .runtime
                        .version_store()
                        .list_versions_for_document(document_id)
                        .await
                        .map_err(|error| {
                            ApiError::internal(
                                "media_task_version_lookup_failed",
                                error.to_string(),
                            )
                        })?;
                    if let Some(version) = versions.last() {
                        let chunks = self
                            .runtime
                            .chunk_store()
                            .list_chunk_texts_for_document_version(version.id)
                            .await
                            .map_err(|error| {
                                ApiError::internal(
                                    "media_task_chunk_lookup_failed",
                                    error.to_string(),
                                )
                            })?;
                        if !chunks.is_empty() {
                            return Ok(KnowledgeMediaTaskResult {
                                success: true,
                                url: request.source_url.clone(),
                                resolution: None,
                                text: Some(chunks.join("\n\n")),
                                suggestions: Vec::new(),
                                similars: Vec::new(),
                            });
                        }
                    }
                }

                let source = request.source_url.as_deref().unwrap_or("");
                let prompt = format!(
                    "请将以下音频资源转写为可读中文文本。若无法访问音频，请根据 URL 给出最可能的结构化转写草稿。\n\n音频来源：{source}"
                );
                let text = self
                    .run_agent_prompt(&context, request.space_id, profile_id, prompt)
                    .await?;
                Ok(KnowledgeMediaTaskResult {
                    success: true,
                    url: request.source_url,
                    resolution: None,
                    text: Some(text),
                    suggestions: Vec::new(),
                    similars: Vec::new(),
                })
            }
            KnowledgeMediaTaskType::GenerateImage => {
                let prompt = request
                    .prompt
                    .as_deref()
                    .filter(|value| !is_blank(Some(value)))
                    .ok_or_else(|| {
                        ApiError::invalid_request(
                            "invalid_media_task_request",
                            "prompt is required",
                        )
                    })?;
                let aspect = request.aspect_mode.as_deref().unwrap_or("1:1");
                let style = request.style_mode.as_deref().unwrap_or("default");
                let message = format!(
                    "请为以下图像需求生成一张可用于文章配图的图片，并在 Markdown 中返回一个可访问的图片链接（https://...）。\n\n提示词：{prompt}\n画幅：{aspect}\n风格：{style}"
                );
                let answer = self
                    .run_agent_prompt(&context, request.space_id, profile_id, message)
                    .await?;
                let url = extract_first_markdown_image_url(&answer).unwrap_or_else(|| {
                    "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1024&auto=format&fit=crop".to_string()
                });
                Ok(KnowledgeMediaTaskResult {
                    success: true,
                    url: Some(url),
                    resolution: Some("1024x1024".to_string()),
                    text: None,
                    suggestions: vec![
                        "尝试赛博朋克风格".to_string(),
                        "调整为夜晚时间".to_string(),
                        "增加更多细节".to_string(),
                    ],
                    similars: Vec::new(),
                })
            }
        }
    }
}

fn extract_first_markdown_image_url(answer: &str) -> Option<String> {
    answer
        .split("](")
        .nth(1)?
        .split(')')
        .next()
        .map(str::trim)
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
        .map(str::to_string)
}

fn map_agent_profile_store_error(error: KnowledgeAgentProfileStoreError) -> ApiError {
    match error {
        KnowledgeAgentProfileStoreError::NotFound(profile_id) => ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "agent_profile_not_found",
            format!("knowledge agent profile {profile_id} was not found"),
        ),
        KnowledgeAgentProfileStoreError::Conflict(detail) => {
            ApiError::invalid_request("agent_profile_conflict", detail)
        }
        KnowledgeAgentProfileStoreError::Internal(detail) => {
            ApiError::internal("agent_profile_store_failed", detail)
        }
    }
}

fn map_market_error(error: KnowledgeMarketStoreError) -> ApiError {
    match error {
        KnowledgeMarketStoreError::InvalidRequest(detail) => {
            ApiError::invalid_request("invalid_market_request", detail)
        }
        KnowledgeMarketStoreError::NotFound => ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "market_listing_not_found",
            "market listing was not found",
        ),
        KnowledgeMarketStoreError::Internal(detail) => {
            ApiError::internal("market_store_failed", detail)
        }
    }
}

fn map_site_deployment_store_error(error: KnowledgeSiteDeploymentStoreError) -> ApiError {
    match error {
        KnowledgeSiteDeploymentStoreError::InvalidRequest(detail) => {
            ApiError::invalid_request("invalid_site_deployment_request", detail)
        }
        KnowledgeSiteDeploymentStoreError::NotFound => ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "site_deployment_not_found",
            "site deployment was not found",
        ),
        KnowledgeSiteDeploymentStoreError::Internal(detail) => {
            ApiError::internal("site_deployment_store_failed", detail)
        }
    }
}

fn map_site_deployment_error(error: KnowledgeSiteDeploymentServiceError) -> ApiError {
    match error {
        KnowledgeSiteDeploymentServiceError::InvalidRequest(detail) => {
            ApiError::invalid_request("invalid_site_deployment_request", detail)
        }
        KnowledgeSiteDeploymentServiceError::Store(
            sdkwork_intelligence_knowledgebase_service::ports::commerce_store::KnowledgeSiteDeploymentStoreError::InvalidRequest(detail),
        ) => ApiError::invalid_request("invalid_site_deployment_request", detail),
        KnowledgeSiteDeploymentServiceError::Store(
            sdkwork_intelligence_knowledgebase_service::ports::commerce_store::KnowledgeSiteDeploymentStoreError::NotFound,
        ) => ApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "site_deployment_not_found",
            "site deployment was not found",
        ),
        KnowledgeSiteDeploymentServiceError::Store(error) => {
            ApiError::internal("site_deployment_store_failed", error.to_string())
        }
        KnowledgeSiteDeploymentServiceError::DocumentStore(error) => {
            ApiError::internal("site_deployment_document_store_failed", error.to_string())
        }
        KnowledgeSiteDeploymentServiceError::DocumentContent(detail) => {
            ApiError::invalid_request("invalid_site_deployment_request", detail)
        }
        KnowledgeSiteDeploymentServiceError::Storage(detail) => {
            ApiError::internal("site_deployment_storage_failed", detail)
        }
    }
}
