mod bundle_linter;
mod bundle_workflow;
mod catalog_log;
mod concept_service;
mod document;
mod exporter;
mod file_registry;
mod governance_drive;
mod importer;
mod index_rebuild;
mod index_renderer;
mod initializer;
mod link_indexer;
mod linter;
mod log_renderer;
mod schema_renderer;
mod standard_bundle_catalog_sync;
mod standard_bundle_refresh;
mod storage;
mod validator;

use crate::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_knowledgebase_contract::okf::{OkfBundlePaths, OkfConceptSummary, OkfLogEntry};

pub use bundle_linter::{to_contract_lint_result, OkfBundleLinterError, OkfBundleLinterService};
pub use bundle_workflow::{
    run_okf_compile_workflow, run_okf_eval_workflow, OkfBundleWorkflowDeps,
    OkfBundleWorkflowEngine, OkfBundleWorkflowError,
};
pub use concept_service::{
    OkfConceptService, OkfConceptServiceError, PublishExistingOkfConceptRevisionRequest,
};
pub use document::{
    extract_concept_links, parse_okf_markdown, render_okf_concept_markdown,
    strip_sdkwork_frontmatter, OkfConceptDocument, OkfConceptLink, OkfDocumentError, OKF_VERSION,
    SDKWORK_FRONTMATTER_KEY,
};
pub use exporter::{
    ExportOkfBundleRequest, ExportedOkfBundle, OkfBundleExporterError, OkfBundleExporterService,
};
pub use file_registry::{OkfBundleFileRegistryService, OkfBundleFileRegistryServiceError};
pub(crate) use governance_drive::DRIVE_WORKSPACE_INIT_DRIVE_SPACE_REQUIRED;
pub use importer::{
    bundle_relative_path_from_logical_path, concept_id_from_bundle_relative_path,
    drive_import_root, load_import_bundle_from_drive, stage_export_bundle_for_drive_import,
    ImportOkfBundleFile, ImportOkfBundleRequest, ImportOkfBundleResult, OkfBundleImporterError,
    OkfBundleImporterService,
};
pub use index_rebuild::{rebuild_bundle_index_for_space, OkfIndexRebuildError};
pub use index_renderer::{render_index_documents, render_index_md};
pub use initializer::{OkfBundleInitializerService, OkfBundleInitializerServiceError};
pub use link_indexer::index_concept_links;
pub use linter::{
    extract_citation_urls, extract_index_linked_concept_ids, lint_bundle_summaries,
    lint_concept_stale_claims, lint_published_concept_markdown,
    lint_stale_claims_against_source_lineage, OkfBundleLintReport, OkfLintIssue, OkfLintSeverity,
};
pub use log_renderer::render_log_md;
pub use schema_renderer::{render_agents_md, render_okf_profile_yaml};
pub use standard_bundle_catalog_sync::StandardBundleCatalogSyncError;
pub use storage::{read_managed_markdown, read_managed_object_bytes};
pub use validator::{
    canonicalize_imported_concept_id, validate_bundle_relative_path,
    validate_catalog_concept_bundle_relative_path, validate_concept_bundle_relative_path,
    validate_concept_document, validate_concept_id, OkfConformanceError,
};

#[derive(Debug, Clone)]
pub struct PersistStandardFilesRequest {
    pub space_name: String,
    pub concepts: Vec<OkfConceptSummary>,
    pub log_entries: Vec<OkfLogEntry>,
    pub drive_space_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PersistedStandardFiles {
    pub agents_md: KnowledgeObjectRef,
    pub profile_yaml: KnowledgeObjectRef,
    pub index_md: KnowledgeObjectRef,
    pub log_md: KnowledgeObjectRef,
}

pub struct OkfBundleStandardFileService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
}

impl<'a> OkfBundleStandardFileService<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage) -> Self {
        Self { drive }
    }

    pub async fn persist_standard_files(
        &self,
        request: PersistStandardFilesRequest,
    ) -> Result<PersistedStandardFiles, KnowledgeStorageError> {
        let drive_space_id = request.drive_space_id.as_deref();
        let paths = OkfBundlePaths::default();
        let agents_md = self
            .drive
            .put_object(
                PutKnowledgeObjectRequest::text(
                    paths.agents_md,
                    "bundle_profile",
                    render_agents_md(&request.space_name),
                    None,
                )
                .with_drive_space_id(drive_space_id),
            )
            .await?;
        let profile_yaml = self
            .drive
            .put_object(
                PutKnowledgeObjectRequest::text(
                    paths.profile_yaml,
                    "bundle_profile",
                    render_okf_profile_yaml(),
                    None,
                )
                .with_drive_space_id(drive_space_id),
            )
            .await?;
        let dynamic = standard_bundle_refresh::persist_dynamic_standard_bundle_files(
            self.drive,
            &request.concepts,
            &request.log_entries,
            drive_space_id,
        )
        .await?;

        Ok(PersistedStandardFiles {
            agents_md,
            profile_yaml,
            index_md: dynamic.root_index_md,
            log_md: dynamic.log_md,
        })
    }

    /// Persists schema/profile files and resolves index/log refs after a prior index rebuild step.
    pub async fn persist_standard_files_after_index_rebuild(
        &self,
        request: PersistStandardFilesRequest,
    ) -> Result<PersistedStandardFiles, KnowledgeStorageError> {
        let drive_space_id = request.drive_space_id.as_deref();
        let paths = OkfBundlePaths::default();
        let agents_md = self
            .drive
            .put_object(
                PutKnowledgeObjectRequest::text(
                    paths.agents_md,
                    "bundle_profile",
                    render_agents_md(&request.space_name),
                    None,
                )
                .with_drive_space_id(drive_space_id),
            )
            .await?;
        let profile_yaml = self
            .drive
            .put_object(
                PutKnowledgeObjectRequest::text(
                    paths.profile_yaml,
                    "bundle_profile",
                    render_okf_profile_yaml(),
                    None,
                )
                .with_drive_space_id(drive_space_id),
            )
            .await?;
        let index_md = self
            .drive
            .head_object(
                HeadKnowledgeObjectRequest::managed_artifact(paths.index_md, "bundle_index")
                    .with_drive_space_id(drive_space_id),
            )
            .await?;
        let log_md = self
            .drive
            .head_object(
                HeadKnowledgeObjectRequest::managed_artifact(paths.log_md, "bundle_log")
                    .with_drive_space_id(drive_space_id),
            )
            .await?;

        Ok(PersistedStandardFiles {
            agents_md,
            profile_yaml,
            index_md,
            log_md,
        })
    }
}
