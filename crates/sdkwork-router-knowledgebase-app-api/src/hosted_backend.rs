use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::KnowledgeSpaceStore;
use sdkwork_intelligence_knowledgebase_service::{
    knowledge_embedding_build::KnowledgeEmbeddingBuildService,
    ports::{
        knowledge_drive_storage::{KnowledgeDriveStorage, PutKnowledgeObjectRequest},
        knowledge_ingestion_job_store::{CreateIngestionJobRecord, IngestionJobStore},
        knowledge_source_store::{CreateKnowledgeSourceRecord, KnowledgeSourceStore},
        knowledge_wiki_file_entry_store::{
            CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
        },
        knowledge_wiki_page_store::{
            AppendKnowledgeWikiLogEntryRecord, KnowledgeWikiPageStore,
            MarkKnowledgeWikiCurrentRevisionRecord,
        },
    },
    retrieval::KnowledgeRetrievalService,
};
use sdkwork_knowledgebase_agent_provider::{
    resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::wiki::WikiPagePublishState;
use sdkwork_knowledgebase_contract::wiki_file::WikiFileEntryType;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest,
    KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest,
    KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList,
    KnowledgeWikiFileEntry, KnowledgeWikiFileEntryList, KnowledgeWikiSchemaProfileRequest,
    WikiCandidateResult, WikiCandidateResultList, WikiCandidateReviewRequest,
    WikiCompileJobRequest, WikiExportRequest, WikiIndexDocument, WikiIndexRebuildRequest,
    WikiLogEntry, WikiPagePublishRequest, WikiPageSummary, WikiQualityRun, WikiQualityRunRequest,
};
use sdkwork_router_knowledgebase_backend_api::{
    BackendApiError, BackendApiResult, KnowledgeBackendApi,
};

use crate::{
    hosted_support::{page_to_summary, persist_wiki_schema_profile, rebuild_wiki_index_document},
    runtime::KnowledgebaseSqliteRuntime,
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct SqliteHostedBackendApi {
    runtime: KnowledgebaseSqliteRuntime,
}

impl SqliteHostedBackendApi {
    pub fn new(runtime: KnowledgebaseSqliteRuntime) -> Self {
        Self { runtime }
    }

    async fn wiki_space_for_log(&self) -> BackendApiResult<u64> {
        self.runtime
            .space_store()
            .find_first_wiki_initialized_space()
            .await
            .map_err(|error| map_internal(error.to_string()))?
            .map(|space| space.id)
            .ok_or_else(|| {
                BackendApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "wiki_space_not_initialized",
                    "no wiki-initialized knowledge space is available for this tenant",
                )
            })
    }

    async fn create_and_run_background_job<F, Fut>(
        &self,
        space_id: u64,
        source_type: &str,
        idempotency_key: String,
        run: F,
    ) -> BackendApiResult<IngestionJob>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        use sdkwork_intelligence_knowledgebase_service::ingest::KnowledgeIngestionService;
        use sdkwork_knowledgebase_contract::ingest::IngestionJobState;

        let result = self
            .runtime
            .ingestion_job_store()
            .create_or_get_job(CreateIngestionJobRecord {
                space_id,
                source_type: source_type.to_string(),
                idempotency_key,
                idempotency_fingerprint_sha256_hex: None,
            })
            .await
            .map_err(|error| map_internal(error.to_string()))?;

        let mut job = result.job;
        if job.state != IngestionJobState::Queued {
            return Ok(job);
        }

        let ingestion = KnowledgeIngestionService::new(self.runtime.ingestion_job_store());
        job = ingestion
            .mark_running(job.id)
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        match run().await {
            Ok(()) => ingestion
                .mark_succeeded(job.id)
                .await
                .map_err(|error| map_internal(error.to_string())),
            Err(detail) => ingestion
                .mark_failed(job.id, detail)
                .await
                .map_err(|error| map_internal(error.to_string())),
        }
    }
}

#[async_trait]
impl KnowledgeBackendApi for SqliteHostedBackendApi {
    async fn list_sources(&self) -> BackendApiResult<KnowledgeSourceList> {
        let items = self
            .runtime
            .source_store()
            .list_active_sources()
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        Ok(KnowledgeSourceList { items })
    }

    async fn create_source(
        &self,
        request: CreateKnowledgeSourceRequest,
    ) -> BackendApiResult<KnowledgeSource> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_knowledge_source_request",
                "space_id is required",
            ));
        }
        self.runtime
            .source_store()
            .create_source(CreateKnowledgeSourceRecord {
                space_id: request.space_id,
                source_type: request.source_type,
                provider: request.provider,
                drive_bucket: request.drive_bucket,
                drive_prefix: request.drive_prefix,
            })
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn create_wiki_compile_job(
        &self,
        request: WikiCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_compile_job_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        self.create_and_run_background_job(
            space_id,
            "wiki_compile",
            format!(
                "wiki-compile:{}:{}",
                request.space_id,
                request.source_id.unwrap_or(0)
            ),
            || async move {
                rebuild_wiki_index_document(&runtime, space_id)
                    .await
                    .map(|_| ())
                    .map_err(|error| format!("{error:?}"))
            },
        )
        .await
    }

    async fn list_wiki_candidates(&self) -> BackendApiResult<WikiCandidateResultList> {
        let items = self
            .runtime
            .wiki_page_store()
            .list_candidate_pages()
            .await
            .map_err(map_wiki_page)?
            .into_iter()
            .map(|(id, state)| WikiCandidateResult {
                id,
                state: state.as_str().to_string(),
            })
            .collect();
        Ok(WikiCandidateResultList { items })
    }

    async fn approve_wiki_candidate(
        &self,
        candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        self.runtime
            .wiki_page_store()
            .update_page_publish_state(candidate_id, WikiPagePublishState::Published)
            .await
            .map_err(map_wiki_page)?;
        Ok(WikiCandidateResult {
            id: candidate_id,
            state: WikiPagePublishState::Published.as_str().to_string(),
        })
    }

    async fn reject_wiki_candidate(
        &self,
        candidate_id: u64,
        _request: WikiCandidateReviewRequest,
    ) -> BackendApiResult<WikiCandidateResult> {
        self.runtime
            .wiki_page_store()
            .update_page_publish_state(candidate_id, WikiPagePublishState::Rejected)
            .await
            .map_err(map_wiki_page)?;
        Ok(WikiCandidateResult {
            id: candidate_id,
            state: WikiPagePublishState::Rejected.as_str().to_string(),
        })
    }

    async fn publish_wiki_page(
        &self,
        page_id: u64,
        _request: WikiPagePublishRequest,
    ) -> BackendApiResult<WikiPageSummary> {
        let page = self
            .runtime
            .wiki_page_store()
            .get_page_by_id(page_id)
            .await
            .map_err(map_wiki_page)?;
        let revision_id = page.current_revision_id.ok_or_else(|| {
            BackendApiError::new(
                axum::http::StatusCode::CONFLICT,
                "wiki_page_not_ready",
                format!("wiki page {page_id} has no current revision to publish"),
            )
        })?;
        let published = self
            .runtime
            .wiki_page_store()
            .mark_current_revision(MarkKnowledgeWikiCurrentRevisionRecord {
                page_id,
                revision_id,
                publish_state: WikiPagePublishState::Published,
            })
            .await
            .map_err(map_wiki_page)?;
        Ok(page_to_summary(published))
    }

    async fn create_wiki_schema_profile(
        &self,
        request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_schema_profile_request",
                "space_id is required",
            ));
        }
        persist_wiki_schema_profile(&self.runtime, request.space_id)
            .await
            .map_err(map_api_error)
    }

    async fn update_wiki_schema_profile(
        &self,
        _profile_id: u64,
        request: KnowledgeWikiSchemaProfileRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        self.create_wiki_schema_profile(request).await
    }

    async fn rebuild_wiki_index(
        &self,
        request: WikiIndexRebuildRequest,
    ) -> BackendApiResult<WikiIndexDocument> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_index_rebuild_request",
                "space_id is required",
            ));
        }
        rebuild_wiki_index_document(&self.runtime, request.space_id)
            .await
            .map_err(map_api_error)
    }

    async fn create_wiki_log_entry(&self, request: WikiLogEntry) -> BackendApiResult<WikiLogEntry> {
        let space_id = self.wiki_space_for_log().await?;
        self.runtime
            .wiki_page_store()
            .append_log_entry(AppendKnowledgeWikiLogEntryRecord {
                space_id,
                event_type: request.event_type.as_str().to_string(),
                event_time: request.occurred_at,
                title: request.title.clone(),
                actor: request.actor.clone(),
                affected_pages: request.affected_pages.clone(),
                audit_event_id: request.audit_event_id.clone(),
                warnings: request.warnings.clone(),
                privacy_level: "internal".to_string(),
            })
            .await
            .map_err(map_wiki_page)
    }

    async fn create_wiki_export(
        &self,
        request: WikiExportRequest,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        if request.space_id == 0 || request.export_type.trim().is_empty() {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_export_request",
                "space_id and export_type are required",
            ));
        }
        let document = rebuild_wiki_index_document(&self.runtime, request.space_id)
            .await
            .map_err(map_api_error)?;
        let logical_path = format!(
            "output/exports/{}-{}.md",
            request.export_type.trim(),
            request.space_id
        );
        let object_ref = self
            .runtime
            .drive_storage()
            .put_object(PutKnowledgeObjectRequest::text(
                logical_path.clone(),
                "output_export",
                document.markdown,
                None,
            ))
            .await
            .map_err(|error| map_api_error(error.into()))?;
        self.runtime
            .wiki_file_entry_store()
            .create_file_entry(CreateKnowledgeWikiFileEntryRecord {
                space_id: request.space_id,
                logical_path,
                entry_type: WikiFileEntryType::OutputExport,
                artifact_role: object_ref.object_role.clone(),
                drive_bucket: object_ref.bucket.clone(),
                drive_object_key: object_ref.object_key.clone(),
                checksum_sha256_hex: object_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn retrieve_wiki_export(
        &self,
        export_id: u64,
    ) -> BackendApiResult<KnowledgeWikiFileEntry> {
        self.runtime
            .wiki_file_entry_store()
            .get_file_entry_by_id(export_id)
            .await
            .map_err(|error| {
                let detail = error.to_string();
                if detail.contains("missing wiki file entry") {
                    BackendApiError::new(
                        axum::http::StatusCode::NOT_FOUND,
                        "wiki_export_not_found",
                        detail,
                    )
                } else {
                    map_internal(detail)
                }
            })
    }

    async fn list_wiki_file_entries(&self) -> BackendApiResult<KnowledgeWikiFileEntryList> {
        let items = self
            .runtime
            .wiki_file_entry_store()
            .list_file_entries()
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        Ok(KnowledgeWikiFileEntryList { items })
    }

    async fn create_wiki_lint_run(
        &self,
        request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_quality_run_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        let job = self
            .create_and_run_background_job(
                space_id,
                "wiki_lint_run",
                format!(
                    "wiki-lint:{}:{}",
                    request.space_id,
                    request.profile.as_deref().unwrap_or("default")
                ),
                || async move {
                    rebuild_wiki_index_document(&runtime, space_id)
                        .await
                        .map(|_| ())
                        .map_err(|error| format!("{error:?}"))
                },
            )
            .await?;
        Ok(WikiQualityRun {
            id: job.id,
            state: format!("{:?}", job.state).to_ascii_lowercase(),
        })
    }

    async fn create_wiki_eval_run(
        &self,
        request: WikiQualityRunRequest,
    ) -> BackendApiResult<WikiQualityRun> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_wiki_quality_run_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        let job = self
            .create_and_run_background_job(
                space_id,
                "wiki_eval_run",
                format!(
                    "wiki-eval:{}:{}",
                    request.space_id,
                    request.profile.as_deref().unwrap_or("default")
                ),
                || async move {
                    persist_wiki_schema_profile(&runtime, space_id)
                        .await
                        .map(|_| ())
                        .map_err(|error| format!("{error:?}"))
                },
            )
            .await?;
        Ok(WikiQualityRun {
            id: job.id,
            state: format!("{:?}", job.state).to_ascii_lowercase(),
        })
    }

    async fn create_index(
        &self,
        request: KnowledgeIndexRequest,
    ) -> BackendApiResult<KnowledgeIndex> {
        self.runtime
            .index_store()
            .create_index(request)
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn retrieve_index(&self, index_id: u64) -> BackendApiResult<KnowledgeIndex> {
        self.runtime
            .index_store()
            .get_index(index_id)
            .await
            .map_err(|error| {
                let detail = error.to_string();
                if detail.contains("missing knowledge index") {
                    BackendApiError::new(
                        axum::http::StatusCode::NOT_FOUND,
                        "knowledge_index_not_found",
                        detail,
                    )
                } else {
                    map_internal(detail)
                }
            })
    }

    async fn rebuild_index(
        &self,
        index_id: u64,
        request: WikiIndexRebuildRequest,
    ) -> BackendApiResult<WikiIndexDocument> {
        let index = self.retrieve_index(index_id).await?;
        let space_id = if request.space_id == 0 {
            index.space_id
        } else {
            request.space_id
        };

        let space = self
            .runtime
            .space_store()
            .get_space(space_id)
            .await
            .map_err(|error| map_internal(error.to_string()))?;

        if space.knowledge_mode == KnowledgeAgentKnowledgeMode::Rag {
            let indexed = if let Ok(client) = resolve_claw_router_client_from_env() {
                let embedder = ClawRouterEmbeddingClient::new(Arc::new(client));
                let build =
                    KnowledgeEmbeddingBuildService::new(self.runtime.embedding_store(), embedder);
                build
                    .embed_space_chunks(self.runtime.tenant_id(), index_id, space_id, None, None)
                    .await
                    .map_err(|error| map_internal(error.to_string()))?
            } else {
                return Err(map_internal(
                    "rag index rebuild requires claw-router embedding client".to_string(),
                ));
            };

            return Ok(WikiIndexDocument {
                markdown: format!(
                    "# RAG embedding index rebuild\n\nIndexed {indexed} chunk embedding(s) for index {index_id} in space {space_id}."
                ),
            });
        }

        rebuild_wiki_index_document(&self.runtime, space_id)
            .await
            .map_err(map_api_error)
    }

    async fn create_retrieval_profile(
        &self,
        request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        self.runtime
            .retrieval_profile_store()
            .create_profile(request)
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn retrieve_retrieval_profile(
        &self,
        profile_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        self.runtime
            .retrieval_profile_store()
            .get_profile(profile_id)
            .await
            .map_err(|error| {
                let detail = error.to_string();
                if detail.contains("missing retrieval profile") {
                    BackendApiError::new(
                        axum::http::StatusCode::NOT_FOUND,
                        "retrieval_profile_not_found",
                        detail,
                    )
                } else {
                    map_internal(detail)
                }
            })
    }

    async fn update_retrieval_profile(
        &self,
        profile_id: u64,
        request: KnowledgeRetrievalProfileRequest,
    ) -> BackendApiResult<KnowledgeRetrievalProfile> {
        self.runtime
            .retrieval_profile_store()
            .update_profile(profile_id, request)
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn list_retrieval_traces(&self) -> BackendApiResult<KnowledgeRetrievalTraceList> {
        let records = self
            .runtime
            .retrieval_store()
            .list_trace_summaries(200)
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        let items = records
            .into_iter()
            .map(|record| KnowledgeRetrievalTrace {
                retrieval_trace_id: record.retrieval_trace_id,
                status: record.status,
                latency_ms: record.latency_ms,
                result_count: record.result_count,
            })
            .collect();
        Ok(KnowledgeRetrievalTraceList { items })
    }

    async fn retrieve_retrieval_trace(
        &self,
        trace_id: u64,
    ) -> BackendApiResult<KnowledgeRetrievalTrace> {
        let result = KnowledgeRetrievalService::new(
            self.runtime.retrieval_store(),
            self.runtime.retrieval_store(),
        )
        .retrieve_persisted(self.runtime.tenant_id(), trace_id)
        .await
        .map_err(map_retrieval)?;
        result.trace.ok_or_else(|| {
            BackendApiError::new(
                axum::http::StatusCode::NOT_FOUND,
                "retrieval_trace_not_found",
                format!("retrieval trace was not found: {trace_id}"),
            )
        })
    }

    async fn retrieve_provider_health(&self) -> BackendApiResult<KnowledgeProviderHealth> {
        self.runtime.readiness_check().await.map_err(|error| {
            BackendApiError::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "provider_health_check_failed",
                error.to_string(),
            )
        })?;
        Ok(KnowledgeProviderHealth {
            status: "ok".to_string(),
            provider_id: "sdkwork-knowledgebase-sqlite".to_string(),
            checked_at: None,
        })
    }
}

fn map_internal(detail: String) -> BackendApiError {
    BackendApiError::new(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "knowledgebase_store_failed",
        detail,
    )
}

fn map_api_error(error: crate::ApiError) -> BackendApiError {
    error.to_backend_api_error()
}

fn map_wiki_page(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::KnowledgeWikiPageStoreError,
) -> BackendApiError {
    let detail = error.to_string();
    if detail.contains("missing wiki page") {
        BackendApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "wiki_page_not_found",
            detail,
        )
    } else {
        map_internal(detail)
    }
}

fn map_retrieval(
    error: sdkwork_intelligence_knowledgebase_service::retrieval::KnowledgeRetrievalServiceError,
) -> BackendApiError {
    match error {
        sdkwork_intelligence_knowledgebase_service::retrieval::KnowledgeRetrievalServiceError::InvalidRequest(detail) => {
            BackendApiError::new(axum::http::StatusCode::BAD_REQUEST, "invalid_retrieval_request", detail)
        }
        sdkwork_intelligence_knowledgebase_service::retrieval::KnowledgeRetrievalServiceError::TraceStore(error) => {
            let detail = error.to_string();
            if detail.contains("not found") {
                BackendApiError::new(axum::http::StatusCode::NOT_FOUND, "retrieval_trace_not_found", detail)
            } else {
                map_internal(detail)
            }
        }
        other => map_internal(other.to_string()),
    }
}
