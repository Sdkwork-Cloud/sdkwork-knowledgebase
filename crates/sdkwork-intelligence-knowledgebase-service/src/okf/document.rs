//! OKF v0.1 concept document parse/render (aligned with knowledge-catalog `layouts/okf.ts`).

use sdkwork_utils_rust::is_blank;
use serde::Deserialize;
use std::collections::BTreeMap;
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
    pub extensions: BTreeMap<String, serde_json::Value>,
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
    let fields = parse_frontmatter_yaml(&frontmatter)?;
    let concept_type = match fields
        .concept_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(concept_type) => concept_type.to_string(),
        None => {
            if fields.okf_version.is_some() {
                return Ok(None);
            }
            return Err(OkfDocumentError::Invalid(
                "frontmatter type is required".to_string(),
            ));
        }
    };
    Ok(Some(OkfConceptDocument {
        concept_type,
        title: fields.title,
        description: yaml_scalar_string(fields.description.as_ref()),
        resource: fields.resource,
        tags: parse_yaml_tags(fields.tags.as_ref()),
        timestamp: fields.timestamp,
        extensions: fields.extensions,
        body,
    }))
}

#[derive(Debug, Deserialize)]
struct OkfFrontmatterFields {
    #[serde(rename = "type")]
    concept_type: Option<String>,
    okf_version: Option<String>,
    title: Option<String>,
    description: Option<serde_yaml::Value>,
    resource: Option<String>,
    tags: Option<serde_yaml::Value>,
    timestamp: Option<String>,
    #[serde(flatten)]
    extensions: BTreeMap<String, serde_json::Value>,
}

fn parse_frontmatter_yaml(input: &str) -> Result<OkfFrontmatterFields, OkfDocumentError> {
    serde_yaml::from_str(input).map_err(|error| {
        OkfDocumentError::Invalid(format!("invalid okf frontmatter yaml: {error}"))
    })
}

fn yaml_scalar_string(value: Option<&serde_yaml::Value>) -> Option<String> {
    let value = value?;
    match value {
        serde_yaml::Value::Null => None,
        serde_yaml::Value::String(text) => Some(text.clone()),
        serde_yaml::Value::Number(number) => Some(number.to_string()),
        serde_yaml::Value::Bool(flag) => Some(flag.to_string()),
        _ => serde_yaml::to_string(value)
            .ok()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty()),
    }
}

fn parse_yaml_tags(value: Option<&serde_yaml::Value>) -> Vec<String> {
    let Some(value) = value else {
        return vec![];
    };
    match value {
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .filter_map(|item| yaml_scalar_string(Some(item)))
            .collect(),
        serde_yaml::Value::String(text) if !is_blank(Some(text.as_str())) => vec![text.clone()],
        _ => vec![],
    }
}

pub fn render_okf_concept_markdown(document: &OkfConceptDocument) -> String {
    let mut fields = BTreeMap::<String, serde_json::Value>::new();
    for (key, value) in &document.extensions {
        if !is_standard_frontmatter_key(key) {
            fields.insert(key.clone(), value.clone());
        }
    }
    fields.insert("type".to_string(), document.concept_type.clone().into());
    if let Some(title) = &document.title {
        fields.insert("title".to_string(), title.clone().into());
    }
    if let Some(description) = &document.description {
        fields.insert("description".to_string(), description.clone().into());
    }
    if let Some(resource) = &document.resource {
        fields.insert("resource".to_string(), resource.clone().into());
    }
    if !document.tags.is_empty() {
        fields.insert(
            "tags".to_string(),
            serde_json::Value::Array(document.tags.iter().cloned().map(Into::into).collect()),
        );
    }
    if let Some(timestamp) = &document.timestamp {
        fields.insert("timestamp".to_string(), timestamp.clone().into());
    }
    let yaml = serde_yaml::to_string(&fields)
        .expect("serializing an in-memory OKF frontmatter mapping must succeed");
    let frontmatter = format!("---\n{}---\n", yaml.trim_start_matches("---\n"));
    let body = document.body.trim_end();
    if body.is_empty() {
        format!("{frontmatter}\n")
    } else {
        format!("{frontmatter}{body}\n")
    }
}

fn is_standard_frontmatter_key(key: &str) -> bool {
    matches!(
        key,
        "type" | "title" | "description" | "resource" | "tags" | "timestamp" | "okf_version"
    )
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
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
    {
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
    if path.is_empty() {
        return None;
    }

    let joined = if target.starts_with('/') {
        path.to_string()
    } else {
        let parent = from_concept_id
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        if parent.is_empty() {
            path.to_string()
        } else {
            format!("{parent}/{path}")
        }
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
            segments.pop()?;
            continue;
        }
        segments.push(segment.to_string());
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

pub fn strip_sdkwork_frontmatter(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.split('\n').collect();
    if lines.first().map(|line| line.trim()) != Some("---") {
        return markdown.to_string();
    }
    let Some(closing_index) = lines.iter().skip(1).position(|line| line.trim() == "---") else {
        return markdown.to_string();
    };
    let end_index = closing_index + 1;
    let frontmatter = lines[1..end_index].join("\n");
    let Ok(mut fields) = serde_yaml::from_str::<serde_yaml::Mapping>(&frontmatter) else {
        return markdown.to_string();
    };
    fields.retain(|key, _| {
        key.as_str()
            .is_none_or(|key| key != SDKWORK_FRONTMATTER_KEY && !key.starts_with("sdkwork."))
    });
    let Ok(frontmatter) = serde_yaml::to_string(&fields) else {
        return markdown.to_string();
    };
    let body = lines[(end_index + 1)..].join("\n");
    let rendered_frontmatter = format!("---\n{}---", frontmatter.trim_start_matches("---\n"));
    if body.is_empty() {
        format!("{rendered_frontmatter}\n")
    } else if body.ends_with('\n') {
        format!("{rendered_frontmatter}\n{body}")
    } else {
        format!("{rendered_frontmatter}\n{body}\n")
    }
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

    #[test]
    fn resolve_parent_relative_and_sibling_links() {
        assert_eq!(
            resolve_link_target("tables/votes", "../datasets/stackoverflow.md").as_deref(),
            Some("datasets/stackoverflow")
        );
        assert_eq!(
            resolve_link_target("tables/posts_questions", "posts_answers.md").as_deref(),
            Some("tables/posts_answers")
        );
        assert_eq!(
            resolve_link_target("tables/posts_tag_wiki", "tags.md").as_deref(),
            Some("tables/tags")
        );
    }

    #[test]
    fn strip_sdkwork_frontmatter_removes_extension_block() {
        let markdown = r#"---
type: Entity
title: Users
sdkwork:
  revisionId: 42
---
# Body
"#;
        let stripped = strip_sdkwork_frontmatter(markdown);
        assert!(!stripped.contains("sdkwork:"));
        assert!(!stripped.contains("revisionId:"));
        assert!(stripped.contains("type: Entity"));
        assert!(stripped.contains("# Body"));
    }

    #[test]
    fn parse_and_render_preserves_unknown_frontmatter_extensions() {
        let markdown = r#"---
type: Entity
owner:
  team: platform
confidence: 0.95
---
# Body
"#;
        let parsed = parse_okf_markdown(markdown).unwrap().unwrap();
        assert_eq!(parsed.extensions["confidence"], serde_json::json!(0.95));
        let rendered = render_okf_concept_markdown(&parsed);
        let reparsed = parse_okf_markdown(&rendered).unwrap().unwrap();
        assert_eq!(reparsed.extensions, parsed.extensions);
    }

    #[test]
    fn render_uses_yaml_safe_scalars() {
        let document = OkfConceptDocument {
            concept_type: "Path\\Type".to_string(),
            title: Some("quoted: # value".to_string()),
            description: None,
            resource: None,
            tags: vec!["line\\next".to_string()],
            timestamp: None,
            extensions: Default::default(),
            body: "Body".to_string(),
        };
        let rendered = render_okf_concept_markdown(&document);
        let reparsed = parse_okf_markdown(&rendered).unwrap().unwrap();
        assert_eq!(reparsed.concept_type, document.concept_type);
        assert_eq!(reparsed.title, document.title);
        assert_eq!(reparsed.tags, document.tags);
    }

    #[test]
    fn parse_stackoverflow_users_frontmatter() {
        let markdown = include_str!(
            "../../../../external/knowledge-catalog/okf/bundles/stackoverflow/tables/users.md"
        );
        let parsed = parse_okf_markdown(markdown).unwrap().unwrap();
        assert_eq!(parsed.concept_type, "BigQuery Table");
        assert_eq!(parsed.title.as_deref(), Some("Users"));
        assert!(parsed
            .description
            .as_deref()
            .is_some_and(|description| description.contains("Stack Overflow")));
        assert!(parsed.tags.iter().any(|tag| tag == "Stack Overflow"));
        assert!(parsed.tags.iter().any(|tag| tag == "users"));
        assert!(parsed.body.contains("# Schema"));
    }
}
