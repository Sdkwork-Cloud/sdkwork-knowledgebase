use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    context_binding::{
        CreateKnowledgeSpaceContextBindingRequest, KnowledgeSpaceContextBinding,
        KnowledgeSpaceContextBindingList, UpdateKnowledgeSpaceContextBindingRequest,
    },
    upload::{
        CompleteKnowledgeUploadSessionRequest, CreateKnowledgeUploadSessionRequest,
        KnowledgeUploadSession,
    },
    CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest,
    CreateKnowledgeSpaceRequest, IngestionJob, KnowledgeAgentBinding, KnowledgeAgentBindingList,
    KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest, KnowledgeAgentChatResponse,
    KnowledgeAgentProfile, KnowledgeAgentProfileRequest, KnowledgeBrowserPage,
    KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeDocument, KnowledgeDocumentList,
    KnowledgeDocumentVersion, KnowledgeDocumentVersionList, KnowledgeDriveImportRequest,
    KnowledgeDriveImportResult, KnowledgeIngestRequest, KnowledgeOkfBundleFile,
    KnowledgeOkfConceptRevisionList, KnowledgeRetrievalRequest, KnowledgeRetrievalResult,
    KnowledgeSpace, ListKnowledgeBrowserRequest, OkfBundleExportRequest, OkfBundleImportRequest,
    OkfBundleImportResult, OkfConceptSummary, OkfConceptSummaryList, OkfConceptUpsertRequest,
    OkfContextPackRequest, OkfFileAnswerRequest, OkfIndexDocument, OkfLogDocument,
    OkfProfileDocument, OkfQualityRun, OkfQualityRunRequest, OkfQueryRequest, OkfQueryResult,
};

use crate::{ApiError, ApiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeAppRequestContext {
    pub tenant_id: u64,
    pub actor_id: Option<u64>,
    pub organization_id: Option<u64>,
    pub session_id: Option<String>,
}

#[async_trait]
pub trait KnowledgeSpaceAppService: Send + Sync + 'static {
    async fn create_space(&self, request: CreateKnowledgeSpaceRequest)
        -> ApiResult<KnowledgeSpace>;

    async fn retrieve_space(&self, space_id: u64) -> ApiResult<KnowledgeSpace>;
}

#[async_trait]
pub trait KnowledgeDriveImportAppService: Send + Sync + 'static {
    async fn import_drive_object(
        &self,
        request: KnowledgeDriveImportRequest,
    ) -> ApiResult<KnowledgeDriveImportResult>;
}

#[async_trait]
pub trait KnowledgeIngestAppService: Send + Sync + 'static {
    async fn create_ingest(&self, request: KnowledgeIngestRequest) -> ApiResult<IngestionJob>;

    async fn retrieve_ingest(&self, ingest_id: u64) -> ApiResult<IngestionJob>;
}

#[async_trait]
pub trait KnowledgeDocumentAppService: Send + Sync + 'static {
    async fn list_documents(&self) -> ApiResult<KnowledgeDocumentList>;

    async fn create_document(
        &self,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument>;

    async fn retrieve_document(&self, document_id: u64) -> ApiResult<KnowledgeDocument>;

    async fn update_document(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentRequest,
    ) -> ApiResult<KnowledgeDocument>;

    async fn delete_document(&self, document_id: u64) -> ApiResult<()>;

    async fn list_document_versions(
        &self,
        document_id: u64,
    ) -> ApiResult<KnowledgeDocumentVersionList>;

    async fn create_document_version(
        &self,
        document_id: u64,
        request: CreateKnowledgeDocumentVersionRequest,
    ) -> ApiResult<KnowledgeDocumentVersion>;
}

#[async_trait]
pub trait KnowledgeOkfAppService: Send + Sync + 'static {
    async fn list_okf_concepts(&self) -> ApiResult<OkfConceptSummaryList>;

    async fn retrieve_okf_concept(&self, concept_row_id: u64) -> ApiResult<OkfConceptSummary>;

    async fn list_okf_concept_revisions(
        &self,
        concept_row_id: u64,
    ) -> ApiResult<KnowledgeOkfConceptRevisionList>;

    async fn upsert_okf_concept(
        &self,
        request: OkfConceptUpsertRequest,
    ) -> ApiResult<OkfConceptSummary>;

    async fn retrieve_okf_index(&self) -> ApiResult<OkfIndexDocument>;

    async fn retrieve_okf_log(&self) -> ApiResult<OkfLogDocument>;

    async fn retrieve_okf_schema(&self) -> ApiResult<OkfProfileDocument>;

    async fn create_okf_query(&self, request: OkfQueryRequest) -> ApiResult<OkfQueryResult>;

    async fn file_okf_query_answer(
        &self,
        query_id: u64,
        request: OkfFileAnswerRequest,
    ) -> ApiResult<OkfQueryResult>;

    async fn create_okf_context_pack(
        &self,
        request: OkfContextPackRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile>;

    async fn create_okf_export(
        &self,
        request: OkfBundleExportRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile>;

    async fn retrieve_okf_export(&self, export_id: u64) -> ApiResult<KnowledgeOkfBundleFile>;

    async fn create_okf_import(
        &self,
        request: OkfBundleImportRequest,
    ) -> ApiResult<OkfBundleImportResult>;

    async fn create_okf_lint_run(&self, request: OkfQualityRunRequest) -> ApiResult<OkfQualityRun>;
}

#[async_trait]
pub trait KnowledgeBrowserApi: Send + Sync + 'static {
    async fn list_browser(
        &self,
        context: KnowledgeAppRequestContext,
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
pub trait KnowledgeUploadSessionAppService: Send + Sync + 'static {
    async fn create_upload_session(
        &self,
        request: CreateKnowledgeUploadSessionRequest,
    ) -> ApiResult<KnowledgeUploadSession>;

    async fn complete_upload_session(
        &self,
        session_id: u64,
        request: CompleteKnowledgeUploadSessionRequest,
    ) -> ApiResult<IngestionJob>;
}

#[async_trait]
pub trait KnowledgeContextBindingAppService: Send + Sync + 'static {
    async fn list_space_context_bindings(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBindingList>;

    async fn create_space_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        space_id: u64,
        request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding>;

    async fn retrieve_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBinding>;

    async fn update_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
        request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding>;

    async fn delete_context_binding(
        &self,
        context: KnowledgeAppRequestContext,
        binding_id: u64,
    ) -> ApiResult<()>;
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

    async fn create_agent_chat(
        &self,
        profile_id: u64,
        request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse>;
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

    async fn list_okf_concepts(&self) -> ApiResult<OkfConceptSummaryList> {
        Err(ApiError::not_implemented("okf.concepts.list"))
    }

    async fn retrieve_okf_concept(&self, _concept_row_id: u64) -> ApiResult<OkfConceptSummary> {
        Err(ApiError::not_implemented("okf.concepts.retrieve"))
    }

    async fn list_okf_concept_revisions(
        &self,
        _concept_row_id: u64,
    ) -> ApiResult<KnowledgeOkfConceptRevisionList> {
        Err(ApiError::not_implemented("okf.concepts.revisions.list"))
    }

    async fn upsert_okf_concept(
        &self,
        _request: OkfConceptUpsertRequest,
    ) -> ApiResult<OkfConceptSummary> {
        Err(ApiError::not_implemented("okf.concepts.upsert"))
    }

    async fn retrieve_okf_index(&self) -> ApiResult<OkfIndexDocument> {
        Err(ApiError::not_implemented("okf.bundle.index.retrieve"))
    }

    async fn retrieve_okf_log(&self) -> ApiResult<OkfLogDocument> {
        Err(ApiError::not_implemented("okf.bundle.log.retrieve"))
    }

    async fn retrieve_okf_schema(&self) -> ApiResult<OkfProfileDocument> {
        Err(ApiError::not_implemented("okf.bundle.profile.retrieve"))
    }

    async fn create_okf_query(&self, _request: OkfQueryRequest) -> ApiResult<OkfQueryResult> {
        Err(ApiError::not_implemented("okf.queries.create"))
    }

    async fn file_okf_query_answer(
        &self,
        _query_id: u64,
        _request: OkfFileAnswerRequest,
    ) -> ApiResult<OkfQueryResult> {
        Err(ApiError::not_implemented("okf.queries.fileAnswer"))
    }

    async fn create_okf_context_pack(
        &self,
        _request: OkfContextPackRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        Err(ApiError::not_implemented("okf.contextPacks.create"))
    }

    async fn create_okf_export(
        &self,
        _request: OkfBundleExportRequest,
    ) -> ApiResult<KnowledgeOkfBundleFile> {
        Err(ApiError::not_implemented("okf.bundle.export.create"))
    }

    async fn retrieve_okf_export(&self, _export_id: u64) -> ApiResult<KnowledgeOkfBundleFile> {
        Err(ApiError::not_implemented("okf.bundle.export.retrieve"))
    }

    async fn create_okf_import(
        &self,
        _request: OkfBundleImportRequest,
    ) -> ApiResult<OkfBundleImportResult> {
        Err(ApiError::not_implemented("okf.bundle.import.create"))
    }

    async fn create_okf_lint_run(
        &self,
        _request: OkfQualityRunRequest,
    ) -> ApiResult<OkfQualityRun> {
        Err(ApiError::not_implemented("okf.lintRuns.create"))
    }

    async fn list_browser(
        &self,
        _context: KnowledgeAppRequestContext,
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

    async fn create_agent_chat(
        &self,
        _profile_id: u64,
        _request: KnowledgeAgentChatRequest,
    ) -> ApiResult<KnowledgeAgentChatResponse> {
        Err(ApiError::not_implemented("agentProfiles.chat.create"))
    }

    async fn list_space_context_bindings(
        &self,
        _context: KnowledgeAppRequestContext,
        _space_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBindingList> {
        Err(ApiError::not_implemented("spaces.contextBindings.list"))
    }

    async fn create_space_context_binding(
        &self,
        _context: KnowledgeAppRequestContext,
        _space_id: u64,
        _request: CreateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        Err(ApiError::not_implemented("spaces.contextBindings.create"))
    }

    async fn retrieve_context_binding(
        &self,
        _context: KnowledgeAppRequestContext,
        _binding_id: u64,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        Err(ApiError::not_implemented("contextBindings.retrieve"))
    }

    async fn update_context_binding(
        &self,
        _context: KnowledgeAppRequestContext,
        _binding_id: u64,
        _request: UpdateKnowledgeSpaceContextBindingRequest,
    ) -> ApiResult<KnowledgeSpaceContextBinding> {
        Err(ApiError::not_implemented("contextBindings.update"))
    }

    async fn delete_context_binding(
        &self,
        _context: KnowledgeAppRequestContext,
        _binding_id: u64,
    ) -> ApiResult<()> {
        Err(ApiError::not_implemented("contextBindings.delete"))
    }

    async fn create_upload_session(
        &self,
        _request: CreateKnowledgeUploadSessionRequest,
    ) -> ApiResult<KnowledgeUploadSession> {
        Err(ApiError::not_implemented("uploadSessions.create"))
    }

    async fn complete_upload_session(
        &self,
        _session_id: u64,
        _request: CompleteKnowledgeUploadSessionRequest,
    ) -> ApiResult<IngestionJob> {
        Err(ApiError::not_implemented("uploadSessions.complete"))
    }
}
