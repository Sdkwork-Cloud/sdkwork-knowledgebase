use sdkwork_knowledgebase_contract::okf::OkfConceptSummary;
use std::collections::BTreeMap;

use crate::okf::document::render_root_index_frontmatter;

pub fn render_index_md(_space_name: &str, concepts: &[OkfConceptSummary]) -> String {
    let mut output = String::new();
    output.push_str(render_root_index_frontmatter());
    output.push_str("# Index\n\n");

    if concepts.is_empty() {
        output.push_str("_(empty)_\n");
        return output;
    }

    let mut grouped: BTreeMap<String, Vec<&OkfConceptSummary>> = BTreeMap::new();
    for concept in concepts {
        let section = directory_section(&concept.bundle_relative_path);
        grouped.entry(section).or_default().push(concept);
    }

    for (section, items) in grouped {
        if section.is_empty() {
            output.push_str("## Concepts\n\n");
        } else {
            output.push_str("## ");
            output.push_str(&title_from_section(&section));
            output.push_str("\n\n");
        }
        for concept in items {
            output.push_str("* [");
            output.push_str(&one_line(&concept.title));
            output.push_str("](");
            output.push_str(&concept.bundle_relative_path);
            output.push(')');
            if !concept.description.trim().is_empty() {
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
