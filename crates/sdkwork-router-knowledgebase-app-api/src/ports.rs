use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, IngestionJob, KnowledgeAgentBinding, KnowledgeAgentBindingList,
    KnowledgeAgentBindingRequest, KnowledgeAgentProfile, KnowledgeAgentProfileRequest,
    KnowledgeBrowserPage, KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeDocument,
    KnowledgeDocumentList, KnowledgeDocumentVersion, KnowledgeDocumentVersionList,
    KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeIngestRequest,
    KnowledgeRetrievalRequest, KnowledgeRetrievalResult, KnowledgeSpace, KnowledgeWikiFileEntry,
    KnowledgeWikiPageRevisionList, ListKnowledgeBrowserRequest, WikiContextPackRequest,
    WikiFileAnswerRequest, WikiIndexDocument, WikiLogDocument, WikiPageSummary,
    WikiPageSummaryList, WikiQueryRequest, WikiQueryResult, WikiSchemaDocument,
};

use crate::{ApiError, ApiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeAppRequestContext {
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
}

#[async_trait]
pub trait KnowledgeBrowserApi: Send + Sync + 'static {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage>;
}

#[async_trait]
pub trait KnowledgeRetrievalAppService: Send + Sync + 'static {
    async fn retrieve(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult>;

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult>;

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack>;
}

#[async_trait]
pub trait KnowledgeAgentAppService: Send + Sync + 'static {
    async fn create_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile>;

    async fn retrieve_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile>;

    async fn update_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile>;

    async fn delete_profile(&self, profile_id: u64) -> ApiResult<()>;

    async fn list_bindings(&self, profile_id: u64) -> ApiResult<KnowledgeAgentBindingList>;

    async fn create_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding>;

    async fn update_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding>;

    async fn delete_binding(&self, profile_id: u64, binding_id: u64) -> ApiResult<()>;

    async fn preview_retrieval(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult>;
}

#[async_trait]
pub trait KnowledgeAppApi: Send + Sync + 'static {
    async fn create_space(
        &self,
        _request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        Err(ApiError::not_implemented("spaces.create"))
    }

    async fn retrieve_space(&self, _space_id: u64) -> ApiResult<KnowledgeSpace> {
        Err(ApiError::not_implemented("spaces.retrieve"))
    }

    async fn create_drive_import(
        &self,
        _request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        Err(ApiError::not_implemented("driveImports.create"))
    }

    async fn create_ingest(&self, _request: KnowledgeIngestRequest) -> ApiResult<IngestionJob> {
        Err(ApiError::not_implemented("ingests.create"))
    }

    async fn retrieve_ingest(&self, _ingest_id: u64) -> ApiResult<IngestionJob> {
        Err(ApiError::not_implemented("ingests.retrieve"))
    }

    async fn list_documents(&self) -> ApiResult<KnowledgeDocumentList> {
        Err(ApiError::not_implemented("documents.list"))
    }

    async fn create_document(
        &self,
        _request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.create"))
    }

    async fn retrieve_document(&self, _document_id: u64) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.retrieve"))
    }

    async fn update_document(
        &self,
        _document_id: u64,
        _request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        Err(ApiError::not_implemented("documents.update"))
    }

    async fn delete_document(&self, _document_id: u64) -> ApiResult<()> {
        Err(ApiError::not_implemented("documents.delete"))
    }

    async fn list_document_versions(
        &self,
        _document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        Err(ApiError::not_implemented("documents.versions.list"))
    }

    async fn create_document_version(
        &self,
        _document_id: u64,
        _request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        Err(ApiError::not_implemented("documents.versions.create"))
    }

    async fn list_wiki_pages(&self) -> ApiResult<WikiPageSummaryList> {
        Err(ApiError::not_implemented("wiki.pages.list"))
    }

    async fn retrieve_wiki_page(&self, _page_id: u64) -> ApiResult<WikiPageSummary> {
        Err(ApiError::not_implemented("wiki.pages.retrieve"))
    }

    async fn list_wiki_page_revisions(
        &self,
        _page_id: u64,
    ) -> ApiResult<KnowledgeWikiPageRevisionList> {
        Err(ApiError::not_implemented("wiki.pages.revisions.list"))
    }

    async fn retrieve_wiki_index(&self) -> ApiResult<WikiIndexDocument> {
        Err(ApiError::not_implemented("wiki.index.retrieve"))
    }

    async fn retrieve_wiki_log(&self) -> ApiResult<WikiLogDocument> {
        Err(ApiError::not_implemented("wiki.log.retrieve"))
    }

    async fn retrieve_wiki_schema(&self) -> ApiResult<WikiSchemaDocument> {
        Err(ApiError::not_implemented("wiki.schema.retrieve"))
    }

    async fn create_wiki_query(&self, _request: WikiQueryRequest) -> ApiResult<WikiQueryResult> {
        Err(ApiError::not_implemented("wiki.queries.create"))
    }

    async fn file_wiki_query_answer(
        &self,
        _query_id: u64,
        _request: WikiFileAnswerRequest,
    ) -> ApiResult<WikiQueryResult> {
        Err(ApiError::not_implemented("wiki.queries.fileAnswer"))
    }

    async fn create_wiki_context_pack(
        &self,
        _request: WikiContextPackRequest,
    ) -> ApiResult<KnowledgeWikiFileEntry> {
        Err(ApiError::not_implemented("wiki.contextPacks.create"))
    }

    async fn list_browser(
        &self,
        _request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        Err(ApiError::not_implemented("spaces.browser.list"))
    }

    async fn create_retrieval(
        &self,
        _request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Err(ApiError::not_implemented("retrievals.create"))
    }

    async fn retrieve_retrieval(
        &self,
        _context: KnowledgeAppRequestContext,
        _retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Err(ApiError::not_implemented("retrievals.retrieve"))
    }

    async fn create_context_pack(
        &self,
        _request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        Err(ApiError::not_implemented("contextPacks.create"))
    }

    async fn create_agent_profile(
        &self,
        _request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        Err(ApiError::not_implemented("agentProfiles.create"))
    }

    async fn retrieve_agent_profile(&self, _profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        Err(ApiError::not_implemented("agentProfiles.retrieve"))
    }

    async fn update_agent_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        Err(ApiError::not_implemented("agentProfiles.update"))
    }

    async fn delete_agent_profile(&self, _profile_id: u64) -> ApiResult<()> {
        Err(ApiError::not_implemented("agentProfiles.delete"))
    }

    async fn list_agent_profile_bindings(
        &self,
        _profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        Err(ApiError::not_implemented("agentProfiles.bindings.list"))
    }

    async fn create_agent_profile_binding(
        &self,
        _profile_id: u64,
        _request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        Err(ApiError::not_implemented("agentProfiles.bindings.create"))
    }

    async fn update_agent_profile_binding(
        &self,
        _profile_id: u64,
        _binding_id: u64,
        _request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        Err(ApiError::not_implemented("agentProfiles.bindings.update"))
    }

    async fn delete_agent_profile_binding(
        &self,
        _profile_id: u64,
        _binding_id: u64,
    ) -> ApiResult<()> {
        Err(ApiError::not_implemented("agentProfiles.bindings.delete"))
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        _profile_id: u64,
        _request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        Err(ApiError::not_implemented(
            "agentProfiles.retrievalPreview.create",
        ))
    }
}
