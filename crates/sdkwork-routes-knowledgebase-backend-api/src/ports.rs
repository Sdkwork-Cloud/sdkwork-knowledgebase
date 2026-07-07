use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    AnonymizeKnowledgeAuditSubjectRequest, AnonymizeKnowledgeAuditSubjectResult,
    CreateKnowledgeSourceRequest, ExportKnowledgeAuditEventsRequest, IngestionJob,
    KnowledgeAuditEventExport, KnowledgeIndex, KnowledgeIndexList, KnowledgeIndexRequest,
    KnowledgeOkfBundleFile, KnowledgeOkfBundleFileList, KnowledgeOkfProfileRequest,
    KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest,
    KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList,
    KnowledgeSpace, KnowledgeSpaceMemberList, KnowledgeTenantStatus, OkfBundleExportRequest,
    OkfBundleImportRequest, OkfBundleImportResult, OkfCandidateResult, OkfCandidateResultList,
    OkfCandidateReviewRequest, OkfCompileJobRequest, OkfConceptPublishRequest, OkfConceptSummary,
    OkfIndexDocument, OkfIndexRebuildRequest, OkfLogEntry, OkfQualityRun, OkfQualityRunRequest,
};
use sdkwork_utils_rust::SdkWorkPageData;

use crate::error::{BackendApiError, BackendApiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBackendRequestContext {
    pub tenant_id: u64,
    pub operator_id: Option<u64>,
    pub organization_id: Option<u64>,
    pub permission_scope: Vec<String>,
}

#[async_trait]
pub trait KnowledgeBackendApi: Send + Sync + 'static {
    async fn list_sources(&self) -> BackendApiResult<KnowledgeSourceList> {
        Err(BackendApiError::unsupported_operation("sources.list"))
    }

    async fn create_source(
        &self,
        _request: CreateKnowledgeSourceRequest,
    ) -> BackendApiResult<KnowledgeSource> {
        Err(BackendApiError::unsupported_operation("sources.create"))
    }

    async fn create_okf_compile_job(
        &self,
        _request: OkfCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        Err(BackendApiError::unsupported_operation(
            "okf.compileJobs.create",
        ))
    }

    async fn list_okf_candidates(
        &self,
        _space_id: u64,
    ) -> BackendApiResult<OkfCandidateResultList> {
        Err(BackendApiError::unsupported_operation(
            "okf.candidates.list",
        ))
    }

    async fn approve_okf_candidate(
        &self,
        _candidate_id: u64,
        _request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        Err(BackendApiError::unsupported_operation(
            "okf.candidates.approve",
        ))
    }

    async fn reject_okf_candidate(
        &self,
        _candidate_id: u64,
        _request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        Err(BackendApiError::unsupported_operation(
            "okf.candidates.reject",
        ))
    }

    async fn publish_okf_concept(
        &self,
        _concept_id: u64,
        _request: OkfConceptPublishRequest,
    ) -> BackendApiResult<OkfConceptSummary> {
        Err(BackendApiError::unsupported_operation(
            "okf.concepts.publish",
        ))
    }

    async fn create_okf_profile(
        &self,
        _request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::unsupported_operation("okf.profile.create"))
    }

    async fn update_okf_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::unsupported_operation("okf.profile.update"))
    }

    async fn rebuild_okf_index(
        &self,
        _request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
        Err(BackendApiError::unsupported_operation(
            "okf.bundle.index.create",
        ))
    }

    async fn create_okf_log_entry(&self, _request: OkfLogEntry) -> BackendApiResult<OkfLogEntry> {
        Err(BackendApiError::unsupported_operation(
            "okf.log.entries.create",
        ))
    }

    async fn create_okf_export(
        &self,
        _request: OkfBundleExportRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::unsupported_operation(
            "okf.bundle.export.create",
        ))
    }

    async fn create_okf_import(
        &self,
        _request: OkfBundleImportRequest,
    ) -> BackendApiResult<OkfBundleImportResult> {
        Err(BackendApiError::unsupported_operation(
            "okf.bundle.import.create",
        ))
    }

    async fn retrieve_okf_export(
        &self,
        _export_id: u64,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::unsupported_operation(
            "okf.bundle.export.retrieve",
        ))
    }

    async fn list_okf_bundle_files(&self) -> BackendApiResult<KnowledgeOkfBundleFileList> {
        Err(BackendApiError::unsupported_operation(
            "okf.bundle.files.list",
        ))
    }

    async fn create_okf_lint_run(
        &self,
        _request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        Err(BackendApiError::unsupported_operation(
            "okf.lintRuns.create",
        ))
    }

    async fn create_okf_eval_run(
        &self,
        _request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        Err(BackendApiError::unsupported_operation(
            "okf.evalRuns.create",
        ))
    }

    async fn list_indexes(&self) -> BackendApiResult<KnowledgeIndexList> {
        Err(BackendApiError::unsupported_operation("indexes.list"))
    }

    async fn create_index(
        &self,
        _request: KnowledgeIndexRequest,
    ) -> BackendApiResult<KnowledgeIndex> {
        Err(BackendApiError::unsupported_operation("indexes.create"))
    }

    async fn retrieve_index(&self, _index_id: u64) -> BackendApiResult<KnowledgeIndex> {
        Err(BackendApiError::unsupported_operation("indexes.retrieve"))
    }

    async fn rebuild_index(
        &self,
        _index_id: u64,
        _request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
        Err(BackendApiError::unsupported_operation("indexes.rebuild"))
    }

    async fn create_retrieval_profile(
        &self,
        _request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::unsupported_operation(
            "retrievalProfiles.create",
        ))
    }

    async fn retrieve_retrieval_profile(
        &self,
        _profile_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::unsupported_operation(
            "retrievalProfiles.retrieve",
        ))
    }

    async fn update_retrieval_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::unsupported_operation(
            "retrievalProfiles.update",
        ))
    }

    async fn list_retrieval_traces(&self) -> BackendApiResult<KnowledgeRetrievalTraceList> {
        Err(BackendApiError::unsupported_operation(
            "retrievalTraces.list",
        ))
    }

    async fn retrieve_retrieval_trace(
        &self,
        _trace_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalTrace> {
        Err(BackendApiError::unsupported_operation(
            "retrievalTraces.retrieve",
        ))
    }

    async fn retrieve_provider_health(&self) -> BackendApiResult<KnowledgeProviderHealth> {
        Err(BackendApiError::unsupported_operation(
            "providerHealth.list",
        ))
    }

    /// Retrieves the caller's own tenant knowledgebase status.
    ///
    /// **Security**: The tenant is identified by the authenticated principal's token claims.
    /// Returns space count, document count, and status for the current tenant.
    async fn retrieve_current_tenant(&self) -> BackendApiResult<KnowledgeTenantStatus> {
        Err(BackendApiError::unsupported_operation("tenants.current"))
    }

    async fn list_spaces(
        &self,
        _cursor: Option<String>,
        _page_size: Option<u32>,
    ) -> BackendApiResult<SdkWorkPageData<KnowledgeSpace>> {
        Err(BackendApiError::unsupported_operation("spaces.list"))
    }

    async fn list_space_members(
        &self,
        _space_id: u64,
        _cursor: Option<String>,
        _page_size: Option<u32>,
    ) -> BackendApiResult<KnowledgeSpaceMemberList> {
        Err(BackendApiError::unsupported_operation(
            "spaces.members.list",
        ))
    }

    async fn export_audit_events(
        &self,
        _request: ExportKnowledgeAuditEventsRequest,
    ) -> BackendApiResult<KnowledgeAuditEventExport> {
        Err(BackendApiError::unsupported_operation(
            "compliance.auditEvents.export.create",
        ))
    }

    async fn anonymize_audit_subject(
        &self,
        _request: AnonymizeKnowledgeAuditSubjectRequest,
    ) -> BackendApiResult<AnonymizeKnowledgeAuditSubjectResult> {
        Err(BackendApiError::unsupported_operation(
            "compliance.auditEvents.anonymizeActor.create",
        ))
    }
}
