use sdkwork_knowledgebase_contract::okf::OkfBundlePaths;

#[test]
fn okf_bundle_paths_match_standard_files() {
    let paths = OkfBundlePaths::default();

    assert_eq!(paths.agents_md, "okf/schema/AGENTS.md");
    assert_eq!(paths.profile_yaml, "okf/schema/okf_profile.yaml");
    assert_eq!(paths.index_md, "okf/index.md");
    assert_eq!(paths.log_md, "okf/log.md");
    assert_eq!(paths.governance_root, ".sdkwork/governance");
    assert_eq!(paths.local_mirror_agents_md, "schema/AGENTS.md");
    assert_eq!(paths.local_mirror_raw_root, "raw/");
    assert_eq!(paths.local_mirror_bundle_root, ".");
}

#[test]
fn okf_concept_logical_path_round_trip() {
    let concept_id = "tables/users";
    let logical = OkfBundlePaths::concept_logical_path(concept_id);
    assert_eq!(logical, "okf/tables/users.md");
    assert_eq!(
        OkfBundlePaths::concept_id_from_logical_path(&logical).as_deref(),
        Some(concept_id)
    );
}
