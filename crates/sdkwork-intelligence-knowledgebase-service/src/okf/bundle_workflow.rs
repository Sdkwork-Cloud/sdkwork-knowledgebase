use crate::okf::index_rebuild::OkfIndexRebuildError;
use crate::okf::OkfBundleStandardFileService;
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::ports::knowledge_okf_bundle_file_store::KnowledgeOkfBundleFileStore;
use crate::ports::knowledge_okf_concept_link_store::KnowledgeOkfConceptLinkStore;
use crate::ports::knowledge_okf_concept_store::{
    AppendKnowledgeOkfLogEntryRecord, KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_source_store::{KnowledgeSourceStore, KnowledgeSourceStoreError};
use crate::ports::knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError};
use sdkwork_knowledgebase_contract::knowledge_engine::KnowledgeEngineError;
use sdkwork_knowledgebase_contract::okf::OkfBundleLintResult;
use sdkwork_knowledgebase_contract::OkfLogEventType;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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
    Engine(#[from] KnowledgeEngineError),
}

pub struct OkfBundleWorkflowDeps<'a> {
    pub concepts: &'a dyn KnowledgeOkfConceptStore,
    pub drive: &'a dyn KnowledgeDriveStorage,
    pub space_store: &'a dyn KnowledgeSpaceStore,
    pub source_store: &'a dyn KnowledgeSourceStore,
    pub link_store: Option<&'a dyn KnowledgeOkfConceptLinkStore>,
    pub bundle_file_store: Option<&'a dyn KnowledgeOkfBundleFileStore>,
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
    append_bundle_log_entry(
        deps.concepts,
        space_id,
        OkfLogEventType::Compile,
        format!("Compiled OKF bundle from {source_label}"),
        actor,
        Vec::new(),
    )
    .await?;

    rebuild_bundle_index(&deps, space_id).await?;
    refresh_standard_bundle_files(&deps, &space.name, space_id).await
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
    append_bundle_log_entry(
        deps.concepts,
        space_id,
        OkfLogEventType::Eval,
        format!("Evaluated OKF bundle quality ({})", lint_result.conformance),
        actor,
        warnings,
    )
    .await?;

    rebuild_bundle_index(&deps, space_id).await?;
    refresh_standard_bundle_files(&deps, &space.name, space_id).await?;

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

async fn append_bundle_log_entry(
    concept_store: &dyn KnowledgeOkfConceptStore,
    space_id: u64,
    event_type: OkfLogEventType,
    title: String,
    actor: &str,
    warnings: Vec<String>,
) -> Result<(), KnowledgeOkfConceptStoreError> {
    concept_store
        .append_log_entry(AppendKnowledgeOkfLogEntryRecord {
            space_id,
            event_type: event_type.as_str().to_string(),
            event_time: current_rfc3339_timestamp(),
            title,
            actor: actor.to_string(),
            affected_concepts: Vec::new(),
            audit_event_id: None,
            warnings,
            privacy_level: "internal".to_string(),
        })
        .await?;
    Ok(())
}

async fn refresh_standard_bundle_files(
    deps: &OkfBundleWorkflowDeps<'_>,
    space_name: &str,
    space_id: u64,
) -> Result<(), OkfBundleWorkflowError> {
    let concepts = deps.concepts.list_concept_summaries(space_id).await?;
    let log_entries = deps.concepts.list_log_entries(space_id).await?;
    let files = OkfBundleStandardFileService::new(deps.drive)
        .persist_standard_files(crate::okf::PersistStandardFilesRequest {
            space_name: space_name.to_string(),
            concepts,
            log_entries,
        })
        .await?;

    if let Some(bundle_file_store) = deps.bundle_file_store {
        crate::okf::OkfBundleFileRegistryService::new(bundle_file_store)
            .register_standard_files(space_id, &files)
            .await?;
    }

    Ok(())
}

fn current_rfc3339_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
