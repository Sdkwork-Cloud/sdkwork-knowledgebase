use super::standard_bundle_refresh;
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError};
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
    standard_bundle_refresh::persist_dynamic_standard_bundle_files(
        drive,
        &concepts,
        &logs,
        space.drive_space_id.as_deref(),
    )
    .await?;
    Ok(())
}
