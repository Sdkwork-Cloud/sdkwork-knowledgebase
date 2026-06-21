mod bundle_linter;
mod bundle_workflow;
mod concept_service;
mod document;
mod exporter;
mod file_registry;
mod importer;
mod index_rebuild;
mod index_renderer;
mod initializer;
mod link_indexer;
mod linter;
mod log_renderer;
mod schema_renderer;
mod storage;
mod validator;

use crate::ports::knowledge_drive_storage::{
    KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError, PutKnowledgeObjectRequest,
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
pub use importer::{
    bundle_relative_path_from_logical_path, concept_id_from_bundle_relative_path,
    discover_bundle_files_from_directory, drive_import_root, load_import_bundle_from_drive,
    stackoverflow_bundle_root, stage_export_bundle_for_drive_import, ImportOkfBundleFile,
    ImportOkfBundleRequest, ImportOkfBundleResult, OkfBundleImporterError,
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
        let paths = OkfBundlePaths::default();
        let agents_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.agents_md,
                "bundle_profile",
                render_agents_md(&request.space_name),
                None,
            ))
            .await?;
        let profile_yaml = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.profile_yaml,
                "bundle_profile",
                render_okf_profile_yaml(),
                None,
            ))
            .await?;
        let index_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.index_md,
                "bundle_index",
                render_index_md(&request.space_name, &request.concepts),
                None,
            ))
            .await?;
        let index_documents = render_index_documents(&request.concepts);
        for (bundle_relative_path, markdown) in index_documents {
            if bundle_relative_path == "index.md" {
                continue;
            }
            self.drive
                .put_object(PutKnowledgeObjectRequest::text(
                    format!("okf/{bundle_relative_path}"),
                    "bundle_index",
                    markdown,
                    None,
                ))
                .await?;
        }
        let log_md = self
            .drive
            .put_object(PutKnowledgeObjectRequest::text(
                paths.log_md,
                "bundle_log",
                render_log_md(&request.log_entries),
                None,
            ))
            .await?;

        Ok(PersistedStandardFiles {
            agents_md,
            profile_yaml,
            index_md,
            log_md,
        })
    }
}
