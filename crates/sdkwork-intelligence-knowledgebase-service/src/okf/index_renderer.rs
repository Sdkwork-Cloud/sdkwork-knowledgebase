use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
use sdkwork_utils_rust::is_blank;
use std::collections::{BTreeMap, BTreeSet};

use crate::okf::document::render_root_index_frontmatter;

pub fn render_index_md(_space_name: &str, concepts: &[OkfConceptSummary]) -> String {
    render_index_documents(concepts)
        .remove("index.md")
        .unwrap_or_else(|| format!("{}# Index\n\n_(empty)_\n", render_root_index_frontmatter()))
}

pub fn render_index_documents(concepts: &[OkfConceptSummary]) -> BTreeMap<String, String> {
    let mut documents = BTreeMap::new();
    let mut directories = collect_directories(concepts);
    if directories.is_empty() {
        directories.insert(String::new());
    }

    for directory in directories {
        let markdown = render_directory_index(&directory, concepts);
        let logical_path = if directory.is_empty() {
            "index.md".to_string()
        } else {
            format!("{directory}/index.md")
        };
        documents.insert(logical_path, markdown);
    }
    documents
}

fn collect_directories(concepts: &[OkfConceptSummary]) -> BTreeSet<String> {
    let mut directories = BTreeSet::new();
    directories.insert(String::new());
    for concept in concepts {
        let section = directory_section(&concept.bundle_relative_path);
        directories.insert(section.clone());
        let mut cursor = section.as_str();
        while let Some((parent, _)) = cursor.rsplit_once('/') {
            directories.insert(parent.to_string());
            cursor = parent;
        }
    }
    directories
}

fn render_directory_index(directory: &str, concepts: &[OkfConceptSummary]) -> String {
    let mut output = String::new();
    if directory.is_empty() {
        output.push_str(render_root_index_frontmatter());
    }
    output.push_str("# Index\n\n");

    let child_directories = direct_child_directories(directory, concepts);
    if !child_directories.is_empty() {
        output.push_str("## Sections\n\n");
        for child in child_directories {
            output.push_str("* [");
            output.push_str(&title_from_section(&child));
            output.push_str("](/");
            output.push_str(&child);
            output.push_str("/index.md)\n");
        }
        output.push('\n');
    }

    let local_concepts = direct_concepts_for_directory(directory, concepts);
    if local_concepts.is_empty() && output.ends_with("\n\n") {
        output.push_str("_(empty)_\n");
        return output;
    }
    if !local_concepts.is_empty() {
        output.push_str("## Concepts\n\n");
        for concept in local_concepts {
            output.push_str("* [");
            output.push_str(&one_line(&concept.title));
            output.push_str("](/");
            output.push_str(&concept.bundle_relative_path);
            output.push(')');
            if !is_blank(Some(concept.description.as_str())) {
                output.push_str(" - ");
                output.push_str(&one_line(&concept.description));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    output
}

fn directory_section(bundle_relative_path: &str) -> String {
    let path = bundle_relative_path.trim();
    let path = path.strip_suffix(".md").unwrap_or(path);
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn direct_child_directories(directory: &str, concepts: &[OkfConceptSummary]) -> Vec<String> {
    let mut children = BTreeSet::new();
    for concept in concepts {
        let section = directory_section(&concept.bundle_relative_path);
        if directory.is_empty() {
            if let Some((first, _)) = section.split_once('/') {
                children.insert(first.to_string());
            } else if !section.is_empty() {
                children.insert(section);
            }
            continue;
        }
        let Some(remainder) = section.strip_prefix(&(directory.to_string() + "/")) else {
            continue;
        };
        if remainder.is_empty() {
            continue;
        }
        let child = remainder
            .split_once('/')
            .map(|(segment, _)| segment)
            .unwrap_or(remainder);
        children.insert(format!("{directory}/{child}"));
    }
    children.into_iter().collect()
}

fn direct_concepts_for_directory<'a>(
    directory: &str,
    concepts: &'a [OkfConceptSummary],
) -> Vec<&'a OkfConceptSummary> {
    let mut items = concepts
        .iter()
        .filter(|concept| directory_section(&concept.bundle_relative_path) == directory)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.bundle_relative_path.cmp(&right.bundle_relative_path));
    items
}

fn title_from_section(section: &str) -> String {
    section
        .rsplit('/')
        .next()
        .unwrap_or(section)
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn concept(path: &str, title: &str) -> OkfConceptSummary {
        OkfConceptSummary {
            title: title.to_string(),
            concept_id: path.trim_end_matches(".md").to_string(),
            concept_type: "Entity".to_string(),
            logical_path: format!("okf/{path}"),
            bundle_relative_path: path.to_string(),
            description: format!("desc-{title}"),
            source_count: 1,
            updated_at: "2026-06-20T00:00:00Z".to_string(),
            tags: vec![],
        }
    }

    #[test]
    fn render_index_documents_includes_nested_indexes() {
        let concepts = vec![
            concept("tables/users.md", "Users"),
            concept("tables/posts/questions.md", "Questions"),
        ];
        let docs = render_index_documents(&concepts);
        assert!(docs.contains_key("index.md"));
        assert!(docs.contains_key("tables/index.md"));
        assert!(docs.contains_key("tables/posts/index.md"));

        let root = docs.get("index.md").expect("root index");
        assert!(root.contains("(/tables/index.md)"));
        let tables = docs.get("tables/index.md").expect("tables index");
        assert!(tables.contains("(/tables/users.md)"));
        assert!(tables.contains("(/tables/posts/index.md)"));
    }
}
