use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeWikiFileEntry {
    pub id: u64,
    pub space_id: u64,
    pub logical_path: String,
    pub entry_type: WikiFileEntryType,
    pub artifact_role: String,
    pub drive_bucket: String,
    pub drive_object_key: String,
    pub checksum_sha256_hex: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WikiFileEntryType {
    WikiSchema,
    WikiIndex,
    WikiLog,
    WikiRevision,
    GraphExport,
    ContextPack,
    OutputExport,
}
