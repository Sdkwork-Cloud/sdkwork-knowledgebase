use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    IngestionJob, KnowledgeBrowserPage, KnowledgeContextPack, KnowledgeContextPackRequest,
    KnowledgeDocument, KnowledgeDocumentList, KnowledgeIngestRequest, KnowledgeRetrievalRequest,
    KnowledgeRetrievalResult, ListKnowledgeBrowserRequest,
};
use sdkwork_router_knowledgebase_open_api::{
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
            organization_id: None,
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
        Self::map_error(self.retrieval.retrieve(request).await)
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
        Self::map_error(self.retrieval.create_context_pack(request).await)
    }

    async fn create_ingest(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: KnowledgeIngestRequest,
    ) -> OpenApiResult<IngestionJob> {
        self.ensure_tenant(&context)?;
        Self::map_error(self.ingest.create_ingest(request).await)
    }

    async fn retrieve_ingest(
        &self,
        context: KnowledgeOpenApiRequestContext,
        ingest_id: u64,
    ) -> OpenApiResult<IngestionJob> {
        self.ensure_tenant(&context)?;
        Self::map_error(self.ingest.retrieve_ingest(ingest_id).await)
    }

    async fn list_documents(
        &self,
        context: KnowledgeOpenApiRequestContext,
    ) -> OpenApiResult<KnowledgeDocumentList> {
        self.ensure_tenant(&context)?;
        Self::map_error(self.document.list_documents().await)
    }

    async fn retrieve_document(
        &self,
        context: KnowledgeOpenApiRequestContext,
        document_id: u64,
    ) -> OpenApiResult<KnowledgeDocument> {
        self.ensure_tenant(&context)?;
        Self::map_error(self.document.retrieve_document(document_id).await)
    }

    async fn list_browser(
        &self,
        context: KnowledgeOpenApiRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> OpenApiResult<KnowledgeBrowserPage> {
        self.ensure_tenant(&context)?;
        Self::map_error(
            self.browser
                .list_browser(Self::app_context(&context), request)
                .await,
        )
    }
}
