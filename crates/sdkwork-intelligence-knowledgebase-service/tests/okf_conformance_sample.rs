use sdkwork_intelligence_knowledgebase_service::okf::{
    extract_concept_links, parse_okf_markdown, validate_bundle_relative_path,
    validate_concept_bundle_relative_path, validate_concept_document,
};
use std::path::PathBuf;

#[test]
fn stackoverflow_sample_concept_passes_okf_conformance() {
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../external/knowledge-catalog/okf/bundles/stackoverflow/tables/users.md");
    let markdown = std::fs::read_to_string(sample).expect("stackoverflow sample must exist");
    let concept_id = "tables/users";
    validate_bundle_relative_path("tables/users.md").expect("bundle path must be valid");
    validate_concept_bundle_relative_path("tables/users.md").expect("concept path must be valid");
    let document = parse_okf_markdown(&markdown)
        .expect("sample markdown must parse")
        .expect("sample must include frontmatter");
    validate_concept_document(&document, concept_id).expect("sample must conform to OKF");
    assert_eq!(document.concept_type, "BigQuery Table");
    assert!(document.body.contains("# Schema"));
    assert!(document.body.contains("# Citations"));
}

#[test]
fn stackoverflow_sample_resolves_parent_relative_links() {
    let votes = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../external/knowledge-catalog/okf/bundles/stackoverflow/tables/votes.md");
    let markdown = std::fs::read_to_string(votes).expect("votes sample must exist");
    let document = parse_okf_markdown(&markdown)
        .expect("votes sample must parse")
        .expect("votes sample must include frontmatter");
    let known = vec![
        "datasets/stackoverflow".to_string(),
        "tables/users".to_string(),
    ];
    let links = extract_concept_links(&document.body, "tables/votes", &known);
    assert!(
        links
            .iter()
            .any(|link| link.target_concept_id.as_deref() == Some("datasets/stackoverflow")),
        "expected parent-relative dataset link to resolve"
    );
}

#[test]
fn index_frontmatter_without_type_is_not_a_concept_document() {
    validate_bundle_relative_path("index.md").expect("root index is valid bundle path");
    assert!(validate_concept_bundle_relative_path("index.md").is_err());
    let markdown = "---\nokf_version: \"0.1\"\n---\n# Index\n";
    assert!(parse_okf_markdown(markdown).unwrap().is_none());
}
