use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorManifest {
    pub schema_version: String,
    pub space_id: String,
    pub snapshot_version: String,
    pub base_snapshot_version: Option<String>,
    pub created_at: String,
    pub package_kind: String,
    pub content_policy: MirrorContentPolicy,
    pub llm_wiki_compatibility: LlmWikiCompatibility,
    pub database: MirrorDatabase,
    pub objects_manifest: String,
    pub index_manifests: Vec<String>,
    pub checksums: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorContentPolicy {
    pub include_raw_sources: bool,
    pub include_parsed_artifacts: bool,
    pub include_wiki: bool,
    pub include_embeddings: bool,
    pub include_eval_reports: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmWikiCompatibility {
    pub profile: String,
    pub agent_instruction_path: String,
    pub schema_path: String,
    pub raw_root: String,
    pub wiki_root: String,
    pub index_path: String,
    pub log_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorDatabase {
    pub engine: String,
    pub schema_version: String,
    pub file: String,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaManifest {
    pub schema_version: String,
    pub space_id: String,
    pub package_kind: String,
    pub from_snapshot_version: String,
    pub to_snapshot_version: String,
    pub created_at: String,
    pub requires_schema_version: String,
    pub operations: DeltaOperations,
    pub checksums: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaOperations {
    pub sql_patch: String,
    pub added_objects: String,
    pub changed_objects: String,
    pub deleted_objects: String,
    pub index_patch: String,
}
