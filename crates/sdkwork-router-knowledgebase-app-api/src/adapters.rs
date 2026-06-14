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
use std::sync::Arc;

use crate::{
    ApiResult, KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext,
    KnowledgeBrowserApi, KnowledgeDocumentAppService, KnowledgeDriveImportAppService,
    KnowledgeIngestAppService, KnowledgeRetrievalAppService, KnowledgeSpaceAppService,
    KnowledgeWikiAppService,
};

pub struct BrowserOnlyAppApi {
    browser: Arc<dyn KnowledgeBrowserApi>,
}

impl BrowserOnlyAppApi {
    pub fn new(browser: Arc<dyn KnowledgeBrowserApi>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl KnowledgeAppApi for BrowserOnlyAppApi {
    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser.list_browser(request).await
    }
}

pub struct RetrievalOnlyAppApi {
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
}

impl RetrievalOnlyAppApi {
    pub fn new(retrieval: Arc<dyn KnowledgeRetrievalAppService>) -> Self {
        Self { retrieval }
    }
}

#[async_trait]
impl KnowledgeAppApi for RetrievalOnlyAppApi {
    async fn create_retrieval(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval.retrieve(request).await
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval
            .retrieve_retrieval(context, retrieval_id)
            .await
    }

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        self.retrieval.create_context_pack(request).await
    }
}

pub struct AgentOnlyAppApi {
    agent: Arc<dyn KnowledgeAgentAppService>,
}

impl AgentOnlyAppApi {
    pub fn new(agent: Arc<dyn KnowledgeAgentAppService>) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl KnowledgeAppApi for AgentOnlyAppApi {
    async fn create_agent_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.create_profile(request).await
    }

    async fn retrieve_agent_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.retrieve_profile(profile_id).await
    }

    async fn update_agent_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.update_profile(profile_id, request).await
    }

    async fn delete_agent_profile(&self, profile_id: u64) -> ApiResult<()> {
        self.agent.delete_profile(profile_id).await
    }

    async fn list_agent_profile_bindings(
        &self,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        self.agent.list_bindings(profile_id).await
    }

    async fn create_agent_profile_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent.create_binding(profile_id, request).await
    }

    async fn update_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .update_binding(profile_id, binding_id, request)
            .await
    }

    async fn delete_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.agent.delete_binding(profile_id, binding_id).await
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.agent.preview_retrieval(profile_id, request).await
    }
}

pub struct AgentAndRetrievalAppApi {
    agent: Arc<dyn KnowledgeAgentAppService>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
}

impl AgentAndRetrievalAppApi {
    pub fn new(
        agent: Arc<dyn KnowledgeAgentAppService>,
        retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    ) -> Self {
        Self { agent, retrieval }
    }
}

#[async_trait]
impl KnowledgeAppApi for AgentAndRetrievalAppApi {
    async fn create_retrieval(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval.retrieve(request).await
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval
            .retrieve_retrieval(context, retrieval_id)
            .await
    }

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        self.retrieval.create_context_pack(request).await
    }

    async fn create_agent_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.create_profile(request).await
    }

    async fn retrieve_agent_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.retrieve_profile(profile_id).await
    }

    async fn update_agent_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.update_profile(profile_id, request).await
    }

    async fn delete_agent_profile(&self, profile_id: u64) -> ApiResult<()> {
        self.agent.delete_profile(profile_id).await
    }

    async fn list_agent_profile_bindings(
        &self,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        self.agent.list_bindings(profile_id).await
    }

    async fn create_agent_profile_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent.create_binding(profile_id, request).await
    }

    async fn update_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .update_binding(profile_id, binding_id, request)
            .await
    }

    async fn delete_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.agent.delete_binding(profile_id, binding_id).await
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.agent.preview_retrieval(profile_id, request).await
    }
}

pub struct FullAppApi {
    space: Arc<dyn KnowledgeSpaceAppService>,
    drive_import: Arc<dyn KnowledgeDriveImportAppService>,
    ingest: Arc<dyn KnowledgeIngestAppService>,
    document: Arc<dyn KnowledgeDocumentAppService>,
    wiki: Arc<dyn KnowledgeWikiAppService>,
    browser: Arc<dyn KnowledgeBrowserApi>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    agent: Arc<dyn KnowledgeAgentAppService>,
}

impl FullAppApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        space: Arc<dyn KnowledgeSpaceAppService>,
        drive_import: Arc<dyn KnowledgeDriveImportAppService>,
        ingest: Arc<dyn KnowledgeIngestAppService>,
        document: Arc<dyn KnowledgeDocumentAppService>,
        wiki: Arc<dyn KnowledgeWikiAppService>,
        browser: Arc<dyn KnowledgeBrowserApi>,
        retrieval: Arc<dyn KnowledgeRetrievalAppService>,
        agent: Arc<dyn KnowledgeAgentAppService>,
    ) -> Self {
        Self {
            space,
            drive_import,
            ingest,
            document,
            wiki,
            browser,
            retrieval,
            agent,
        }
    }
}

#[async_trait]
impl KnowledgeAppApi for FullAppApi {
    async fn create_space(
        &self,
        request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        self.space.create_space(request).await
    }

    async fn retrieve_space(&self, space_id: u64) -> ApiResult<KnowledgeSpace> {
        self.space.retrieve_space(space_id).await
    }

    async fn create_drive_import(
        &self,
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        self.drive_import.import_drive_object(request).await
    }

    async fn create_ingest(&self, request: KnowledgeIngestRequest) -> ApiResult<IngestionJob> {
        self.ingest.create_ingest(request).await
    }

    async fn retrieve_ingest(&self, ingest_id: u64) -> ApiResult<IngestionJob> {
        self.ingest.retrieve_ingest(ingest_id).await
    }

    async fn list_documents(&self) -> ApiResult<KnowledgeDocumentList> {
        self.document.list_documents().await
    }

    async fn create_document(
        &self,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        self.document.create_document(request).await
    }

    async fn retrieve_document(&self, document_id: u64) -> ApiResult<KnowledgeDocument> {
        self.document.retrieve_document(document_id).await
    }

    async fn update_document(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        self.document.update_document(document_id, request).await
    }

    async fn delete_document(&self, document_id: u64) -> ApiResult<()> {
        self.document.delete_document(document_id).await
    }

    async fn list_document_versions(
        &self,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        self.document.list_document_versions(document_id).await
    }

    async fn create_document_version(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        self.document
            .create_document_version(document_id, request)
            .await
    }

    async fn list_wiki_pages(&self) -> ApiResult<WikiPageSummaryList> {
        self.wiki.list_wiki_pages().await
    }

    async fn retrieve_wiki_page(&self, page_id: u64) -> ApiResult<WikiPageSummary> {
        self.wiki.retrieve_wiki_page(page_id).await
    }

    async fn list_wiki_page_revisions(
        &self,
        page_id: u64,
    ) -> ApiResult<KnowledgeWikiPageRevisionList> {
        self.wiki.list_wiki_page_revisions(page_id).await
    }

    async fn retrieve_wiki_index(&self) -> ApiResult<WikiIndexDocument> {
        self.wiki.retrieve_wiki_index().await
    }

    async fn retrieve_wiki_log(&self) -> ApiResult<WikiLogDocument> {
        self.wiki.retrieve_wiki_log().await
    }

    async fn retrieve_wiki_schema(&self) -> ApiResult<WikiSchemaDocument> {
        self.wiki.retrieve_wiki_schema().await
    }

    async fn create_wiki_query(&self, request: WikiQueryRequest) -> ApiResult<WikiQueryResult> {
        self.wiki.create_wiki_query(request).await
    }

    async fn file_wiki_query_answer(
        &self,
        query_id: u64,
        request: WikiFileAnswerRequest,
    ) -> ApiResult<WikiQueryResult> {
        self.wiki.file_wiki_query_answer(query_id, request).await
    }

    async fn create_wiki_context_pack(
        &self,
        request: WikiContextPackRequest,
    ) -> ApiResult<KnowledgeWikiFileEntry> {
        self.wiki.create_wiki_context_pack(request).await
    }

    async fn list_browser(
        &self,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser.list_browser(request).await
    }

    async fn create_retrieval(
        &self,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval.retrieve(request).await
    }

    async fn retrieve_retrieval(
        &self,
        context: KnowledgeAppRequestContext,
        retrieval_id: u64,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.retrieval
            .retrieve_retrieval(context, retrieval_id)
            .await
    }

    async fn create_context_pack(
        &self,
        request: KnowledgeContextPackRequest,
    ) -> ApiResult<KnowledgeContextPack> {
        self.retrieval.create_context_pack(request).await
    }

    async fn create_agent_profile(
        &self,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.create_profile(request).await
    }

    async fn retrieve_agent_profile(&self, profile_id: u64) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.retrieve_profile(profile_id).await
    }

    async fn update_agent_profile(
        &self,
        profile_id: u64,
        request: KnowledgeAgentProfileRequest,
    ) -> ApiResult<KnowledgeAgentProfile> {
        self.agent.update_profile(profile_id, request).await
    }

    async fn delete_agent_profile(&self, profile_id: u64) -> ApiResult<()> {
        self.agent.delete_profile(profile_id).await
    }

    async fn list_agent_profile_bindings(
        &self,
        profile_id: u64,
    ) -> ApiResult<KnowledgeAgentBindingList> {
        self.agent.list_bindings(profile_id).await
    }

    async fn create_agent_profile_binding(
        &self,
        profile_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent.create_binding(profile_id, request).await
    }

    async fn update_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
        request: KnowledgeAgentBindingRequest,
    ) -> ApiResult<KnowledgeAgentBinding> {
        self.agent
            .update_binding(profile_id, binding_id, request)
            .await
    }

    async fn delete_agent_profile_binding(
        &self,
        profile_id: u64,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.agent.delete_binding(profile_id, binding_id).await
    }

    async fn create_agent_profile_retrieval_preview(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalRequest,
    ) -> ApiResult<KnowledgeRetrievalResult> {
        self.agent.preview_retrieval(profile_id, request).await
    }
}
