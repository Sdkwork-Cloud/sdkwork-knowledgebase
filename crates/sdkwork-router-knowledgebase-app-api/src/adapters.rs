use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    context_binding::{
        CreateKnowledgeSpaceContextBindingRequest, KnowledgeSpaceContextBinding,
        KnowledgeSpaceContextBindingList, UpdateKnowledgeSpaceContextBindingRequest,
    },
    CompleteKnowledgeUploadSessionRequest, CreateKnowledgeDocumentRequest,
    CreateKnowledgeDocumentVersionRequest, CreateKnowledgeSpaceRequest,
    CreateKnowledgeUploadSessionRequest, IngestionJob, KnowledgeAgentBinding,
    KnowledgeAgentBindingList, KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest,
    KnowledgeAgentChatResponse, KnowledgeAgentProfile, KnowledgeAgentProfileRequest,
    KnowledgeBrowserPage, KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeDocument,
    KnowledgeDocumentList, KnowledgeDocumentVersion, KnowledgeDocumentVersionList,
    KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeIngestRequest,
    KnowledgeOkfBundleFile, KnowledgeOkfConceptRevisionList, KnowledgeRetrievalRequest,
    KnowledgeRetrievalResult, KnowledgeSpace, KnowledgeUploadSession, ListKnowledgeBrowserRequest,
    OkfConceptSummary, OkfConceptSummaryList, OkfContextPackRequest, OkfFileAnswerRequest,
    OkfIndexDocument, OkfLogDocument, OkfProfileDocument, OkfQueryRequest, OkfQueryResult,
};
use std::sync::Arc;

use crate::{
    ApiResult, KnowledgeAgentAppService, KnowledgeAppApi, KnowledgeAppRequestContext,
    KnowledgeBrowserApi, KnowledgeContextBindingAppService, KnowledgeDocumentAppService,
    KnowledgeDriveImportAppService, KnowledgeIngestAppService, KnowledgeOkfAppService,
    KnowledgeRetrievalAppService, KnowledgeSpaceAppService, KnowledgeUploadSessionAppService,
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
        context: KnowledgeAppRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser.list_browser(context, request).await
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

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        self.agent.create_agent_chat(profile_id, request).await
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

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        self.agent.create_agent_chat(profile_id, request).await
    }
}

pub struct FullAppApi {
    space: Arc<dyn KnowledgeSpaceAppService>,
    drive_import: Arc<dyn KnowledgeDriveImportAppService>,
    ingest: Arc<dyn KnowledgeIngestAppService>,
    document: Arc<dyn KnowledgeDocumentAppService>,
    okf: Arc<dyn KnowledgeOkfAppService>,
    browser: Arc<dyn KnowledgeBrowserApi>,
    retrieval: Arc<dyn KnowledgeRetrievalAppService>,
    agent: Arc<dyn KnowledgeAgentAppService>,
    context_binding: Arc<dyn KnowledgeContextBindingAppService>,
    upload_session: Arc<dyn KnowledgeUploadSessionAppService>,
}

impl FullAppApi {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        space: Arc<dyn KnowledgeSpaceAppService>,
        drive_import: Arc<dyn KnowledgeDriveImportAppService>,
        ingest: Arc<dyn KnowledgeIngestAppService>,
        document: Arc<dyn KnowledgeDocumentAppService>,
        okf: Arc<dyn KnowledgeOkfAppService>,
        browser: Arc<dyn KnowledgeBrowserApi>,
        retrieval: Arc<dyn KnowledgeRetrievalAppService>,
        agent: Arc<dyn KnowledgeAgentAppService>,
        context_binding: Arc<dyn KnowledgeContextBindingAppService>,
        upload_session: Arc<dyn KnowledgeUploadSessionAppService>,
    ) -> Self {
        Self {
            space,
            drive_import,
            ingest,
            document,
            okf,
            browser,
            retrieval,
            agent,
            context_binding,
            upload_session,
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

    async fn list_okf_concepts(&self) -> ApiResult<OkfConceptSummaryList> {
        self.okf.list_okf_concepts().await
    }

    async fn retrieve_okf_concept(&self, concept_row_id: u64) -> ApiResult<OkfConceptSummary> {
        self.okf.retrieve_okf_concept(concept_row_id).await
    }

    async fn list_okf_concept_revisions(
        &self,
        concept_row_id: u64,
    ) -> ApiResult<KnowledgeOkfConceptRevisionList> {
        self.okf.list_okf_concept_revisions(concept_row_id).await
    }

    async fn retrieve_okf_index(&self) -> ApiResult<OkfIndexDocument> {
        self.okf.retrieve_okf_index().await
    }

    async fn retrieve_okf_log(&self) -> ApiResult<OkfLogDocument> {
        self.okf.retrieve_okf_log().await
    }

    async fn retrieve_okf_schema(&self) -> ApiResult<OkfProfileDocument> {
        self.okf.retrieve_okf_schema().await
    }

    async fn create_okf_query(&self, request: OkfQueryRequest) -> ApiResult<OkfQueryResult> {
        self.okf.create_okf_query(request).await
    }

    async fn file_okf_query_answer(
        &self,
        query_id: u64,
        request: OkfFileAnswerRequest,
    ) -> ApiResult<OkfQueryResult> {
        self.okf.file_okf_query_answer(query_id, request).await
    }

    async fn create_okf_context_pack(
        &self,
        request: OkfContextPackRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        self.okf.create_okf_context_pack(request).await
    }

    async fn list_browser(
        &self,
        context: KnowledgeAppRequestContext,
        request: ListKnowledgeBrowserRequest,
    ) -> ApiResult<KnowledgeBrowserPage> {
        self.browser.list_browser(context, request).await
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

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        self.agent.create_agent_chat(profile_id, request).await
    }

    async fn list_space_context_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBindingList> {
        self.context_binding
            .list_space_context_bindings(context, space_id)
            .await
    }

    async fn create_space_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        self.context_binding
            .create_space_context_binding(context, space_id, request)
            .await
    }

    async fn retrieve_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        self.context_binding
            .retrieve_context_binding(context, binding_id)
            .await
    }

    async fn update_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        self.context_binding
            .update_context_binding(context, binding_id, request)
            .await
    }

    async fn delete_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<()> {
        self.context_binding
            .delete_context_binding(context, binding_id)
            .await
    }

    async fn create_upload_session(
        &self,
        request: CreateKnowledgeUploadSessionRequest,
    ) -> ApiResult<KnowledgeUploadSession> {
        self.upload_session.create_upload_session(request).await
    }

    async fn complete_upload_session(
        &self,
        session_id: u64,
        request: CompleteKnowledgeUploadSessionRequest,
    ) -> ApiResult<IngestionJob> {
        self.upload_session
            .complete_upload_session(session_id, request)
            .await
    }
}
