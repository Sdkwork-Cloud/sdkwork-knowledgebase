use crate::okf::index_renderer::render_index_documents;
use crate::okf::log_renderer::render_log_md;
use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_contract::okf::{OkfBundlePaths, OkfConceptSummary, OkfLogEntry};

#[derive(Debug, Clone)]
pub(crate) struct DynamicStandardBundleFiles {
    pub index_object_refs: Vec<KnowledgeObjectRef>,
    pub root_index_md: KnowledgeObjectRef,
    pub log_md: KnowledgeObjectRef,
}

pub(crate) async fn persist_dynamic_standard_bundle_files(
    drive: &dyn KnowledgeDriveStorage,
    summaries: &[OkfConceptSummary],
    log_entries: &[OkfLogEntry],
    drive_space_id: Option<&str>,
) -> Result<DynamicStandardBundleFiles, KnowledgeStorageError> {
    let paths = OkfBundlePaths::default();
    let index_documents = render_index_documents(summaries);
    let mut index_object_refs = Vec::with_capacity(index_documents.len());
    let mut root_index_md = None;

    for (bundle_relative_path, markdown) in index_documents {
        let logical_path = if bundle_relative_path == "index.md" {
            paths.index_md.to_string()
        } else {
            format!("okf/{bundle_relative_path}")
        };
        let object_ref = drive
            .put_object(
                PutKnowledgeObjectRequest::text(
                    logical_path.clone(),
                    "bundle_index",
                    markdown,
                    None,
                )
                .with_drive_space_id(drive_space_id),
            )
            .await?;
        if logical_path == paths.index_md {
            root_index_md = Some(object_ref.clone());
        }
        index_object_refs.push(object_ref);
    }

    let root_index_md = root_index_md.ok_or_else(|| {
        KnowledgeStorageError::internal(
            "dynamic standard bundle refresh did not produce root index.md",
        )
    })?;

    let log_md = drive
        .put_object(
            PutKnowledgeObjectRequest::text(
                paths.log_md,
                "bundle_log",
                render_log_md(log_entries),
                None,
            )
            .with_drive_space_id(drive_space_id),
        )
        .await?;

    Ok(DynamicStandardBundleFiles {
        index_object_refs,
        root_index_md,
        log_md,
    })
}
