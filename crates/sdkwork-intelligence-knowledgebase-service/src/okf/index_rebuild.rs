use crate::okf::{render_index_documents, render_log_md};
use crate::ports::knowledge_drive_storage::{KnowledgeDriveStorage, PutKnowledgeObjectRequest};
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError};
use sdkwork_knowledgebase_contract::okf::OkfBundlePaths;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OkfIndexRebuildError {
    #[error(transparent)]
    SpaceStore(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    ConceptStore(#[from] KnowledgeOkfConceptStoreError),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
}

pub async fn rebuild_bundle_index_for_space(
    drive: &dyn KnowledgeDriveStorage,
    concept_store: &dyn KnowledgeOkfConceptStore,
    space_store: &dyn KnowledgeSpaceStore,
    space_id: u64,
) -> Result<(), OkfIndexRebuildError> {
    space_store.get_space(space_id).await?;
    let concepts = concept_store.list_concept_summaries(space_id).await?;
    let logs = concept_store.list_log_entries(space_id).await?;
    let index_documents = render_index_documents(&concepts);
    let log_markdown = render_log_md(&logs);
    let paths = OkfBundlePaths::default();

    for (bundle_relative_path, markdown) in index_documents {
        let logical_path = if bundle_relative_path == "index.md" {
            paths.index_md.to_string()
        } else {
            format!("okf/{bundle_relative_path}")
        };
        drive
            .put_object(PutKnowledgeObjectRequest::text(
                logical_path,
                "bundle_index",
                markdown,
                None,
            ))
            .await?;
    }
    drive
        .put_object(PutKnowledgeObjectRequest::text(
            paths.log_md,
            "bundle_log",
            log_markdown,
            None,
        ))
        .await?;

    Ok(())
}
