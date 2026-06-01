use sdkwork_knowledgebase_contract::mirror::{
    LlmWikiCompatibility, MirrorContentPolicy, MirrorDatabase, MirrorManifest,
};

#[test]
fn mirror_manifest_serializes_llm_wiki_compatibility_with_camel_case() {
    let manifest = MirrorManifest {
        schema_version: "1.0.0".to_string(),
        space_id: "space_uuid".to_string(),
        snapshot_version: "2026.06.01.000001".to_string(),
        base_snapshot_version: None,
        created_at: "2026-06-01T00:00:00Z".to_string(),
        package_kind: "snapshot".to_string(),
        content_policy: MirrorContentPolicy {
            include_raw_sources: false,
            include_parsed_artifacts: true,
            include_wiki: true,
            include_embeddings: true,
            include_eval_reports: false,
        },
        llm_wiki_compatibility: LlmWikiCompatibility {
            profile: "docs/llm-wiki.md".to_string(),
            agent_instruction_path: "AGENTS.md".to_string(),
            schema_path: "schema/wiki_schema.yaml".to_string(),
            raw_root: "raw/".to_string(),
            wiki_root: "wiki/".to_string(),
            index_path: "wiki/index.md".to_string(),
            log_path: "wiki/log.md".to_string(),
        },
        database: MirrorDatabase {
            engine: "sqlite".to_string(),
            schema_version: "1.0.0".to_string(),
            file: "sqlite/knowledgebase.sqlite".to_string(),
            checksum_sha256: "checksum".to_string(),
        },
        objects_manifest: "drive_objects/objects_manifest.jsonl".to_string(),
        index_manifests: vec!["indexes/full_text/index_manifest.json".to_string()],
        checksums: "checksums.sha256".to_string(),
    };

    let json = serde_json::to_value(&manifest).unwrap();

    assert_eq!(json["llmWikiCompatibility"]["profile"], "docs/llm-wiki.md");
    assert_eq!(
        json["llmWikiCompatibility"]["agentInstructionPath"],
        "AGENTS.md"
    );
    assert_eq!(json["llmWikiCompatibility"]["indexPath"], "wiki/index.md");
    assert_eq!(json["llmWikiCompatibility"]["logPath"], "wiki/log.md");
    assert_eq!(json["contentPolicy"]["includeRawSources"], false);
}
