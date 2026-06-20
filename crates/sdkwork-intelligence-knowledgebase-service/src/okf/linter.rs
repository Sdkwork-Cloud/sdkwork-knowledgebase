use crate::okf::document::{extract_concept_links, parse_okf_markdown};
use crate::okf::validator::{validate_bundle_relative_path, validate_concept_document};
use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfLintIssue {
    pub check: &'static str,
    pub severity: OkfLintSeverity,
    pub message: String,
    pub concept_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OkfLintSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfBundleLintReport {
    pub issues: Vec<OkfLintIssue>,
}

impl OkfBundleLintReport {
    pub fn conformance_passed(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|issue| issue.check == "okf_conformance" && issue.severity == OkfLintSeverity::Error)
    }
}

pub fn lint_published_concept_markdown(
    concept_id: &str,
    markdown: &str,
    known_concept_ids: &[String],
) -> Vec<OkfLintIssue> {
    let mut issues = Vec::new();
    match parse_okf_markdown(markdown) {
        Ok(Some(document)) => {
            if let Err(error) = validate_concept_document(&document, concept_id) {
                issues.push(OkfLintIssue {
                    check: "okf_conformance",
                    severity: OkfLintSeverity::Error,
                    message: error.to_string(),
                    concept_id: Some(concept_id.to_string()),
                });
            }
            for link in extract_concept_links(&document.body, concept_id, known_concept_ids) {
                if link.broken {
                    issues.push(OkfLintIssue {
                        check: "broken_links",
                        severity: OkfLintSeverity::Warning,
                        message: format!(
                            "broken concept link from {} to {}",
                            concept_id, link.raw_target
                        ),
                        concept_id: Some(concept_id.to_string()),
                    });
                }
            }
            if document.body.contains("TODO") || document.body.contains("FIXME") {
                issues.push(OkfLintIssue {
                    check: "stale_claims",
                    severity: OkfLintSeverity::Warning,
                    message: format!(
                        "concept {} contains unresolved TODO/FIXME markers",
                        concept_id
                    ),
                    concept_id: Some(concept_id.to_string()),
                });
            }
        }
        Ok(None) => issues.push(OkfLintIssue {
            check: "okf_conformance",
            severity: OkfLintSeverity::Error,
            message: "published concept markdown must include YAML frontmatter with type".to_string(),
            concept_id: Some(concept_id.to_string()),
        }),
        Err(error) => issues.push(OkfLintIssue {
            check: "okf_conformance",
            severity: OkfLintSeverity::Error,
            message: error.to_string(),
            concept_id: Some(concept_id.to_string()),
        }),
    }
    issues
}

pub fn lint_bundle_summaries(
    concepts: &[OkfConceptSummary],
    orphan_concept_ids: &[String],
) -> OkfBundleLintReport {
    let mut issues = Vec::new();
    for concept in concepts {
        if let Err(error) = validate_bundle_relative_path(&concept.bundle_relative_path) {
            issues.push(OkfLintIssue {
                check: "okf_conformance",
                severity: OkfLintSeverity::Error,
                message: error.to_string(),
                concept_id: Some(concept.concept_id.clone()),
            });
        }
        if concept.description.trim().is_empty() {
            issues.push(OkfLintIssue {
                check: "missing_citations",
                severity: OkfLintSeverity::Warning,
                message: format!(
                    "concept {} is missing a frontmatter description used by index.md",
                    concept.concept_id
                ),
                concept_id: Some(concept.concept_id.clone()),
            });
        }
    }

    let published: BTreeSet<&str> = concepts
        .iter()
        .map(|concept| concept.concept_id.as_str())
        .collect();
    for concept_id in orphan_concept_ids {
        if published.contains(concept_id.as_str()) {
            issues.push(OkfLintIssue {
                check: "orphan_concepts",
                severity: OkfLintSeverity::Warning,
                message: format!("concept {concept_id} has no inbound concept links"),
                concept_id: Some(concept_id.clone()),
            });
        }
    }

    OkfBundleLintReport { issues }
}
