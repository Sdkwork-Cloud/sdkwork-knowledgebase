use async_trait::async_trait;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest,
    KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest,
    KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList,
    KnowledgeWikiFileEntry, KnowledgeWikiFileEntryList, KnowledgeWikiSchemaProfileRequest,
    WikiCandidateResult, WikiCandidateResultList, WikiCandidateReviewRequest,
    WikiCompileJobRequest, WikiExportRequest, WikiIndexDocument, WikiIndexRebuildRequest,
    WikiLogEntry, WikiPagePublishRequest, WikiPageSummary, WikiQualityRun, WikiQualityRunRequest,
};

use crate::error::{BackendApiError, BackendApiResult};

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

    async fn create_wiki_compile_job(
        &self,
        _request: WikiCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        Err(BackendApiError::not_implemented("wiki.compileJobs.create"))
    }

    async fn list_wiki_candidates(&self) -> BackendApiResult<WikiCandidateResultList> {
        Err(BackendApiError::not_implemented("wiki.candidates.list"))
    }

    async fn approve_wiki_candidate(
        &self,
        _candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        Err(BackendApiError::not_implemented("wiki.candidates.approve"))
    }

    async fn reject_wiki_candidate(
        &self,
        _candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        Err(BackendApiError::not_implemented("wiki.candidates.reject"))
    }

    async fn publish_wiki_page(
        &self,
        _page_id: u64,
        _request: WikiPagePublishRequest,
    ) -> BackendApiResult<WikiPageSummary> {
        Err(BackendApiError::not_implemented("wiki.pages.publish"))
    }

    async fn create_wiki_schema_profile(
        &self,
        _request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented(
            "wiki.schema.profiles.create",
        ))
    }

    async fn update_wiki_schema_profile(
        &self,
        _profile_id: u64,
        _request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented(
            "wiki.schema.profiles.update",
        ))
    }

    async fn rebuild_wiki_index(
        &self,
        _request: WikiIndexRebuildRequest,
    ) -> BackendApiResult<WikiIndexDocument> {
        Err(BackendApiError::not_implemented("wiki.index.rebuild"))
    }

    async fn create_wiki_log_entry(
        &self,
        _request: WikiLogEntry,
    ) -> BackendApiResult<WikiLogEntry> {
        Err(BackendApiError::not_implemented("wiki.log.entries.create"))
    }

    async fn create_wiki_export(
        &self,
        _request: WikiExportRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented("wiki.exports.create"))
    }

    async fn retrieve_wiki_export(
        &self,
        _export_id: u64,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        Err(BackendApiError::not_implemented("wiki.exports.retrieve"))
    }

    async fn list_wiki_file_entries(&self) -> BackendApiResult<KnowledgeWikiFileEntryList> {
        Err(BackendApiError::not_implemented("wiki.fileEntries.list"))
    }

    async fn create_wiki_lint_run(
        &self,
        _request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        Err(BackendApiError::not_implemented("wiki.lintRuns.create"))
    }

    async fn create_wiki_eval_run(
        &self,
        _request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        Err(BackendApiError::not_implemented("wiki.evalRuns.create"))
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
        _request: WikiIndexRebuildRequest,
    ) -> BackendApiResult<WikiIndexDocument> {
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
}
