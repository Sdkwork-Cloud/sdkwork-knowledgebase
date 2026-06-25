use sdkwork_knowledgebase_contract::mirror::{
    MirrorContentPolicy, MirrorDatabase, MirrorManifest, OkfBundleCompatibility,
};

#[test]
fn mirror_manifest_serializes_okf_bundle_compatibility_with_camel_case() {
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
            include_okf_bundle: true,
            include_embeddings: true,
            include_eval_reports: false,
        },
        okf_bundle_compatibility: OkfBundleCompatibility {
            okf_version: "0.1".to_string(),
            profile: "docs/okf.md".to_string(),
            agent_instruction_path: "schema/AGENTS.md".to_string(),
            profile_path: "schema/okf_profile.yaml".to_string(),
            raw_root: "raw/".to_string(),
            bundle_root: ".".to_string(),
            index_path: "index.md".to_string(),
            log_path: "log.md".to_string(),
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

    assert_eq!(json["okfBundleCompatibility"]["profile"], "docs/okf.md");
    assert_eq!(
        json["okfBundleCompatibility"]["agentInstructionPath"],
        "schema/AGENTS.md"
    );
    assert_eq!(
        json["okfBundleCompatibility"]["profilePath"],
        "schema/okf_profile.yaml"
    );
    assert_eq!(json["okfBundleCompatibility"]["indexPath"], "index.md");
    assert_eq!(json["okfBundleCompatibility"]["logPath"], "log.md");
    assert_eq!(json["contentPolicy"]["includeRawSources"], false);
}
