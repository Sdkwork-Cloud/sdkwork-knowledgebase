use crate::okf::catalog_log;
use crate::okf::index_rebuild::OkfIndexRebuildError;
use crate::okf::standard_bundle_catalog_sync::{
    sync_full_standard_bundle_catalog, StandardBundleCatalogSyncDeps,
    StandardBundleCatalogSyncError,
};
use crate::okf::OkfBundleStandardFileService;
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::ports::knowledge_drive_workspace::KnowledgeDriveWorkspace;
use crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStore;
use crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStore;
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_source_store::{KnowledgeSourceStore, KnowledgeSourceStoreError};
use crate::ports::knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError;
use sdkwork_knowledgebase_contract::okf::OkfBundleLintResult;
use sdkwork_knowledgebase_contract::OkfLogEventType;
use thiserror::Error;

#[async_trait::async_trait]
pub trait OkfBundleWorkflowEngine: Send + Sync {
    async fn rebuild_index(&self, space_id: u64) -> Result<(), KnowledgeEngineError>;

    async fn lint_bundle_report(
        &self,
        space_id: u64,
    ) -> Result<OkfBundleLintResult, KnowledgeEngineError>;
}

#[derive(Debug, Error)]
pub enum OkfBundleWorkflowError {
    #[error("invalid okf bundle workflow request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    SpaceStore(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    ConceptStore(#[from] KnowledgeOkfConceptStoreError),
    #[error(transparent)]
    SourceStore(#[from] KnowledgeSourceStoreError),
    #[error(transparent)]
    IndexRebuild(#[from] OkfIndexRebuildError),
    #[error(transparent)]
    Linter(#[from] crate::okf::OkfBundleLinterError),
    #[error(transparent)]
    Storage(#[from] crate::ports::knowledge_drive_storage::KnowledgeStorageError),
    #[error(transparent)]
    BundleFileStore(
        #[from] crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStoreError,
    ),
    #[error(transparent)]
    BundleFileRegistry(#[from] crate::okf::OkfBundleFileRegistryServiceError),
    #[error(transparent)]
    CatalogSync(#[from] StandardBundleCatalogSyncError),
    #[error(transparent)]
    Engine(#[from] KnowledgeEngineError),
}

pub struct OkfBundleWorkflowDeps<'a> {
    pub concepts: &'a dyn KnowledgeOkfConceptStore,
    pub drive: &'a dyn KnowledgeDriveStorage,
    pub space_store: &'a dyn KnowledgeSpaceStore,
    pub source_store: &'a dyn KnowledgeSourceStore,
    pub link_store: Option<&'a dyn KnowledgeOkfConceptLinkStore>,
    pub bundle_file_store: Option<&'a dyn KnowledgeOkfBundleFileStore>,
    pub drive_workspace: Option<&'a dyn KnowledgeDriveWorkspace>,
    pub engine: Option<&'a dyn OkfBundleWorkflowEngine>,
}

pub async fn run_okf_compile_workflow(
    deps: OkfBundleWorkflowDeps<'_>,
    space_id: u64,
    source_id: Option<u64>,
    actor: &str,
) -> Result<(), OkfBundleWorkflowError> {
    if space_id == 0 {
        return Err(OkfBundleWorkflowError::InvalidRequest(
            "space_id is required".to_string(),
        ));
    }

    let space = deps.space_store.get_space(space_id).await?;
    validate_compile_source(deps.source_store, space_id, source_id).await?;

    let source_label = match source_id {
        Some(id) => format!("source {id}"),
        None => "bundle".to_string(),
    };
    catalog_log::append_okf_bundle_log_entry(
        deps.concepts,
        space_id,
        OkfLogEventType::Compile.as_str(),
        format!("Compiled OKF bundle from {source_label}"),
        actor,
        Vec::new(),
        Vec::new(),
    )
    .await?;

    rebuild_bundle_index(&deps, space_id).await?;
    refresh_standard_bundle_files(&deps, &space.name, space_id, space.drive_space_id.clone()).await
}

pub async fn run_okf_eval_workflow(
    deps: OkfBundleWorkflowDeps<'_>,
    space_id: u64,
    actor: &str,
) -> Result<OkfBundleLintResult, OkfBundleWorkflowError> {
    if space_id == 0 {
        return Err(OkfBundleWorkflowError::InvalidRequest(
            "space_id is required".to_string(),
        ));
    }

    let space = deps.space_store.get_space(space_id).await?;
    let lint_result = lint_bundle_report(&deps, space_id).await?;

    let warnings = if lint_result.conformance == "pass" {
        Vec::new()
    } else {
        vec![format!(
            "eval found {} lint issue(s)",
            lint_result.issues.len()
        )]
    };
    catalog_log::append_okf_bundle_log_entry(
        deps.concepts,
        space_id,
        OkfLogEventType::Eval.as_str(),
        format!("Evaluated OKF bundle quality ({})", lint_result.conformance),
        actor,
        Vec::new(),
        warnings,
    )
    .await?;

    rebuild_bundle_index(&deps, space_id).await?;
    refresh_standard_bundle_files(&deps, &space.name, space_id, space.drive_space_id.clone())
        .await?;

    Ok(lint_result)
}

async fn rebuild_bundle_index(
    deps: &OkfBundleWorkflowDeps<'_>,
    space_id: u64,
) -> Result<(), OkfBundleWorkflowError> {
    let Some(engine) = deps.engine else {
        return Err(OkfBundleWorkflowError::InvalidRequest(
            "okf bundle workflow requires knowledge engine SPI wiring".to_string(),
        ));
    };
    engine.rebuild_index(space_id).await?;
    Ok(())
}

async fn lint_bundle_report(
    deps: &OkfBundleWorkflowDeps<'_>,
    space_id: u64,
) -> Result<OkfBundleLintResult, OkfBundleWorkflowError> {
    let Some(engine) = deps.engine else {
        return Err(OkfBundleWorkflowError::InvalidRequest(
            "okf bundle workflow requires knowledge engine SPI wiring".to_string(),
        ));
    };
    engine
        .lint_bundle_report(space_id)
        .await
        .map_err(Into::into)
}

async fn validate_compile_source(
    source_store: &dyn KnowledgeSourceStore,
    space_id: u64,
    source_id: Option<u64>,
) -> Result<(), OkfBundleWorkflowError> {
    let Some(source_id) = source_id.filter(|value| *value > 0) else {
        return Ok(());
    };

    let sources = source_store.list_sources_for_space(space_id).await?;
    if sources.iter().any(|source| source.id == source_id) {
        return Ok(());
    }

    Err(OkfBundleWorkflowError::InvalidRequest(format!(
        "source_id {source_id} is not registered for space_id {space_id}"
    )))
}

async fn refresh_standard_bundle_files(
    deps: &OkfBundleWorkflowDeps<'_>,
    space_name: &str,
    space_id: u64,
    drive_space_id: Option<String>,
) -> Result<(), OkfBundleWorkflowError> {
    let files = OkfBundleStandardFileService::new(deps.drive)
        .persist_standard_files_after_index_rebuild(crate::okf::PersistStandardFilesRequest {
            space_name: space_name.to_string(),
            concepts: vec![],
            log_entries: vec![],
            drive_space_id: drive_space_id.clone(),
        })
        .await?;

    sync_full_standard_bundle_catalog(
        StandardBundleCatalogSyncDeps {
            bundle_file_store: deps.bundle_file_store,
            drive_workspace: deps.drive_workspace,
        },
        space_id,
        &files,
        drive_space_id.as_deref(),
    )
    .await?;

    Ok(())
}
