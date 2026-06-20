use crate::okf::document::{extract_concept_links, parse_okf_markdown};
use crate::okf::validator::{validate_bundle_relative_path, validate_concept_document};
use crate::ports::knowledge_source_store::KnowledgeSourceLineageSnapshot;
use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
use std::collections::BTreeSet;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

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

pub fn extract_citation_urls(body: &str) -> Vec<String> {
    section_content_lines(body, "# Citations")
        .unwrap_or_default()
        .iter()
        .filter_map(|line| extract_markdown_link_url(line))
        .collect()
}

pub fn lint_concept_stale_claims(
    concept: &OkfConceptSummary,
    resource: Option<&str>,
    citation_urls: &[String],
    sources: &[KnowledgeSourceLineageSnapshot],
) -> Vec<OkfLintIssue> {
    let Some(concept_updated) = parse_rfc3339_timestamp(&concept.updated_at) else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    let mut matched_source = false;

    for source in sources {
        let references: Vec<&str> = resource
            .into_iter()
            .chain(citation_urls.iter().map(String::as_str))
            .collect();
        if !references
            .iter()
            .any(|reference| source_matches_lineage_reference(source, reference))
        {
            continue;
        }
        matched_source = true;
        let Some(source_activity) = source_activity_timestamp(source) else {
            continue;
        };
        if source_activity <= concept_updated {
            continue;
        }
        issues.push(OkfLintIssue {
            check: "stale_claims",
            severity: OkfLintSeverity::Warning,
            message: format!(
                "concept {} was last updated at {} but kb_source {} lineage changed at {}",
                concept.concept_id,
                concept.updated_at,
                source.source_id,
                format_source_activity(source)
            ),
            concept_id: Some(concept.concept_id.clone()),
        });
    }

    if !matched_source && concept.source_count > 0 {
        if let Some(newest_source_activity) = newest_space_source_activity(sources) {
            if let Some(newest) = parse_rfc3339_timestamp(&newest_source_activity) {
                if newest > concept_updated {
                    issues.push(OkfLintIssue {
                        check: "stale_claims",
                        severity: OkfLintSeverity::Warning,
                        message: format!(
                            "concept {} was last updated at {} but kb_source lineage changed at {}",
                            concept.concept_id, concept.updated_at, newest_source_activity
                        ),
                        concept_id: Some(concept.concept_id.clone()),
                    });
                }
            }
        }
    }

    issues
}

pub fn lint_stale_claims_against_source_lineage(
    concepts: &[OkfConceptSummary],
    sources: &[KnowledgeSourceLineageSnapshot],
) -> Vec<OkfLintIssue> {
    concepts
        .iter()
        .flat_map(|concept| lint_concept_stale_claims(concept, None, &[], sources))
        .collect()
}

fn newest_space_source_activity(sources: &[KnowledgeSourceLineageSnapshot]) -> Option<String> {
    sources
        .iter()
        .filter_map(|source| Some(format_source_activity(source)))
        .max_by(|left, right| {
            parse_rfc3339_timestamp(left)
                .zip(parse_rfc3339_timestamp(right))
                .map(|(left, right)| left.cmp(&right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn source_activity_timestamp(source: &KnowledgeSourceLineageSnapshot) -> Option<OffsetDateTime> {
    source
        .last_sync_at
        .as_deref()
        .or(Some(source.updated_at.as_str()))
        .and_then(parse_rfc3339_timestamp)
}

fn format_source_activity(source: &KnowledgeSourceLineageSnapshot) -> String {
    source
        .last_sync_at
        .clone()
        .unwrap_or_else(|| source.updated_at.clone())
}

fn source_matches_lineage_reference(
    source: &KnowledgeSourceLineageSnapshot,
    reference: &str,
) -> bool {
    let reference = reference.trim().to_ascii_lowercase();
    if reference.is_empty() {
        return false;
    }
    for candidate in [
        source.provider.as_deref(),
        source.drive_prefix.as_deref(),
        source.drive_bucket.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let candidate = candidate.trim().to_ascii_lowercase();
        if !candidate.is_empty() && reference.contains(&candidate) {
            return true;
        }
    }
    false
}

fn extract_markdown_link_url(line: &str) -> Option<String> {
    let start = line.find("](")? + 2;
    let rest = &line[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn parse_rfc3339_timestamp(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value.trim(), &Rfc3339).ok()
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

    #[test]
    fn stale_claims_flags_concepts_older_than_kb_source_lineage() {
        let concept = OkfConceptSummary {
            title: "Users".to_string(),
            concept_id: "tables/users".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/tables/users.md".to_string(),
            bundle_relative_path: "tables/users.md".to_string(),
            description: "User table".to_string(),
            source_count: 2,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec![],
        };
        let sources = vec![KnowledgeSourceLineageSnapshot {
            source_id: 1,
            updated_at: "2026-06-19T09:00:00Z".to_string(),
            last_sync_at: Some("2026-06-19T10:00:00Z".to_string()),
            provider: Some("stackoverflow".to_string()),
            drive_bucket: None,
            drive_prefix: None,
        }];
        let issues = lint_concept_stale_claims(&concept, None, &[], &sources);
        assert!(issues.iter().any(|issue| issue.check == "stale_claims"));
    }

    #[test]
    fn stale_claims_matches_resource_frontmatter_to_kb_source_provider() {
        let concept = OkfConceptSummary {
            title: "Users".to_string(),
            concept_id: "tables/users".to_string(),
            concept_type: "Entity".to_string(),
            logical_path: "okf/tables/users.md".to_string(),
            bundle_relative_path: "tables/users.md".to_string(),
            description: "User table".to_string(),
            source_count: 1,
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            tags: vec![],
        };
        let sources = vec![KnowledgeSourceLineageSnapshot {
            source_id: 7,
            updated_at: "2026-06-19T09:00:00Z".to_string(),
            last_sync_at: Some("2026-06-19T10:00:00Z".to_string()),
            provider: Some("stackoverflow".to_string()),
            drive_bucket: None,
            drive_prefix: Some("sources/raw/stackoverflow".to_string()),
        }];
        let issues = lint_concept_stale_claims(
            &concept,
            Some("sources/raw/stackoverflow/users.csv"),
            &[],
            &sources,
        );
        assert!(issues.iter().any(|issue| issue.check == "stale_claims"));
        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("kb_source 7")));
    }

    #[test]
    fn extract_citation_urls_reads_markdown_links() {
        let body = "# Citations\n\n[1] [Src](https://example.com/raw/users.csv)\n";
        assert_eq!(
            extract_citation_urls(body),
            vec!["https://example.com/raw/users.csv".to_string()]
        );
    }
}
