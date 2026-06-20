use sdkwork_intelligence_knowledgebase_service::{
    ingest::KnowledgeIngestionService,
    okf::{
        load_import_bundle_from_drive, render_index_md, render_log_md, to_contract_lint_result,
        ExportOkfBundleRequest, ImportOkfBundleRequest, OkfBundleExporterService,
        OkfBundleFileRegistryService, OkfBundleImporterService, OkfBundleLinterService,
        OkfBundleStandardFileService, OkfConceptService, PersistStandardFilesRequest,
        PublishExistingOkfConceptRevisionRequest,
    },
    ports::{
        knowledge_drive_storage::{
            HeadKnowledgeObjectRequest, KnowledgeDriveStorage, PutKnowledgeObjectRequest,
        },
        knowledge_ingestion_job_store::{CreateIngestionJobRecord, IngestionJobStore},
        knowledge_okf_bundle_file_store::{
            CreateKnowledgeOkfBundleFileRecord, KnowledgeOkfBundleFileStore,
        },
        knowledge_okf_concept_store::KnowledgeOkfConceptStore,
        knowledge_space_store::KnowledgeSpaceStore,
    },
};
use sdkwork_knowledgebase_contract::ingest::IngestionJobState;
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult,
    OkfBundleLintResult, OkfBundlePaths, OkfConceptSummary, OkfIndexDocument, OkfQualityRun,
    OkfQualityRunRequest,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeContextFragment, KnowledgeRetrievalBinding, KnowledgeRetrievalMethod,
};
use sdkwork_knowledgebase_contract::{IngestionJob, KnowledgeOkfBundleFile, OkfBundleFileKind};

use crate::ApiError;

pub(crate) fn concept_to_summary(concept: KnowledgeOkfConcept) -> OkfConceptSummary {
    OkfConceptSummary {
        title: concept.title,
        concept_id: concept.concept_id,
        concept_type: concept.concept_type,
        logical_path: concept.logical_path,
        bundle_relative_path: concept.bundle_relative_path,
        description: concept.description,
        source_count: concept.source_count,
        updated_at: concept.updated_at,
        tags: concept.tags,
    }
}

pub(crate) fn space_binding(space_id: u64) -> KnowledgeRetrievalBinding {
    KnowledgeRetrievalBinding {
        space_id,
        collection_id: None,
        source_filter: None,
        document_filter: None,
        priority: 10,
        top_k: None,
        min_score: None,
    }
}

pub(crate) fn default_retrieval_methods() -> Vec<KnowledgeRetrievalMethod> {
    vec![KnowledgeRetrievalMethod::Hybrid]
}

pub(crate) fn format_retrieval_answer(hits: &[KnowledgeContextFragment]) -> String {
    if hits.is_empty() {
        return "_No matching knowledge fragments were found._".to_string();
    }

    hits.iter()
        .map(|hit| format!("### {}\n\n{}", hit.title, hit.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn okf_answer_concept_id(query_id: u64) -> String {
    format!("answers/answer-{query_id}")
}

pub(crate) async fn read_managed_okf_text(
    drive: &dyn KnowledgeDriveStorage,
    logical_path: &str,
    object_role: &str,
) -> Result<String, ApiError> {
    let object_ref = drive
        .head_object(HeadKnowledgeObjectRequest::managed_artifact(
            logical_path,
            object_role,
        ))
        .await?;
    drive.get_object_text(&object_ref).await.map_err(Into::into)
}

pub(crate) fn okf_concept_service(
    runtime: &crate::runtime::KnowledgebaseRuntime,
) -> OkfConceptService<'_> {
    OkfConceptService::new(
        runtime.drive_storage(),
        runtime.object_ref_store(),
        runtime.okf_concept_store(),
    )
    .with_link_store(runtime.okf_concept_link_store())
    .with_candidate_store(runtime.okf_candidate_store())
    .with_file_entry_store(runtime.okf_bundle_file_store())
    .with_drive_workspace(runtime.drive_workspace())
}

pub(crate) async fn publish_okf_concept_revision(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    concept_row_id: u64,
    actor: &str,
) -> Result<KnowledgeOkfConcept, ApiError> {
    let concept = runtime
        .okf_concept_store()
        .get_concept_by_row_id(concept_row_id)
        .await
        .map_err(map_okf_concept_store)?;
    let revision_id = concept.current_revision_id.ok_or_else(|| {
        ApiError::conflict(
            "okf_concept_not_ready",
            format!("okf concept {concept_row_id} has no current revision to publish"),
        )
    })?;
    let revision = runtime
        .okf_concept_store()
        .get_revision_by_id(revision_id)
        .await
        .map_err(map_okf_concept_store)?;
    let space = runtime.space_store().get_space(concept.space_id).await?;
    let publication = okf_concept_service(runtime)
        .publish_existing_revision(
            PublishExistingOkfConceptRevisionRequest {
                space_id: concept.space_id,
                concept: concept.clone(),
                revision,
                actor: actor.to_string(),
            },
            space.drive_space_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(publication.concept)
}

pub(crate) async fn import_okf_bundle(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    space_id: u64,
    actor: &str,
    publish: bool,
    files: Vec<sdkwork_intelligence_knowledgebase_service::okf::ImportOkfBundleFile>,
) -> Result<sdkwork_knowledgebase_contract::okf::OkfBundleImportResult, ApiError> {
    let space = runtime.space_store().get_space(space_id).await?;
    let result = OkfBundleImporterService::new(okf_concept_service(runtime))
        .import_bundle(
            ImportOkfBundleRequest {
                space_id,
                actor: actor.to_string(),
                publish,
                files,
            },
            space.drive_space_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(sdkwork_knowledgebase_contract::okf::OkfBundleImportResult {
        imported_concept_count: result.imported_concept_count,
        skipped_files: result.skipped_files,
    })
}

pub(crate) fn okf_paths() -> OkfBundlePaths {
    OkfBundlePaths::default()
}

pub(crate) fn okf_bundle_not_initialized_detail() -> String {
    "no okf-bundle-initialized knowledge space is available for this tenant".to_string()
}

pub(crate) async fn run_okf_bundle_lint(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    space_id: u64,
) -> Result<OkfBundleLintResult, ApiError> {
    let report = OkfBundleLinterService::new(runtime.drive_storage(), runtime.okf_concept_store())
        .with_link_store(runtime.okf_concept_link_store())
        .lint_space(space_id)
        .await
        .map_err(|error| ApiError::internal("okf_bundle_lint_failed", error.to_string()))?;
    Ok(to_contract_lint_result(&report))
}

pub(crate) async fn rebuild_okf_index_document(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    space_id: u64,
) -> Result<OkfIndexDocument, ApiError> {
    let space = runtime.space_store().get_space(space_id).await?;
    let concepts = runtime
        .okf_concept_store()
        .list_concept_summaries(space_id)
        .await
        .map_err(map_okf_concept_store)?;
    let logs = runtime
        .okf_concept_store()
        .list_log_entries(space_id)
        .await
        .map_err(map_okf_concept_store)?;
    let markdown = render_index_md(&space.name, &concepts);
    let log_markdown = render_log_md(&logs);
    let paths = okf_paths();
    runtime
        .drive_storage()
        .put_object(PutKnowledgeObjectRequest::text(
            paths.index_md,
            "bundle_index",
            markdown.clone(),
            None,
        ))
        .await?;
    runtime
        .drive_storage()
        .put_object(PutKnowledgeObjectRequest::text(
            paths.log_md,
            "bundle_log",
            log_markdown,
            None,
        ))
        .await?;
    Ok(OkfIndexDocument { markdown })
}

pub(crate) async fn persist_okf_profile(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    space_id: u64,
) -> Result<sdkwork_knowledgebase_contract::KnowledgeOkfBundleFile, ApiError> {
    let space = runtime.space_store().get_space(space_id).await?;
    let concepts = runtime
        .okf_concept_store()
        .list_concept_summaries(space_id)
        .await
        .map_err(map_okf_concept_store)?;
    let logs = runtime
        .okf_concept_store()
        .list_log_entries(space_id)
        .await
        .map_err(map_okf_concept_store)?;
    let files = OkfBundleStandardFileService::new(runtime.drive_storage())
        .persist_standard_files(PersistStandardFilesRequest {
            space_name: space.name,
            concepts: concepts,
            log_entries: logs,
        })
        .await?;
    let registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let entries = registry
        .register_standard_files(space_id, &files)
        .await
        .map_err(|error| {
            ApiError::internal("okf_bundle_file_registry_failed", error.to_string())
        })?;
    entries
        .into_iter()
        .find(|entry| entry.logical_path.ends_with("okf_profile.yaml"))
        .ok_or_else(|| {
            ApiError::internal(
                "okf_profile_missing",
                "profile registration did not produce okf_profile.yaml entry",
            )
        })
}

pub(crate) async fn create_okf_bundle_export(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    request: OkfBundleExportRequest,
) -> Result<KnowledgeOkfBundleFile, ApiError> {
    if request.space_id == 0 || request.export_type.trim().is_empty() {
        return Err(ApiError::invalid_request(
            "invalid_okf_export_request",
            "space_id and export_type are required",
        ));
    }
    let source_object_refs = if request.export_type.trim() == "okf_with_sources" {
        runtime
            .object_ref_store()
            .list_object_refs_by_logical_path_prefix(request.space_id, "sources/raw/")
            .await
            .map_err(|error| ApiError::internal("okf_export_failed", error.to_string()))?
    } else {
        Vec::new()
    };
    let exported =
        OkfBundleExporterService::new(runtime.drive_storage(), runtime.okf_concept_store())
            .with_source_object_refs(source_object_refs)
            .export_bundle(ExportOkfBundleRequest {
                space_id: request.space_id,
                export_type: request.export_type,
            })
            .await
            .map_err(|error| ApiError::internal("okf_export_failed", error.to_string()))?;
    runtime
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
        .map_err(|error| ApiError::internal("okf_export_failed", error.to_string()))
}

pub(crate) async fn retrieve_okf_bundle_export(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    export_id: u64,
) -> Result<KnowledgeOkfBundleFile, ApiError> {
    runtime
        .okf_bundle_file_store()
        .get_file_entry_by_id(export_id)
        .await
        .map_err(|error| {
            let detail = error.to_string();
            if detail.contains("missing okf bundle file") {
                ApiError::not_found("okf_export_not_found", detail)
            } else {
                ApiError::internal("okf_export_retrieve_failed", detail)
            }
        })
}

pub(crate) async fn create_okf_bundle_import(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    request: OkfBundleImportRequest,
    actor: &str,
) -> Result<OkfBundleImportResult, ApiError> {
    if request.space_id == 0 || request.import_type.trim().is_empty() {
        return Err(ApiError::invalid_request(
            "invalid_okf_import_request",
            "space_id and import_type are required",
        ));
    }
    let import_type = request.import_type.trim();
    if import_type != "okf_strict" && import_type != "okf_bundle" {
        return Err(ApiError::invalid_request(
            "invalid_okf_import_request",
            format!("unsupported import_type: {import_type}"),
        ));
    }
    let publish = import_type == "okf_strict";
    let space_id = request.space_id;
    let files = load_import_bundle_from_drive(runtime.drive_storage(), space_id)
        .await
        .map_err(|error| ApiError::internal("okf_import_failed", error.to_string()))?;
    import_okf_bundle(runtime, space_id, actor, publish, files).await
}

pub(crate) async fn create_okf_lint_run(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    request: OkfQualityRunRequest,
) -> Result<OkfQualityRun, ApiError> {
    if request.space_id == 0 {
        return Err(ApiError::invalid_request(
            "invalid_okf_quality_run_request",
            "space_id is required",
        ));
    }
    let space_id = request.space_id;
    let runtime_for_job = runtime.clone();
    let runtime_in_closure = runtime.clone();
    let job = run_okf_ingestion_job(
        &runtime_for_job,
        space_id,
        "okf_lint_run",
        format!(
            "okf-lint:{}:{}",
            request.space_id,
            request.profile.as_deref().unwrap_or("default")
        ),
        || async move {
            let lint_result = run_okf_bundle_lint(&runtime_in_closure, space_id)
                .await
                .map_err(|error| format!("{error:?}"))?;
            let report_path = format!("output/lint-reports/{space_id}.json");
            runtime_in_closure
                .drive_storage()
                .put_object(PutKnowledgeObjectRequest {
                    logical_path: report_path,
                    object_role: "output_export".to_string(),
                    content_type: "application/json; charset=utf-8".to_string(),
                    body: serde_json::to_vec_pretty(&lint_result)
                        .map_err(|error| format!("failed to serialize lint report: {error}"))?,
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
            rebuild_okf_index_document(&runtime_in_closure, space_id)
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

async fn run_okf_ingestion_job<F, Fut>(
    runtime: &crate::runtime::KnowledgebaseRuntime,
    space_id: u64,
    source_type: &str,
    idempotency_key: String,
    run: F,
) -> Result<IngestionJob, ApiError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let result = runtime
        .ingestion_job_store()
        .create_or_get_job(CreateIngestionJobRecord {
            space_id,
            source_type: source_type.to_string(),
            idempotency_key,
            idempotency_fingerprint_sha256_hex: None,
        })
        .await
        .map_err(|error| ApiError::internal("okf_ingestion_job_failed", error.to_string()))?;

    let mut job = result.job;
    if job.state != IngestionJobState::Queued {
        return Ok(job);
    }

    let ingestion = KnowledgeIngestionService::new(runtime.ingestion_job_store());
    job = ingestion
        .mark_running(job.id)
        .await
        .map_err(|error| ApiError::internal("okf_ingestion_job_failed", error.to_string()))?;
    match run().await {
        Ok(()) => ingestion
            .mark_succeeded(job.id)
            .await
            .map_err(|error| ApiError::internal("okf_ingestion_job_failed", error.to_string())),
        Err(detail) => ingestion
            .mark_failed(job.id, detail)
            .await
            .map_err(|error| ApiError::internal("okf_ingestion_job_failed", error.to_string())),
    }
}

fn map_okf_concept_store(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> ApiError {
    ApiError::internal("knowledge_okf_concept_store_failed", error.to_string())
}
