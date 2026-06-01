use sdkwork_knowledgebase_contract::wiki::{
    LlmWikiPaths, WikiCandidateType, WikiLogEventType, WikiPageType,
};

#[test]
fn llm_wiki_paths_match_standard_files() {
    let paths = LlmWikiPaths::default();

    assert_eq!(paths.agents_md, "wiki/schema/AGENTS.md");
    assert_eq!(paths.schema_yaml, "wiki/schema/wiki_schema.yaml");
    assert_eq!(paths.index_md, "wiki/index.md");
    assert_eq!(paths.log_md, "wiki/log.md");
    assert_eq!(paths.local_mirror_agents_md, "AGENTS.md");
    assert_eq!(paths.local_mirror_raw_root, "raw/");
}

#[test]
fn llm_wiki_enums_use_snake_case_wire_values() {
    assert_eq!(WikiPageType::Answer.as_str(), "answer");
    assert_eq!(WikiPageType::Comparison.as_str(), "comparison");
    assert_eq!(WikiCandidateType::QueryAnswer.as_str(), "query_answer");
    assert_eq!(WikiLogEventType::DeltaUpdate.as_str(), "delta_update");
}
