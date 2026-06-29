use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest,
    KnowledgeOkfBundleFile, KnowledgeOkfBundleFileList, KnowledgeOkfProfileRequest,
    KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest,
    KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList,
    KnowledgeTenantStatus, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult,
    OkfCandidateResult, OkfCandidateResultList, OkfCandidateReviewRequest, OkfCompileJobRequest,
    OkfConceptPublishRequest, OkfConceptSummary, OkfIndexDocument, OkfIndexRebuildRequest,
    OkfLogEntry, OkfQualityRun, OkfQualityRunRequest,
};

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
        Err(BackendApiError::not_implemented("sources.list"))
    }

    async fn create_source(
        &self,
        _request: CreateKnowledgeSourceRequest,
    ) -> BackendApiResult<KnowledgeSource> {
        Err(BackendApiError::not_implemented("sources.create"))
    }

    async fn create_okf_compile_job(
        &self,
        _request: OkfCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        Err(BackendApiError::not_implemented("okf.compileJobs.create"))
    }

    async fn list_okf_candidates(
        &self,
        _space_id: u64,
    ) -> BackendApiResult<OkfCandidateResultList> {
        Err(BackendApiError::not_implemented("okf.candidates.list"))
    }

    async fn approve_okf_candidate(
        &self,
        _candidate_id: u64,
        _request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        Err(BackendApiError::not_implemented("okf.candidates.approve"))
    }

    async fn reject_okf_candidate(
        &self,
        _candidate_id: u64,
        _request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        Err(BackendApiError::not_implemented("okf.candidates.reject"))
    }

    async fn publish_okf_concept(
        &self,
        _concept_id: u64,
        _request: OkfConceptPublishRequest,
    ) -> BackendApiResult<OkfConceptSummary> {
        Err(BackendApiError::not_implemented("okf.concepts.publish"))
    }

    async fn create_okf_profile(
        &self,
        _request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::not_implemented("okf.profile.create"))
    }

    async fn update_okf_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::not_implemented("okf.profile.update"))
    }

    async fn rebuild_okf_index(
        &self,
        _request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
        Err(BackendApiError::not_implemented("okf.bundle.index.rebuild"))
    }

    async fn create_okf_log_entry(&self, _request: OkfLogEntry) -> BackendApiResult<OkfLogEntry> {
        Err(BackendApiError::not_implemented("okf.log.entries.create"))
    }

    async fn create_okf_export(
        &self,
        _request: OkfBundleExportRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::not_implemented("okf.bundle.export.create"))
    }

    async fn create_okf_import(
        &self,
        _request: OkfBundleImportRequest,
    ) -> BackendApiResult<OkfBundleImportResult> {
        Err(BackendApiError::not_implemented("okf.bundle.import.create"))
    }

    async fn retrieve_okf_export(
        &self,
        _export_id: u64,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        Err(BackendApiError::not_implemented(
            "okf.bundle.export.retrieve",
        ))
    }

    async fn list_okf_bundle_files(&self) -> BackendApiResult<KnowledgeOkfBundleFileList> {
        Err(BackendApiError::not_implemented("okf.bundle.files.list"))
    }

    async fn create_okf_lint_run(
        &self,
        _request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        Err(BackendApiError::not_implemented("okf.lintRuns.create"))
    }

    async fn create_okf_eval_run(
        &self,
        _request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        Err(BackendApiError::not_implemented("okf.evalRuns.create"))
    }

    async fn create_index(
        &self,
        _request: KnowledgeIndexRequest,
    ) -> BackendApiResult<KnowledgeIndex> {
        Err(BackendApiError::not_implemented("indexes.create"))
    }

    async fn retrieve_index(&self, _index_id: u64) -> BackendApiResult<KnowledgeIndex> {
        Err(BackendApiError::not_implemented("indexes.retrieve"))
    }

    async fn rebuild_index(
        &self,
        _index_id: u64,
        _request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
        Err(BackendApiError::not_implemented("indexes.rebuild"))
    }

    async fn create_retrieval_profile(
        &self,
        _request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::not_implemented("retrievalProfiles.create"))
    }

    async fn retrieve_retrieval_profile(
        &self,
        _profile_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::not_implemented(
            "retrievalProfiles.retrieve",
        ))
    }

    async fn update_retrieval_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        Err(BackendApiError::not_implemented("retrievalProfiles.update"))
    }

    async fn list_retrieval_traces(&self) -> BackendApiResult<KnowledgeRetrievalTraceList> {
        Err(BackendApiError::not_implemented("retrievalTraces.list"))
    }

    async fn retrieve_retrieval_trace(
        &self,
        _trace_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalTrace> {
        Err(BackendApiError::not_implemented("retrievalTraces.retrieve"))
    }

    async fn retrieve_provider_health(&self) -> BackendApiResult<KnowledgeProviderHealth> {
        Err(BackendApiError::not_implemented("providerHealth.retrieve"))
    }

    /// Retrieves the caller's own tenant knowledgebase status.
    ///
    /// **Security**: The tenant is identified by the authenticated principal's token claims.
    /// Returns space count, document count, and status for the current tenant.
    async fn retrieve_current_tenant(&self) -> BackendApiResult<KnowledgeTenantStatus> {
        Err(BackendApiError::not_implemented("tenants.current"))
    }
}
