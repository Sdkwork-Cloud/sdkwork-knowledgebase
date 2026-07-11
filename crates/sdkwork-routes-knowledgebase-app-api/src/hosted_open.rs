use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    IngestionJob, KnowledgeBrowserListData, KnowledgeContextPack, KnowledgeContextPackRequest,
    KnowledgeDocument, KnowledgeIngestRequest, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
    ListKnowledgeBrowserRequest,
};
use sdkwork_routes_knowledgebase_open_api::{
    ApiError as OpenApiError, ApiResult as OpenApiResult, KnowledgeOpenApi,
    KnowledgeOpenApiRequestContext,
};

use crate::{
    hosted::{HostedBrowserService, HostedDocumentService, HostedIngestService},
    runtime::{HostedRetrievalService, KnowledgebaseRuntime},
    ApiError, KnowledgeAppRequestContext, KnowledgeBrowserApi, KnowledgeDocumentAppService,
    KnowledgeIngestAppService, KnowledgeRetrievalAppService,
};

#[derive(Clone)]
pub(crate) struct HostedOpenApi {
    runtime: KnowledgebaseRuntime,
    retrieval: HostedRetrievalService,
    ingest: HostedIngestService,
    document: HostedDocumentService,
    browser: HostedBrowserService,
}

impl HostedOpenApi {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self {
            retrieval: HostedRetrievalService::new(runtime.clone()),
            ingest: HostedIngestService::new(runtime.clone()),
            document: HostedDocumentService::new(runtime.clone()),
            browser: HostedBrowserService::new(runtime.clone()),
            runtime,
        }
    }

    fn ensure_tenant(&self, context: &KnowledgeOpenApiRequestContext) -> Result<(), OpenApiError> {
        if context.tenant_id != self.runtime.tenant_id() {
            return Err(OpenApiError::new(
                axum::http::StatusCode::FORBIDDEN,
                "tenant_id_mismatch",
                "authenticated tenant does not match configured runtime tenant",
            ));
        }
        Ok(())
    }

    fn app_context(context: &KnowledgeOpenApiRequestContext) -> KnowledgeAppRequestContext {
        KnowledgeAppRequestContext {
            tenant_id: context.tenant_id,
            actor_id: context.actor_id,
            organization_id: context.organization_id,
            session_id: None,
        }
    }

    fn map_error<T>(result: Result<T, ApiError>) -> OpenApiResult<T> {
        result.map_err(|error| error.to_open_api_error())
    }
}

#[async_trait]
impl KnowledgeOpenApi for HostedOpenApi {
    async fn create_retrieval(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: KnowledgeRetrievalRequest,
    ) -> OpenApiResult<KnowledgeRetrievalResult> {
        self.ensure_tenant(&context)?;
        let app_context = Self::app_context(&context);
        Self::map_error(
            self.retrieval
                .retrieve(
                    app_context,
                    request
                        .with_tenant_id(context.tenant_id)
                        .with_actor_id(context.actor_id),
                )
                .await,
        )
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeOpenApiRequestContext,
        retrieval_id: u64,
    ) -> OpenApiResult<KnowledgeRetrievalResult> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.retrieval
                .retrieve_retrieval(Self::app_context(&context), retrieval_id)
                .await,
        )
    }

    async fn create_context_pack(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: KnowledgeContextPackRequest,
    ) -> OpenApiResult<KnowledgeContextPack> {
        self.ensure_tenant(&context)?;
        let app_context = Self::app_context(&context);
        Self::map_error(
            self.retrieval
                .create_context_pack(
                    app_context,
                    request
                        .with_tenant_id(context.tenant_id)
                        .with_actor_id(context.actor_id),
                )
                .await,
        )
    }

    async fn create_ingest(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: KnowledgeIngestRequest,
    ) -> OpenApiResult<IngestionJob> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.ingest
                .create_ingest(Self::app_context(&context), request)
                .await,
        )
    }

    async fn retrieve_ingest(
        &self,
        context: KnowledgeOpenApiRequestContext,
        ingest_id: u64,
    ) -> OpenApiResult<IngestionJob> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.ingest
                .retrieve_ingest(Self::app_context(&context), ingest_id)
                .await,
        )
    }

    async fn list_documents(
        &self,
        context: KnowledgeOpenApiRequestContext,
        space_id: u64,
        cursor: Option<String>,
        page_size: Option<u32>,
    ) -> OpenApiResult<sdkwork_utils_rust::SdkWorkPageData<KnowledgeDocument>> {
        self.ensure_tenant(&context)?;
        self.document
            .list_documents(Self::app_context(&context), space_id, cursor, page_size)
            .await
            .map_err(|error| error.to_open_api_error())
    }

    async fn retrieve_document(
        &self,
        context: KnowledgeOpenApiRequestContext,
        document_id: u64,
    ) -> OpenApiResult<KnowledgeDocument> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.document
                .retrieve_document(Self::app_context(&context), document_id)
                .await,
        )
    }

    async fn list_browser(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> OpenApiResult<KnowledgeBrowserListData> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.browser
                .list_browser(Self::app_context(&context), request)
                .await,
        )
    }
}
