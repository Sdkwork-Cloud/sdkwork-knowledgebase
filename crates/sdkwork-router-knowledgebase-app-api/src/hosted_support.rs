use sdkwork_intelligence_knowledgebase_service::{
    okf::{
        render_index_md, render_log_md, OkfBundleFileRegistryService, OkfBundleStandardFileService,
        PersistStandardFilesRequest,
    },
    ports::{
        knowledge_drive_storage::{
            HeadKnowledgeObjectRequest, KnowledgeDriveStorage, PutKnowledgeObjectRequest,
        },
        knowledge_okf_concept_store::KnowledgeOkfConceptStore,
        knowledge_space_store::KnowledgeSpaceStore,
    },
};
use sdkwork_knowledgebase_contract::okf::{
    KnowledgeOkfConcept, OkfBundlePaths, OkfConceptSummary, OkfIndexDocument,
};
use sdkwork_knowledgebase_contract::rag::{
    KnowledgeContextFragment, KnowledgeRetrievalBinding, KnowledgeRetrievalMethod,
};

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

pub(crate) fn okf_paths() -> OkfBundlePaths {
    OkfBundlePaths::default()
}

pub(crate) fn okf_bundle_not_initialized_detail() -> String {
    "no okf-bundle-initialized knowledge space is available for this tenant".to_string()
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

fn map_okf_concept_store(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError,
) -> ApiError {
    ApiError::internal("knowledge_okf_concept_store_failed", error.to_string())
}
