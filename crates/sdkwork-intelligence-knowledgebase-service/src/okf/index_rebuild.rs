use crate::okf::{render_index_md, render_log_md};
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
    let space = space_store.get_space(space_id).await?;
    let concepts = concept_store.list_concept_summaries(space_id).await?;
    let logs = concept_store.list_log_entries(space_id).await?;
    let markdown = render_index_md(&space.name, &concepts);
    let log_markdown = render_log_md(&logs);
    let paths = OkfBundlePaths::default();

    drive
        .put_object(PutKnowledgeObjectRequest::text(
            paths.index_md,
            "bundle_index",
            markdown,
            None,
        ))
        .await?;
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
