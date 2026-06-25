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
use sdkwork_knowledgebase_observability::record_okf_bundle_lint_completed;
use std::collections::BTreeSet;
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
        drive_space_id: Option<&str>,
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
            match read_managed_markdown(self.drive, &concept.logical_path, drive_space_id).await {
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
            let index_linked_concepts = self
                .read_all_index_linked_concepts(&concepts, &known, drive_space_id)
                .await;
            orphans.retain(|concept_id| !index_linked_concepts.contains(concept_id));
            orphans
        } else {
            Vec::new()
        };
        issues.extend(lint_bundle_summaries(&concepts, &orphan_concept_ids).issues);
        let report = OkfBundleLintReport { issues };
        let conformance_failures = report
            .issues
            .iter()
            .filter(|issue| {
                issue.check == "okf_conformance"
                    && issue.severity == crate::okf::linter::OkfLintSeverity::Error
            })
            .count() as u64;
        record_okf_bundle_lint_completed(
            space_id,
            report.issues.len() as u64,
            conformance_failures,
        );
        Ok(report)
    }

    async fn read_all_index_linked_concepts(
        &self,
        concepts: &[sdkwork_knowledgebase_contract::okf::OkfConceptSummary],
        known: &[String],
        drive_space_id: Option<&str>,
    ) -> BTreeSet<String> {
        let mut linked = BTreeSet::new();
        for logical_path in hierarchical_index_paths(concepts) {
            if let Ok(index_markdown) =
                read_managed_markdown(self.drive, &logical_path, drive_space_id).await
            {
                linked.extend(extract_index_linked_concept_ids(&index_markdown, known));
            }
        }
        linked
    }
}

fn hierarchical_index_paths(
    concepts: &[sdkwork_knowledgebase_contract::okf::OkfConceptSummary],
) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    paths.insert(OkfBundlePaths::default().index_md.to_string());
    for concept in concepts {
        let mut directory = concept
            .bundle_relative_path
            .trim()
            .trim_end_matches(".md")
            .rsplit_once('/')
            .map(|(parent, _)| parent.to_string());
        while let Some(current) = directory {
            paths.insert(format!("okf/{current}/index.md"));
            directory = current
                .rsplit_once('/')
                .map(|(parent, _)| parent.to_string());
        }
    }
    paths
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierarchical_index_paths_include_parent_directories() {
        let concepts = vec![sdkwork_knowledgebase_contract::okf::OkfConceptSummary {
            title: "Users".to_string(),
            concept_id: "tables/users".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/tables/users.md".to_string(),
            bundle_relative_path: "tables/users.md".to_string(),
            description: "Users table".to_string(),
            source_count: 1,
            updated_at: "2026-06-20T00:00:00Z".to_string(),
            tags: vec![],
        }];

        let paths = hierarchical_index_paths(&concepts);
        assert!(paths.contains("okf/index.md"));
        assert!(paths.contains("okf/tables/index.md"));
    }
}
