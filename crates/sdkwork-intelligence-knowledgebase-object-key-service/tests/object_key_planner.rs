use sdkwork_intelligence_knowledgebase_object_key_service::object_key::{
    safe_file_name, KnowledgeObjectKeyPlanner, ObjectKeyPlanError,
};

#[test]
fn planner_generates_standard_llm_wiki_object_keys() {
    let planner = KnowledgeObjectKeyPlanner::new("tenant-1", "space-uuid").unwrap();

    assert_eq!(
        planner.llm_wiki_file("wiki/index.md").unwrap(),
        "knowledge/tenant-1/space-uuid/wiki/index.md"
    );
    assert_eq!(
        planner.llm_wiki_file("wiki/schema/AGENTS.md").unwrap(),
        "knowledge/tenant-1/space-uuid/wiki/schema/AGENTS.md"
    );
}

#[test]
fn planner_generates_raw_source_original_keys_with_safe_file_names() {
    let planner = KnowledgeObjectKeyPlanner::new("tenant-1", "space-uuid").unwrap();

    let key = planner
        .raw_source_original("source-uuid", "..\\Quarterly Report 2026?.PDF")
        .unwrap();

    assert_eq!(
        key,
        "knowledge/tenant-1/space-uuid/sources/raw/source-uuid/original/Quarterly-Report-2026.PDF"
    );
}

#[test]
fn planner_rejects_path_traversal_and_absolute_paths() {
    let planner = KnowledgeObjectKeyPlanner::new("tenant-1", "space-uuid").unwrap();

    assert!(matches!(
        planner.llm_wiki_file("../secrets.md"),
        Err(ObjectKeyPlanError::UnsafePath(_))
    ));
    assert!(matches!(
        planner.llm_wiki_file("/wiki/index.md"),
        Err(ObjectKeyPlanError::UnsafePath(_))
    ));
}

#[test]
fn safe_file_name_preserves_extension_and_removes_unsafe_segments() {
    assert_eq!(
        safe_file_name("..\\My Research: Notes?.md").unwrap(),
        "My-Research-Notes.md"
    );
    assert!(safe_file_name("..").is_err());
}
