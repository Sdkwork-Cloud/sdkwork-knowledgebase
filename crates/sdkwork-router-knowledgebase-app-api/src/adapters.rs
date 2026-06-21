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
    OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult, OkfConceptSummary,
    OkfConceptSummaryList, OkfConceptUpsertRequest, OkfContextPackRequest, OkfFileAnswerRequest,
    OkfIndexDocument, OkfLogDocument, OkfProfileDocument, OkfQualityRun, OkfQualityRunRequest,
    OkfQueryRequest, OkfQueryResult, GrantKnowledgeSpaceMemberRequest, KnowledgeSpaceMemberList,
    KnowledgeSpaceMemberSubjectType, UpdateKnowledgeSpaceRequest,
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
        context: KnowledgeAppRequestContext,
        request: CreateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        self.space.create_space(context, request).await
    }

    async fn retrieve_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpace> {
        self.space.retrieve_space(context, space_id).await
    }

    async fn update_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: UpdateKnowledgeSpaceRequest,
    ) -> ApiResult<KnowledgeSpace> {
        self.space.update_space(context, space_id, request).await
    }

    async fn delete_space(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<()> {
        self.space.delete_space(context, space_id).await
    }

    async fn list_space_members(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpaceMemberList> {
        self.space.list_space_members(context, space_id).await
    }

    async fn grant_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: GrantKnowledgeSpaceMemberRequest,
    ) -> ApiResult<()> {
        self.space.grant_space_member(context, space_id, request).await
    }

    async fn revoke_space_member(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        subject_type: KnowledgeSpaceMemberSubjectType,
        subject_id: String,
    ) -> ApiResult<()> {
        self.space
            .revoke_space_member(context, space_id, subject_type, subject_id)
            .await
    }

    async fn create_drive_import(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult> {
        self.drive_import.import_drive_object(context, request).await
    }

    async fn create_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        request: KnowledgeIngestRequest,
    ) -> ApiResult<IngestionJob> {
        self.ingest.create_ingest(context, request).await
    }

    async fn retrieve_ingest(
        &self,
        context: KnowledgeAppRequestContext,
        ingest_id: u64,
    ) -> ApiResult<IngestionJob> {
        self.ingest.retrieve_ingest(context, ingest_id).await
    }

    async fn list_documents(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeDocumentList> {
        self.document.list_documents(context, space_id).await
    }

    async fn create_document(
        &self,
        context: KnowledgeAppRequestContext,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        self.document.create_document(context, request).await
    }

    async fn retrieve_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocument> {
        self.document.retrieve_document(context, document_id).await
    }

    async fn update_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument> {
        self.document
            .update_document(context, document_id, request)
            .await
    }

    async fn delete_document(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<()> {
        self.document.delete_document(context, document_id).await
    }

    async fn list_document_versions(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList> {
        self.document
            .list_document_versions(context, document_id)
            .await
    }

    async fn create_document_version(
        &self,
        context: KnowledgeAppRequestContext,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion> {
        self.document
            .create_document_version(context, document_id, request)
            .await
    }

    async fn list_okf_concepts(&self, space_id: u64) -> ApiResult<OkfConceptSummaryList> {
        self.okf.list_okf_concepts(space_id).await
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

    async fn upsert_okf_concept(
        &self,
        request: OkfConceptUpsertRequest,
    ) -> ApiResult<OkfConceptSummary> {
        self.okf.upsert_okf_concept(request).await
    }

    async fn delete_okf_concept(
        &self,
        context: KnowledgeAppRequestContext,
        concept_row_id: u64,
    ) -> ApiResult<()> {
        self.okf.delete_okf_concept(context, concept_row_id).await
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

    async fn create_okf_export(
        &self,
        request: OkfBundleExportRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        self.okf.create_okf_export(request).await
    }

    async fn retrieve_okf_export(&self, export_id: u64) -> ApiResult<KnowledgeOkfBundleFile> {
        self.okf.retrieve_okf_export(export_id).await
    }

    async fn create_okf_import(
        &self,
        request: OkfBundleImportRequest,
    ) -> ApiResult<OkfBundleImportResult> {
        self.okf.create_okf_import(request).await
    }

    async fn create_okf_lint_run(&self, request: OkfQualityRunRequest) -> ApiResult<OkfQualityRun> {
        self.okf.create_okf_lint_run(request).await
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
