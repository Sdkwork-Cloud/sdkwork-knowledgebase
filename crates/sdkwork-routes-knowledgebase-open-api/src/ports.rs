use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    IngestionJob, KnowledgeBrowserListData, KnowledgeContextPack, KnowledgeContextPackRequest,
    KnowledgeDocument, KnowledgeIngestRequest, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
    ListKnowledgeBrowserRequest,
};
use sdkwork_utils_rust::SdkWorkPageData;

use crate::{ApiError, ApiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOpenApiRequestContext {
    pub api_key_id: String,
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
    pub organization_id: Option<u64>,
}

#[async_trait]
pub trait KnowledgeOpenApi: Send + Sync + 'static {
    async fn create_retrieval(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Err(ApiError::unsupported_operation("retrievals.create"))
    }

    async fn retrieve_retrieval(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Err(ApiError::unsupported_operation("retrievals.retrieve"))
    }

    async fn create_context_pack(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        Err(ApiError::unsupported_operation("contextPacks.create"))
    }

    async fn create_ingest(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _request: KnowledgeIngestRequest,
    ) -> ApiResult<IngestionJob> {
        Err(ApiError::unsupported_operation("ingests.create"))
    }

    async fn retrieve_ingest(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _ingest_id: u64,
    ) -> ApiResult<IngestionJob> {
        Err(ApiError::unsupported_operation("ingests.retrieve"))
    }

    async fn list_documents(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _space_id: u64,
        _cursor: Option<String>,
        _page_size: Option<u32>,
    ) -> ApiResult<SdkWorkPageData<KnowledgeDocument>> {
        Err(ApiError::unsupported_operation("documents.list"))
    }

    async fn retrieve_document(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _document_id: u64,
    ) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::unsupported_operation("documents.retrieve"))
    }

    async fn list_browser(
        &self,
        _context: KnowledgeOpenApiRequestContext,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserListData> {
        Err(ApiError::unsupported_operation("spaces.browser.list"))
    }
}
