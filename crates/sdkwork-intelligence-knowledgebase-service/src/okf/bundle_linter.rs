use crate::okf::document::parse_okf_markdown;
use crate::okf::linter::{
    extract_citation_urls, extract_index_linked_concept_ids, lint_bundle_summaries,
    lint_concept_stale_claims, lint_published_concept_markdown, OkfBundleLintReport, OkfLintIssue,
};
use crate::okf::storage::read_managed_markdown;
use crate::ports::knowledge_drive_storage::KnowledgeDriveStorage;
use crate::ports::knowledge_okf_concept_link_store::{
    KnowledgeOkfConceptLinkStore, KnowledgeOkfConceptLinkStoreError,
};
use crate::ports::knowledge_okf_concept_store::{
    KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError,
};
use crate::ports::knowledge_source_store::{KnowledgeSourceStore, KnowledgeSourceStoreError};
use sdkwork_knowledgebase_contract::okf::{OkfBundleLintResult, OkfBundlePaths};
use thiserror::Error;

pub struct OkfBundleLinterService<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    concept_store: &'a dyn KnowledgeOkfConceptStore,
    link_store: Option<&'a dyn KnowledgeOkfConceptLinkStore>,
    source_store: Option<&'a dyn KnowledgeSourceStore>,
}

impl<'a> OkfBundleLinterService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        concept_store: &'a dyn KnowledgeOkfConceptStore,
    ) -> Self {
        Self {
            drive,
            concept_store,
            link_store: None,
            source_store: None,
        }
    }

    pub fn with_link_store(mut self, link_store: &'a dyn KnowledgeOkfConceptLinkStore) -> Self {
        self.link_store = Some(link_store);
        self
    }

    pub fn with_source_store(mut self, source_store: &'a dyn KnowledgeSourceStore) -> Self {
        self.source_store = Some(source_store);
        self
    }

    pub async fn lint_space(
        &self,
        space_id: u64,
    ) -> Result<OkfBundleLintReport, OkfBundleLinterError> {
        let concepts = self.concept_store.list_concept_summaries(space_id).await?;
        let known = concepts
            .iter()
            .map(|concept| concept.concept_id.clone())
            .collect::<Vec<_>>();

        let source_lineage = if let Some(source_store) = self.source_store {
            Some(source_store.list_space_source_lineage(space_id).await?)
        } else {
            None
        };

        let mut issues = Vec::new();
        for concept in &concepts {
            match read_managed_markdown(self.drive, &concept.logical_path).await {
                Ok(markdown) => {
                    issues.extend(lint_published_concept_markdown(
                        &concept.concept_id,
                        &markdown,
                        &known,
                    ));
                    if let Some(sources) = source_lineage.as_deref() {
                        let resource = parse_okf_markdown(&markdown)
                            .ok()
                            .flatten()
                            .and_then(|document| document.resource);
                        let citation_urls = extract_citation_urls(&markdown);
                        issues.extend(lint_concept_stale_claims(
                            concept,
                            resource.as_deref(),
                            &citation_urls,
                            sources,
                        ));
                    }
                }
                Err(error) => issues.push(OkfLintIssue {
                    check: "okf_conformance",
                    severity: crate::okf::linter::OkfLintSeverity::Error,
                    message: format!(
                        "failed to read published concept {} from drive: {error}",
                        concept.concept_id
                    ),
                    concept_id: Some(concept.concept_id.clone()),
                }),
            }
        }

        let orphan_concept_ids = if let Some(link_store) = self.link_store {
            let mut orphans = link_store.list_orphan_concept_ids(space_id, &known).await?;
            let index_roots =
                match read_managed_markdown(self.drive, OkfBundlePaths::default().index_md).await {
                    Ok(index_markdown) => extract_index_linked_concept_ids(&index_markdown, &known),
                    Err(_) => Default::default(),
                };
            orphans.retain(|concept_id| !index_roots.contains(concept_id));
            orphans
        } else {
            Vec::new()
        };
        issues.extend(lint_bundle_summaries(&concepts, &orphan_concept_ids).issues);
        Ok(OkfBundleLintReport { issues })
    }
}

pub fn to_contract_lint_result(report: &OkfBundleLintReport) -> OkfBundleLintResult {
    OkfBundleLintResult {
        conformance: if report.conformance_passed() {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        issues: report
            .issues
            .iter()
            .map(|issue| sdkwork_knowledgebase_contract::okf::OkfLintIssue {
                code: issue.check.to_string(),
                severity: match issue.severity {
                    crate::okf::linter::OkfLintSeverity::Error => "error".to_string(),
                    crate::okf::linter::OkfLintSeverity::Warning => "warning".to_string(),
                },
                message: issue.message.clone(),
                concept_id: issue.concept_id.clone(),
            })
            .collect(),
    }
}

#[derive(Debug, Error)]
pub enum OkfBundleLinterError {
    #[error(transparent)]
    ConceptStore(#[from] KnowledgeOkfConceptStoreError),
    #[error(transparent)]
    LinkStore(#[from] KnowledgeOkfConceptLinkStoreError),
    #[error(transparent)]
    SourceStore(#[from] KnowledgeSourceStoreError),
}
