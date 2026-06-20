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
        !self.issues.iter().any(|issue| {
            issue.check == "okf_conformance" && issue.severity == OkfLintSeverity::Error
        })
    }
}

pub fn extract_index_linked_concept_ids(
    index_markdown: &str,
    known_concept_ids: &[String],
) -> BTreeSet<String> {
    let body = index_markdown
        .split("\n---\n")
        .nth(1)
        .and_then(|rest| rest.split("\n---\n").nth(1))
        .unwrap_or(index_markdown);
    extract_concept_links(body, "", known_concept_ids)
        .into_iter()
        .filter_map(|link| link.target_concept_id)
        .collect()
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
            if !has_citations_section(&document.body) {
                issues.push(OkfLintIssue {
                    check: "missing_citations",
                    severity: OkfLintSeverity::Warning,
                    message: format!(
                        "concept {} is missing a # Citations section for external source lineage",
                        concept_id
                    ),
                    concept_id: Some(concept_id.to_string()),
                });
            } else if citations_section_is_empty(&document.body) {
                issues.push(OkfLintIssue {
                    check: "stale_claims",
                    severity: OkfLintSeverity::Warning,
                    message: format!("concept {} has an empty # Citations section", concept_id),
                    concept_id: Some(concept_id.to_string()),
                });
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
            message: "published concept markdown must include YAML frontmatter with type"
                .to_string(),
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

fn has_citations_section(body: &str) -> bool {
    section_content_lines(body, "# Citations").is_some()
}

fn citations_section_is_empty(body: &str) -> bool {
    match section_content_lines(body, "# Citations") {
        Some(lines) => lines.is_empty(),
        None => false,
    }
}

fn section_content_lines(body: &str, heading: &str) -> Option<Vec<String>> {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in body.lines() {
        let stripped = line.trim();
        if stripped.starts_with("# ") {
            in_section = stripped == heading;
            continue;
        }
        if in_section && !stripped.is_empty() {
            lines.push(line.to_string());
        }
    }
    if in_section {
        Some(lines)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citations_section_detection_matches_okf_reference() {
        let body = "Prose.\n\n# Schema\n\n- `id` STRING\n\n# Citations\n\n[1] [Src](https://example.com)\n";
        assert!(has_citations_section(body));
        assert!(!citations_section_is_empty(body));
    }

    #[test]
    fn missing_citations_flags_concepts_without_section() {
        let markdown = r#"---
type: Entity
title: Users
---
# Schema

- `id` STRING
"#;
        let issues = lint_published_concept_markdown("tables/users", markdown, &[]);
        assert!(issues
            .iter()
            .any(|issue| issue.check == "missing_citations"));
    }
}
