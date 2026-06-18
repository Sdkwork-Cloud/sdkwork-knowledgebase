use sdkwork_intelligence_knowledgebase_service::{
    ports::{
        knowledge_drive_storage::{
            HeadKnowledgeObjectRequest, KnowledgeDriveStorage, PutKnowledgeObjectRequest,
        },
        knowledge_space_store::KnowledgeSpaceStore,
        knowledge_wiki_page_store::KnowledgeWikiPageStore,
    },
    wiki::{
        render_index_md, render_log_md, KnowledgeWikiFileRegistryService,
        LlmWikiStandardFileService, PersistStandardFilesRequest,
    },
};
use sdkwork_knowledgebase_contract::{
    rag::{KnowledgeContextFragment, KnowledgeRetrievalBinding, KnowledgeRetrievalMethod},
    wiki::{KnowledgeWikiPage, LlmWikiPaths, WikiIndexDocument, WikiPageSummary},
};

use crate::ApiError;

pub(crate) fn page_to_summary(page: KnowledgeWikiPage) -> WikiPageSummary {
    WikiPageSummary {
        title: page.title,
        slug: page.slug,
        page_type: page.page_type,
        logical_path: page.logical_path,
        summary: page.summary,
        source_count: page.source_count,
        updated_at: page.updated_at,
        tags: page.tags,
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

pub(crate) fn wiki_answer_slug(query_id: u64) -> String {
    format!("answer-{query_id}")
}

pub(crate) async fn read_managed_wiki_text(
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

pub(crate) fn wiki_paths() -> LlmWikiPaths {
    LlmWikiPaths::default()
}

pub(crate) fn wiki_not_initialized_detail() -> String {
    "no wiki-initialized knowledge space is available for this tenant".to_string()
}

pub(crate) async fn rebuild_wiki_index_document(
    runtime: &crate::runtime::KnowledgebaseSqliteRuntime,
    space_id: u64,
) -> Result<WikiIndexDocument, ApiError> {
    let space = runtime.space_store().get_space(space_id).await?;
    let pages = runtime
        .wiki_page_store()
        .list_page_summaries(space_id)
        .await
        .map_err(map_wiki_page_store)?;
    let logs = runtime
        .wiki_page_store()
        .list_log_entries(space_id)
        .await
        .map_err(map_wiki_page_store)?;
    let markdown = render_index_md(&space.name, &pages);
    let log_markdown = render_log_md(&logs);
    let paths = wiki_paths();
    runtime
        .drive_storage()
        .put_object(PutKnowledgeObjectRequest::text(
            paths.index_md,
            "wiki_index",
            markdown.clone(),
            None,
        ))
        .await?;
    runtime
        .drive_storage()
        .put_object(PutKnowledgeObjectRequest::text(
            paths.log_md,
            "wiki_log",
            log_markdown,
            None,
        ))
        .await?;
    Ok(WikiIndexDocument { markdown })
}

pub(crate) async fn persist_wiki_schema_profile(
    runtime: &crate::runtime::KnowledgebaseSqliteRuntime,
    space_id: u64,
) -> Result<sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry, ApiError> {
    let space = runtime.space_store().get_space(space_id).await?;
    let pages = runtime
        .wiki_page_store()
        .list_page_summaries(space_id)
        .await
        .map_err(map_wiki_page_store)?;
    let logs = runtime
        .wiki_page_store()
        .list_log_entries(space_id)
        .await
        .map_err(map_wiki_page_store)?;
    let files = LlmWikiStandardFileService::new(runtime.drive_storage())
        .persist_standard_files(PersistStandardFilesRequest {
            space_name: space.name,
            pages,
            log_entries: logs,
        })
        .await?;
    let registry = KnowledgeWikiFileRegistryService::new(runtime.wiki_file_entry_store());
    let entries = registry
        .register_standard_files(space_id, &files)
        .await
        .map_err(|error| ApiError::internal("wiki_file_registry_failed", error.to_string()))?;
    entries
        .into_iter()
        .find(|entry| entry.logical_path.ends_with("wiki_schema.yaml"))
        .ok_or_else(|| {
            ApiError::internal(
                "wiki_schema_profile_missing",
                "schema profile registration did not produce wiki_schema.yaml entry",
            )
        })
}

fn map_wiki_page_store(
    error: sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::KnowledgeWikiPageStoreError,
) -> ApiError {
    ApiError::internal("knowledge_wiki_page_store_failed", error.to_string())
}
