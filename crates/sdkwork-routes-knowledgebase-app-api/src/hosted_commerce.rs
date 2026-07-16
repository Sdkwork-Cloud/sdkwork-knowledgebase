use async_trait::async_trait;
use clawrouter_open_sdk::{
    OpenAiAudioTranscriptionRequest, OpenAiFileReferenceInput, OpenAiImageGenerationRequest,
};
use sdkwork_intelligence_knowledgebase_service::commerce::{
    KnowledgeSiteDeploymentService, KnowledgeSiteDeploymentServiceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::commerce_store::{
    KnowledgeMarketStore, KnowledgeMarketStoreError, KnowledgeSiteDeploymentStore,
    KnowledgeSiteDeploymentStoreError, KnowledgeSitePublisher, KnowledgeSitePublisherError,
    PublishKnowledgeSiteRequest, PublishedKnowledgeSite,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_access_control::KnowledgeAccessRole;
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
    hosted::RuntimeDocumentMarkdownReader,
    hosted_access::{
        ensure_runtime_tenant, require_actor_id, require_space_access,
        require_space_access_with_role,
    },
    runtime::KnowledgebaseRuntime,
    ApiError, ApiResult, KnowledgeAppRequestContext, KnowledgeCommerceAppService,
};

#[derive(Clone)]
pub(crate) struct HostedCommerceService {
    runtime: KnowledgebaseRuntime,
}

impl HostedCommerceService {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
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
        let normalized_page_size = crate::pagination::normalize_api_page_size(page_size)?;
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
        Ok(KnowledgeMarketSubscriptionResult {
            accepted: true,
            status: "completed".to_string(),
        })
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
        Ok(KnowledgeMarketSubscriptionResult {
            accepted: true,
            status: "completed".to_string(),
        })
    }

    async fn create_site_deployment(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeSiteDeploymentRequest,
    ) -> ApiResult<KnowledgeSiteDeploymentResult> {
        ensure_runtime_tenant(&self.runtime, &context)?;
        let space = require_space_access_with_role(
            &self.runtime,
            &context,
            request.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
        let markdown_reader = RuntimeDocumentMarkdownReader::new(self.runtime.clone());
        let publisher = DriveStaticSitePublisher::from_env().map_err(|_| {
            ApiError::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "site_deployment_publisher_unavailable",
                "the static site publisher configuration is invalid",
            )
        })?;
        let service = KnowledgeSiteDeploymentService::new(
            self.runtime.document_store(),
            &markdown_reader,
            self.runtime.commerce_store(),
            self.runtime.drive_storage(),
            publisher
                .as_ref()
                .map(|publisher| publisher as &dyn KnowledgeSitePublisher),
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
        let space = require_space_access(&self.runtime, &context, record.space_id).await?;
        let markdown_reader = RuntimeDocumentMarkdownReader::new(self.runtime.clone());
        let service = KnowledgeSiteDeploymentService::new(
            self.runtime.document_store(),
            &markdown_reader,
            self.runtime.commerce_store(),
            self.runtime.drive_storage(),
            None,
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
        require_space_access_with_role(
            &self.runtime,
            &context,
            request.space_id,
            KnowledgeAccessRole::Writer,
        )
        .await?;
        match request.task_type {
            KnowledgeMediaTaskType::SpeechToText => {
                let source_url = require_https_media_url(request.source_url.as_deref())?;
                let client = resolve_media_client("media_transcription_provider_unavailable")?;
                let mut file = OpenAiFileReferenceInput::default();
                file.additional_properties
                    .insert("url".to_string(), serde_json::Value::String(source_url));
                let transcription = client
                    .audio()
                    .create_transcription(&OpenAiAudioTranscriptionRequest {
                        file,
                        language: None,
                        model: media_model(
                            "SDKWORK_KNOWLEDGEBASE_TRANSCRIPTION_MODEL",
                            "openai/whisper-1",
                        ),
                        prompt: normalize_optional_media_text(request.prompt.as_deref(), 4_000)?,
                        response_format: Some("json".to_string()),
                    })
                    .await
                    .map_err(|error| media_provider_error("media_transcription_failed", &error))?;
                if is_blank(Some(transcription.text.as_str())) {
                    return Err(ApiError::new(
                        axum::http::StatusCode::BAD_GATEWAY,
                        "media_transcription_invalid_response",
                        "transcription provider returned no text",
                    ));
                }
                Ok(KnowledgeMediaTaskResult {
                    accepted: true,
                    status: "completed".to_string(),
                    url: request.source_url,
                    resolution: None,
                    text: Some(transcription.text),
                    suggestions: Vec::new(),
                    similars: Vec::new(),
                })
            }
            KnowledgeMediaTaskType::GenerateImage => {
                let prompt = normalize_optional_media_text(request.prompt.as_deref(), 4_000)?
                    .ok_or_else(|| {
                        ApiError::invalid_request(
                            "invalid_media_task_request",
                            "prompt is required",
                        )
                    })?;
                let resolution = image_resolution(request.aspect_mode.as_deref())?;
                let quality = image_quality(request.style_mode.as_deref())?;
                let client = resolve_media_client("media_image_provider_unavailable")?;
                let result = client
                    .images()
                    .create_generation(&OpenAiImageGenerationRequest {
                        model: media_model(
                            "SDKWORK_KNOWLEDGEBASE_IMAGE_GENERATION_MODEL",
                            "openai/gpt-image-1",
                        ),
                        n: Some(1),
                        prompt,
                        quality,
                        response_format: Some("url".to_string()),
                        size: Some(resolution.to_string()),
                    })
                    .await
                    .map_err(|error| {
                        media_provider_error("media_image_generation_failed", &error)
                    })?;
                let image = result.data.into_iter().next().ok_or_else(|| {
                    ApiError::new(
                        axum::http::StatusCode::BAD_GATEWAY,
                        "media_image_invalid_response",
                        "image provider returned no output",
                    )
                })?;
                let image_url = require_https_provider_url(image.url.as_deref())?;
                Ok(KnowledgeMediaTaskResult {
                    accepted: true,
                    status: "completed".to_string(),
                    url: Some(image_url),
                    resolution: Some(resolution.to_string()),
                    text: None,
                    suggestions: image.revised_prompt.into_iter().collect(),
                    similars: Vec::new(),
                })
            }
        }
    }
}

struct DriveStaticSitePublisher {
    public_base_url: url::Url,
}

impl DriveStaticSitePublisher {
    fn from_env() -> Result<Option<Self>, String> {
        let Some(value) = std::env::var("SDKWORK_KNOWLEDGEBASE_SITE_PUBLIC_BASE_URL")
            .ok()
            .filter(|value| !is_blank(Some(value.as_str())))
        else {
            return Ok(None);
        };
        let public_base_url = url::Url::parse(value.trim())
            .map_err(|error| format!("invalid static site public base URL: {error}"))?;
        if public_base_url.scheme() != "https"
            || public_base_url.host_str().is_none()
            || !public_base_url.username().is_empty()
            || public_base_url.query().is_some()
            || public_base_url.fragment().is_some()
        {
            return Err(
                "static site public base URL must be absolute HTTPS without credentials, query, or fragment"
                    .to_string(),
            );
        }
        Ok(Some(Self { public_base_url }))
    }
}

#[async_trait]
impl KnowledgeSitePublisher for DriveStaticSitePublisher {
    async fn publish_site(
        &self,
        request: PublishKnowledgeSiteRequest,
    ) -> Result<PublishedKnowledgeSite, KnowledgeSitePublisherError> {
        if let Some(custom_domain) = request
            .custom_domain
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if !self
                .public_base_url
                .host_str()
                .is_some_and(|host| host.eq_ignore_ascii_case(custom_domain))
            {
                return Err(KnowledgeSitePublisherError::InvalidRequest(
                    "custom_domain must match the configured static site public host".to_string(),
                ));
            }
        }
        let mut public_url = self.public_base_url.clone();
        let mut segments = public_url.path_segments_mut().map_err(|_| {
            KnowledgeSitePublisherError::InvalidRequest(
                "static site public base URL cannot contain path segments".to_string(),
            )
        })?;
        segments.pop_if_empty();
        for segment in request.preview_object_key.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(KnowledgeSitePublisherError::InvalidRequest(
                    "preview object key contains an invalid path segment".to_string(),
                ));
            }
            segments.push(segment);
        }
        drop(segments);
        Ok(PublishedKnowledgeSite {
            public_url: public_url.to_string(),
        })
    }
}

fn resolve_media_client(
    unavailable_code: &'static str,
) -> ApiResult<clawrouter_open_sdk::SdkworkAiClient> {
    sdkwork_knowledgebase_agent_provider::resolve_claw_router_client_from_env().map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            unavailable_code,
            "the configured media provider is unavailable",
        )
    })
}

fn media_model(environment_key: &str, default_model: &str) -> String {
    std::env::var(environment_key)
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .unwrap_or_else(|| default_model.to_string())
}

fn normalize_optional_media_text(
    value: Option<&str>,
    max_chars: usize,
) -> ApiResult<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.chars().count() > max_chars {
        return Err(ApiError::invalid_request(
            "invalid_media_task_request",
            format!("media text exceeds {max_chars} characters"),
        ));
    }
    Ok(Some(value.to_string()))
}

fn require_https_media_url(value: Option<&str>) -> ApiResult<String> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 2_048)
        .ok_or_else(|| {
            ApiError::invalid_request(
                "invalid_media_task_request",
                "source_url is required and must not exceed 2048 bytes",
            )
        })?;
    let url = url::Url::parse(value).map_err(|_| {
        ApiError::invalid_request(
            "invalid_media_task_request",
            "source_url must be a valid URL",
        )
    })?;
    if url.scheme() != "https" || url.host_str().is_none() || !url.username().is_empty() {
        return Err(ApiError::invalid_request(
            "invalid_media_task_request",
            "source_url must be an absolute HTTPS URL without user information",
        ));
    }
    let blocked_host = match url.host() {
        Some(url::Host::Domain(domain)) => {
            domain.eq_ignore_ascii_case("localhost")
                || domain.to_ascii_lowercase().ends_with(".localhost")
        }
        Some(url::Host::Ipv4(address)) => {
            address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_broadcast()
                || address.is_documentation()
                || address.is_unspecified()
        }
        Some(url::Host::Ipv6(address)) => {
            address.is_loopback() || address.is_unspecified() || address.is_unique_local()
        }
        None => true,
    };
    if blocked_host {
        return Err(ApiError::invalid_request(
            "invalid_media_task_request",
            "source_url host is not allowed",
        ));
    }
    Ok(url.to_string())
}

fn require_https_provider_url(value: Option<&str>) -> ApiResult<String> {
    require_https_media_url(value).map_err(|_| {
        ApiError::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "media_image_invalid_response",
            "image provider returned an invalid URL",
        )
    })
}

fn image_resolution(aspect_mode: Option<&str>) -> ApiResult<&'static str> {
    match aspect_mode.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("square") | Some("1:1") => Ok("1024x1024"),
        Some("landscape") | Some("3:2") | Some("16:9") => Ok("1536x1024"),
        Some("portrait") | Some("2:3") | Some("9:16") => Ok("1024x1536"),
        Some(_) => Err(ApiError::invalid_request(
            "invalid_media_task_request",
            "aspect_mode must be square, landscape, portrait, 1:1, 3:2, 2:3, 16:9, or 9:16",
        )),
    }
}

fn image_quality(style_mode: Option<&str>) -> ApiResult<Option<String>> {
    match style_mode.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("auto") => Ok(Some("auto".to_string())),
        Some(value @ ("low" | "medium" | "high")) => Ok(Some(value.to_string())),
        Some(_) => Err(ApiError::invalid_request(
            "invalid_media_task_request",
            "style_mode must be auto, low, medium, or high",
        )),
    }
}

fn media_provider_error(code: &'static str, error: &clawrouter_open_sdk::SdkworkError) -> ApiError {
    tracing::warn!(error = %error, "Claw Router media request failed");
    ApiError::new(
        axum::http::StatusCode::BAD_GATEWAY,
        code,
        "the media provider request failed",
    )
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
        KnowledgeSiteDeploymentServiceError::PublisherUnavailable => ApiError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "site_deployment_publisher_unavailable",
            "no site publisher is configured for this deployment",
        ),
        KnowledgeSiteDeploymentServiceError::Publisher(error) => ApiError::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "site_deployment_publisher_failed",
            error.to_string(),
        ),
    }
}
