//! OKF v0.1 concept document parse/render (aligned with knowledge-catalog `layouts/okf.ts`).

use sdkwork_knowledgebase_contract::OkfBundlePaths;
use thiserror::Error;

pub const OKF_VERSION: &str = "0.1";
pub const SDKWORK_FRONTMATTER_KEY: &str = "sdkwork";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfConceptDocument {
    pub concept_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    pub tags: Vec<String>,
    pub timestamp: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfConceptLink {
    pub anchor_text: String,
    pub target_concept_id: Option<String>,
    pub raw_target: String,
    pub broken: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OkfDocumentError {
    #[error("invalid okf markdown: {0}")]
    Invalid(String),
}

pub fn parse_okf_markdown(content: &str) -> Result<Option<OkfConceptDocument>, OkfDocumentError> {
    let lines: Vec<&str> = content.split('\n').collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return Ok(None);
    }
    let end_index = lines
        .iter()
        .skip(1)
        .position(|line| line.trim() == "---")
        .ok_or_else(|| OkfDocumentError::Invalid("unclosed frontmatter block".to_string()))?
        + 1;
    let frontmatter = lines[1..end_index].join("\n");
    let body = lines[(end_index + 1)..].join("\n");
    let fields = parse_simple_yaml(&frontmatter)?;
    let concept_type = fields
        .get("type")
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| OkfDocumentError::Invalid("frontmatter type is required".to_string()))?;
    Ok(Some(OkfConceptDocument {
        concept_type,
        title: fields.get("title").cloned(),
        description: fields.get("description").cloned(),
        resource: fields.get("resource").cloned(),
        tags: parse_tags(fields.get("tags")),
        timestamp: fields.get("timestamp").cloned(),
        body,
    }))
}

pub fn render_okf_concept_markdown(document: &OkfConceptDocument) -> String {
    let mut frontmatter = String::from("---\n");
    frontmatter.push_str(&format!("type: {}\n", yaml_scalar(&document.concept_type)));
    if let Some(title) = &document.title {
        frontmatter.push_str(&format!("title: {}\n", yaml_scalar(title)));
    }
    if let Some(description) = &document.description {
        frontmatter.push_str(&format!("description: {}\n", yaml_scalar(description)));
    }
    if let Some(resource) = &document.resource {
        frontmatter.push_str(&format!("resource: {}\n", yaml_scalar(resource)));
    }
    if !document.tags.is_empty() {
        frontmatter.push_str("tags: [");
        frontmatter.push_str(
            &document
                .tags
                .iter()
                .map(|tag| yaml_scalar(tag))
                .collect::<Vec<_>>()
                .join(", "),
        );
        frontmatter.push_str("]\n");
    }
    if let Some(timestamp) = &document.timestamp {
        frontmatter.push_str(&format!("timestamp: {}\n", yaml_scalar(timestamp)));
    }
    frontmatter.push_str("---\n");
    let body = document.body.trim_end();
    if body.is_empty() {
        frontmatter.push('\n');
        frontmatter
    } else if body.ends_with('\n') {
        format!("{frontmatter}{body}")
    } else {
        format!("{frontmatter}{body}\n")
    }
}

pub fn render_root_index_frontmatter() -> &'static str {
    "---\nokf_version: \"0.1\"\n---\n"
}

pub fn extract_concept_links(
    body: &str,
    from_concept_id: &str,
    known_concept_ids: &[String],
) -> Vec<OkfConceptLink> {
    let known: std::collections::BTreeSet<&str> =
        known_concept_ids.iter().map(String::as_str).collect();
    markdown_links(body)
        .into_iter()
        .map(|(anchor, raw_target)| {
            let resolved = resolve_link_target(from_concept_id, &raw_target);
            let broken = resolved
                .as_ref()
                .is_none_or(|target| !known.contains(target.as_str()));
            OkfConceptLink {
                anchor_text: anchor,
                target_concept_id: resolved,
                raw_target,
                broken,
            }
        })
        .collect()
}

pub fn resolve_link_target(from_concept_id: &str, raw_target: &str) -> Option<String> {
    let target = raw_target.trim();
    if target.is_empty() || target.ends_with('/') {
        return None;
    }
    if target.starts_with("http://") || target.starts_with("https://") {
        return None;
    }
    let path = target
        .strip_prefix('/')
        .unwrap_or(target)
        .trim_start_matches("./");
    if !path.ends_with(".md") {
        return None;
    }
    let path = path.strip_suffix(".md")?;
    if path.is_empty() || path.contains("..") {
        return None;
    }
    if target.starts_with('/') {
        return Some(path.to_string());
    }
    let parent = from_concept_id
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");
    let joined = if parent.is_empty() {
        path.to_string()
    } else {
        format!("{parent}/{path}")
    };
    normalize_concept_path(&joined)
}

fn normalize_concept_path(path: &str) -> Option<String> {
    let mut segments = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return None;
        }
        segments.push(segment);
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments.join("/"))
    }
}

fn markdown_links(body: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let mut index = 0;
    while let Some(start) = body[index..].find('[') {
        let start = index + start;
        let rest = &body[start + 1..];
        let Some(text_end) = rest.find(']') else {
            break;
        };
        let anchor = rest[..text_end].to_string();
        let after = &rest[text_end + 1..];
        if !after.starts_with('(') {
            index = start + 1;
            continue;
        }
        let after = &after[1..];
        let Some(url_end) = after.find(')') else {
            break;
        };
        links.push((anchor, after[..url_end].to_string()));
        index = start + text_end + url_end + 3;
    }
    links
}

fn parse_simple_yaml(input: &str) -> Result<std::collections::BTreeMap<String, String>, OkfDocumentError> {
    let mut map = std::collections::BTreeMap::new();
    for line in input.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        map.insert(key.trim().to_string(), unquote_yaml_value(value.trim()));
    }
    Ok(map)
}

fn parse_tags(raw: Option<&String>) -> Vec<String> {
    let Some(raw) = raw else {
        return vec![];
    };
    let raw = raw.trim();
    if raw.starts_with('[') && raw.ends_with(']') {
        return raw[1..raw.len() - 1]
            .split(',')
            .map(|part| unquote_yaml_value(part.trim()))
            .filter(|part| !part.is_empty())
            .collect();
    }
    if !raw.is_empty() {
        vec![unquote_yaml_value(raw)]
    } else {
        vec![]
    }
}

fn unquote_yaml_value(value: &str) -> String {
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn yaml_scalar(value: &str) -> String {
    if value
        .chars()
        .any(|ch| ch.is_whitespace() || ch == ':' || ch == '#' || ch == '"' || ch == '\'')
    {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

pub fn concept_logical_path(concept_id: &str) -> String {
    OkfBundlePaths::concept_logical_path(concept_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_render_round_trip() {
        let markdown = r#"---
type: BigQuery Table
title: Users
description: One row per user.
tags: [users, schema]
timestamp: 2026-06-19T10:00:00Z
---
# Schema

See [customers](/tables/customers.md).
"#;
        let parsed = parse_okf_markdown(markdown).unwrap().unwrap();
        assert_eq!(parsed.concept_type, "BigQuery Table");
        assert_eq!(parsed.title.as_deref(), Some("Users"));
        let rendered = render_okf_concept_markdown(&parsed);
        let reparsed = parse_okf_markdown(&rendered).unwrap().unwrap();
        assert_eq!(reparsed.concept_type, parsed.concept_type);
        assert_eq!(reparsed.title, parsed.title);
        assert!(reparsed.body.contains("customers"));
    }

    #[test]
    fn resolve_bundle_relative_and_relative_links() {
        assert_eq!(
            resolve_link_target("tables/users", "/metrics/dau.md").as_deref(),
            Some("metrics/dau")
        );
        assert_eq!(
            resolve_link_target("tables/users", "./orders.md").as_deref(),
            Some("tables/orders")
        );
    }

    #[test]
    fn extract_links_marks_unknown_targets_as_broken() {
        let body = "See [orders](/tables/orders.md) and [missing](/missing/x.md).";
        let links = extract_concept_links(body, "tables/users", &["tables/orders".to_string()]);
        assert_eq!(links.len(), 2);
        assert!(!links[0].broken);
        assert!(links[1].broken);
    }
}
