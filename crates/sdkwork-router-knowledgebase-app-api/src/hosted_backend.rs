use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_space_store::KnowledgeSpaceStore;
use sdkwork_intelligence_knowledgebase_service::{
    knowledge_embedding_build::KnowledgeEmbeddingBuildService,
    okf::{load_import_bundle_from_drive, ExportOkfBundleRequest, OkfBundleExporterService},
    ports::{
        knowledge_drive_storage::{KnowledgeDriveStorage, PutKnowledgeObjectRequest},
        knowledge_ingestion_job_store::{CreateIngestionJobRecord, IngestionJobStore},
        knowledge_okf_bundle_file_store::{
            CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
        },
        knowledge_okf_candidate_store::KnowledgeOkfCandidateStore,
        knowledge_okf_concept_store::{AppendKnowledgeOkfLogEntryRecord, KnowledgeOkfConceptStore},
        knowledge_source_store::{CreateKnowledgeSourceRecord, KnowledgeSourceStore},
    },
    retrieval::KnowledgeRetrievalService,
};
use sdkwork_knowledgebase_agent_provider::{
    resolve_claw_router_client_from_env, ClawRouterEmbeddingClient,
};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::OkfBundleFileKind;
use sdkwork_knowledgebase_contract::OkfConceptPublishState;
use sdkwork_knowledgebase_contract::{
    CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest,
    KnowledgeOkfBundleFile, KnowledgeOkfBundleFileList, KnowledgeOkfProfileRequest,
    KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest,
    KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList,
    OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult, OkfCandidateResult,
    OkfCandidateResultList, OkfCandidateReviewRequest, OkfCompileJobRequest,
    OkfConceptPublishRequest, OkfConceptSummary, OkfIndexDocument, OkfIndexRebuildRequest,
    OkfLogEntry, OkfQualityRun, OkfQualityRunRequest,
};
use sdkwork_router_knowledgebase_backend_api::{
    BackendApiError, BackendApiResult, KnowledgeBackendApi,
};

use crate::{
    hosted_support::{
        concept_to_summary, import_okf_bundle, persist_okf_profile, rebuild_okf_index_document,
        run_okf_bundle_lint,
    },
    runtime::KnowledgebaseRuntime,
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct HostedBackendApi {
    runtime: KnowledgebaseRuntime,
}

impl HostedBackendApi {
    pub fn new(runtime: KnowledgebaseRuntime) -> Self {
        Self { runtime }
    }

    async fn okf_space_for_log(&self) -> BackendApiResult<u64> {
        self.runtime
            .space_store()
            .find_first_okf_bundle_initialized_space()
            .await
            .map_err(|error| map_internal(error.to_string()))?
            .map(|space| space.id)
            .ok_or_else(|| {
                BackendApiError::new(
                    axum::http::StatusCode::NOT_FOUND,
                    "okf_bundle_not_initialized",
                    "no okf-bundle-initialized knowledge space is available for this tenant",
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
impl KnowledgeBackendApi for HostedBackendApi {
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

    async fn create_okf_compile_job(
        &self,
        request: OkfCompileJobRequest,
    ) -> BackendApiResult<IngestionJob> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_compile_job_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        self.create_and_run_background_job(
            space_id,
            "okf_compile",
            format!(
                "okf-compile:{}:{}",
                request.space_id,
                request.source_id.unwrap_or(0)
            ),
            || async move {
                rebuild_okf_index_document(&runtime, space_id)
                    .await
                    .map(|_| ())
                    .map_err(|error| format!("{error:?}"))
            },
        )
        .await
    }

    async fn list_okf_candidates(&self) -> BackendApiResult<OkfCandidateResultList> {
        let items = self
            .runtime
            .okf_candidate_store()
            .list_open_candidates(None)
            .await
            .map_err(|error| map_internal(error.to_string()))?
            .into_iter()
            .map(|candidate| OkfCandidateResult {
                id: candidate.concept_row_id,
                state: candidate.publish_state.as_str().to_string(),
            })
            .collect();
        Ok(OkfCandidateResultList { items })
    }

    async fn approve_okf_candidate(
        &self,
        candidate_id: u64,
        _request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        let actor = self.runtime.operator_id().to_string();
        let published = crate::hosted_support::publish_okf_concept_revision(
            &self.runtime,
            candidate_id,
            &actor,
        )
        .await
        .map_err(map_api_error)?;
        Ok(OkfCandidateResult {
            id: published.id,
            state: published.publish_state.as_str().to_string(),
        })
    }

    async fn reject_okf_candidate(
        &self,
        candidate_id: u64,
        request: OkfCandidateReviewRequest,
    ) -> BackendApiResult<OkfCandidateResult> {
        self.runtime
            .okf_concept_store()
            .update_concept_publish_state(candidate_id, OkfConceptPublishState::Rejected)
            .await
            .map_err(map_okf_concept)?;
        self.runtime
            .okf_candidate_store()
            .update_candidate_state_by_concept_row_id(
                candidate_id,
                OkfConceptPublishState::Rejected,
                request.reviewer_id,
                request.note,
            )
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        Ok(OkfCandidateResult {
            id: candidate_id,
            state: OkfConceptPublishState::Rejected.as_str().to_string(),
        })
    }

    async fn publish_okf_concept(
        &self,
        concept_id: u64,
        _request: OkfConceptPublishRequest,
    ) -> BackendApiResult<OkfConceptSummary> {
        let actor = self.runtime.operator_id().to_string();
        let published =
            crate::hosted_support::publish_okf_concept_revision(&self.runtime, concept_id, &actor)
                .await
                .map_err(map_api_error)?;
        Ok(concept_to_summary(published))
    }

    async fn create_okf_profile(
        &self,
        request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_profile_request",
                "space_id is required",
            ));
        }
        persist_okf_profile(&self.runtime, request.space_id)
            .await
            .map_err(map_api_error)
    }

    async fn update_okf_profile(
        &self,
        _profile_id: u64,
        request: KnowledgeOkfProfileRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        self.create_okf_profile(request).await
    }

    async fn rebuild_okf_index(
        &self,
        request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_index_rebuild_request",
                "space_id is required",
            ));
        }
        rebuild_okf_index_document(&self.runtime, request.space_id)
            .await
            .map_err(map_api_error)
    }

    async fn create_okf_log_entry(&self, request: OkfLogEntry) -> BackendApiResult<OkfLogEntry> {
        let space_id = self.okf_space_for_log().await?;
        self.runtime
            .okf_concept_store()
            .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
                space_id,
                event_type: request.event_type.as_str().to_string(),
                event_time: request.occurred_at,
                title: request.title.clone(),
                actor: request.actor.clone(),
                affected_concepts: request.affected_concepts.clone(),
                audit_event_id: request.audit_event_id.clone(),
                warnings: request.warnings.clone(),
                privacy_level: "internal".to_string(),
            })
            .await
            .map_err(map_okf_concept)
    }

    async fn create_okf_export(
        &self,
        request: OkfBundleExportRequest,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        if request.space_id == 0 || request.export_type.trim().is_empty() {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_export_request",
                "space_id and export_type are required",
            ));
        }
        let source_object_refs = if request.export_type.trim() == "okf_with_sources" {
            self.runtime
                .object_ref_store()
                .list_object_refs_by_logical_path_prefix(request.space_id, "sources/raw/")
                .await
                .map_err(|error| map_internal(error.to_string()))?
        } else {
            Vec::new()
        };
        let exported = OkfBundleExporterService::new(
            self.runtime.drive_storage(),
            self.runtime.okf_concept_store(),
        )
        .with_source_object_refs(source_object_refs)
        .export_bundle(ExportOkfBundleRequest {
            space_id: request.space_id,
            export_type: request.export_type,
        })
        .await
        .map_err(|error| map_internal(error.to_string()))?;
        self.runtime
            .okf_bundle_file_store()
            .create_file_entry(CreateKnowledgeOkfBundleFileRecord {
                space_id: request.space_id,
                logical_path: exported.manifest_path,
                file_kind: OkfBundleFileKind::OutputExport,
                artifact_role: exported.manifest_ref.object_role.clone(),
                drive_bucket: exported.manifest_ref.bucket.clone(),
                drive_object_key: exported.manifest_ref.object_key.clone(),
                checksum_sha256_hex: exported.manifest_ref.checksum_sha256_hex.clone(),
            })
            .await
            .map_err(|error| map_internal(error.to_string()))
    }

    async fn create_okf_import(
        &self,
        request: OkfBundleImportRequest,
    ) -> BackendApiResult<OkfBundleImportResult> {
        if request.space_id == 0 || request.import_type.trim().is_empty() {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_import_request",
                "space_id and import_type are required",
            ));
        }
        let import_type = request.import_type.trim();
        if import_type != "okf_strict" && import_type != "okf_bundle" {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_import_request",
                format!("unsupported import_type: {import_type}"),
            ));
        }
        let publish = import_type == "okf_strict";
        let space_id = request.space_id;
        let files = load_import_bundle_from_drive(self.runtime.drive_storage(), space_id)
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        let actor = self.runtime.operator_id().to_string();
        import_okf_bundle(&self.runtime, space_id, &actor, publish, files)
            .await
            .map_err(map_api_error)
    }

    async fn retrieve_okf_export(
        &self,
        export_id: u64,
    ) -> BackendApiResult<KnowledgeOkfBundleFile> {
        self.runtime
            .okf_bundle_file_store()
            .get_file_entry_by_id(export_id)
            .await
            .map_err(|error| {
                let detail = error.to_string();
                if detail.contains("missing okf bundle file") {
                    BackendApiError::new(
                        axum::http::StatusCode::NOT_FOUND,
                        "okf_export_not_found",
                        detail,
                    )
                } else {
                    map_internal(detail)
                }
            })
    }

    async fn list_okf_bundle_files(&self) -> BackendApiResult<KnowledgeOkfBundleFileList> {
        let items = self
            .runtime
            .okf_bundle_file_store()
            .list_file_entries()
            .await
            .map_err(|error| map_internal(error.to_string()))?;
        Ok(KnowledgeOkfBundleFileList { items })
    }

    async fn create_okf_lint_run(
        &self,
        request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_quality_run_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        let job = self
            .create_and_run_background_job(
                space_id,
                "okf_lint_run",
                format!(
                    "okf-lint:{}:{}",
                    request.space_id,
                    request.profile.as_deref().unwrap_or("default")
                ),
                || async move {
                    let lint_result = run_okf_bundle_lint(&runtime, space_id)
                        .await
                        .map_err(|error| format!("{error:?}"))?;
                    let report_path = format!("output/lint-reports/{space_id}.json");
                    runtime
                        .drive_storage()
                        .put_object(PutKnowledgeObjectRequest {
                            logical_path: report_path,
                            object_role: "output_export".to_string(),
                            content_type: "application/json; charset=utf-8".to_string(),
                            body: serde_json::to_vec_pretty(&lint_result).map_err(|error| {
                                format!("failed to serialize lint report: {error}")
                            })?,
                            checksum_sha256_hex: None,
                        })
                        .await
                        .map_err(|error| format!("{error:?}"))?;
                    if lint_result.conformance != "pass" {
                        return Err(format!(
                            "okf bundle lint failed with {} issue(s)",
                            lint_result.issues.len()
                        ));
                    }
                    rebuild_okf_index_document(&runtime, space_id)
                        .await
                        .map(|_| ())
                        .map_err(|error| format!("{error:?}"))
                },
            )
            .await?;
        Ok(OkfQualityRun {
            id: job.id,
            state: format!("{:?}", job.state).to_ascii_lowercase(),
        })
    }

    async fn create_okf_eval_run(
        &self,
        request: OkfQualityRunRequest,
    ) -> BackendApiResult<OkfQualityRun> {
        if request.space_id == 0 {
            return Err(BackendApiError::new(
                axum::http::StatusCode::BAD_REQUEST,
                "invalid_okf_quality_run_request",
                "space_id is required",
            ));
        }
        let space_id = request.space_id;
        let runtime = self.runtime.clone();
        let job = self
            .create_and_run_background_job(
                space_id,
                "okf_eval_run",
                format!(
                    "okf-eval:{}:{}",
                    request.space_id,
                    request.profile.as_deref().unwrap_or("default")
                ),
                || async move {
                    persist_okf_profile(&runtime, space_id)
                        .await
                        .map(|_| ())
                        .map_err(|error| format!("{error:?}"))
                },
            )
            .await?;
        Ok(OkfQualityRun {
            id: job.id,
            state: format!("{:?}", job.state).to_ascii_lowercase(),
        })
    }

    async fn create_index(
        &self,
        request: KnowledgeIndexRequest,
    ) -> BackendApiResult<KnowledgeIndex> {
        let index = self
            .runtime
            .index_store()
            .create_index(request)
            .await
            .map_err(|error| map_internal(error.to_string()))?;

        let space = self
            .runtime
            .space_store()
            .get_space(index.space_id)
            .await
            .map_err(|error| map_internal(error.to_string()))?;

        if space.knowledge_mode == KnowledgeAgentKnowledgeMode::Rag {
            if let Ok(client) = resolve_claw_router_client_from_env() {
                let embedder = ClawRouterEmbeddingClient::new(Arc::new(client));
                let build =
                    KnowledgeEmbeddingBuildService::new(self.runtime.embedding_store(), embedder);
                let _ = build
                    .embed_space_chunks(
                        self.runtime.tenant_id(),
                        index.index_id,
                        index.space_id,
                        None,
                        None,
                    )
                    .await;
            }
        }

        Ok(index)
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
        request: OkfIndexRebuildRequest,
    ) -> BackendApiResult<OkfIndexDocument> {
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

            return Ok(OkfIndexDocument {
                markdown: format!(
                    "# RAG embedding index rebuild\n\nIndexed {indexed} chunk embedding(s) for index {index_id} in space {space_id}."
                ),
            });
        }

        rebuild_okf_index_document(&self.runtime, space_id)
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
            provider_id: "sdkwork-knowledgebase-sqlx".to_string(),
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

fn map_okf_concept(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> BackendApiError {
    let detail = error.to_string();
    if detail.contains("missing okf concept") {
        BackendApiError::new(
            axum::http::StatusCode::NOT_FOUND,
            "okf_concept_not_found",
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
